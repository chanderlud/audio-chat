import 'dart:convert';
import 'dart:typed_data';

import 'package:audio_chat/src/rust/api/error.dart';
import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

import '../console.dart';
import '../src/rust/api/contact.dart';
import '../src/rust/api/crypto.dart';

const defaultRelayAddress = '5.78.76.47:40142';
const defaultRelayId = '12D3KooWMpeKAbMK4BTPsQY3rG7XwtdstseHGcq7kffY8LToYYKK';
const defaultOverlayEnabled = false;
const defaultOverlayX = 0.0;
const defaultOverlayY = 0.0;
const defaultOverlayWidth = 600.0;
const defaultOverlayHeight = 38.0;
const defaultOverlayFontFamily = 'Inconsolata';
const defaultOverlayFontColor = 0xFFFFFFFF;
const defaultOverlayFontHeight = 36;
const defaultOverlayFontBackgroundColor = 0x80000000;

class SettingsController with ChangeNotifier {
  final FlutterSecureStorage storage;
  final SharedPreferences options;

  SettingsController({required this.storage, required this.options});

  /// the ids of all available profiles
  late Map<String, Profile> profiles;

  /// the id of the active profile
  late String activeProfile;

  /// the output volume for calls (applies to output device)
  late double outputVolume;

  /// the input volume for calls (applies to input device)
  late double inputVolume;

  /// the output volume for sound effects
  late double soundVolume;

  /// the input sensitivity for calls
  late double inputSensitivity;

  /// whether to use rnnoise
  late bool useDenoise;

  /// the output device for calls
  late String? outputDevice;

  /// the input device for calls
  late String? inputDevice;

  /// whether to play custom ringtones
  late bool playCustomRingtones;

  /// the custom ringtone file
  late String? customRingtoneFile;

  /// the network configuration
  late NetworkConfig networkConfig;

  /// the screenshare configuration
  late ScreenshareConfig screenshareConfig;

  /// the overlay configuration
  late OverlayConfig overlayConfig;

  /// the codec configuration
  late CodecConfig codecConfig;

  /// the name of a denoise model
  late String? denoiseModel;

  Map<String, Contact> get contacts => profiles[activeProfile]!.contacts;

  List<int> get keypair => profiles[activeProfile]!.keypair;

  String get peerId => profiles[activeProfile]!.peerId;

  Future<void> init() async {
    // initialize an empty map for profiles
    profiles = {};
    // load a list of profile ids from the options storage
    List<String> profileIds = options.getStringList('profiles') ?? [];

    // load each profile from the secure storage
    for (String id in profileIds) {
      String? keyStr = await storage.read(key: '$id-keypair');
      String? peerId = await storage.read(key: '$id-peerId');

      // if the key is missing, skip this profile
      if (keyStr == null || peerId == null) {
        await removeProfile(id);
        continue;
      }

      // load the contacts for this profile
      Map<String, Contact> contacts = await loadContacts(id);
      String nickname =
          await storage.read(key: '$id-nickname') ?? 'Unnamed Profile';
      List<int> keyBytes = base64Decode(keyStr);

      // construct the profile object and add it to the profiles map
      profiles[id] = Profile(
        id: id,
        nickname: nickname,
        peerId: peerId,
        keypair: keyBytes,
        contacts: contacts,
      );
    }

    if (profiles.isEmpty) {
      // if there are no profiles, create a default profile
      activeProfile = await createProfile('Default');
    } else {
      // if there are profiles, load the active profile or use the first profile if needed
      activeProfile = options.getString('activeProfile') ?? profiles.keys.first;
    }

    // ensure that the active profile is now persisted
    await setActiveProfile(activeProfile);

    // load the remaining options with default values as needed
    outputVolume = options.getDouble('outputVolume') ?? 0;
    inputVolume = options.getDouble('inputVolume') ?? 0;
    soundVolume = options.getDouble('soundVolume') ?? -10;
    inputSensitivity = options.getDouble('inputSensitivity') ?? -16;
    useDenoise = options.getBool('useDenoise') ?? true;
    outputDevice = options.getString('outputDevice');
    inputDevice = options.getString('inputDevice');
    playCustomRingtones = options.getBool('playCustomRingtones') ?? true;
    customRingtoneFile = options.getString('customRingtoneFile');
    denoiseModel = options.getString('denoiseModel');

    networkConfig = loadNetworkConfig();
    screenshareConfig = await loadScreenshareConfig();
    overlayConfig = loadOverlayConfig();
    codecConfig = loadCodecConfig();

    notifyListeners();
  }

  /// This function can raise [DartError] if the verifying key is invalid
  Contact addContact(
    String nickname,
    String peerId,
  ) {
    Contact contact = Contact(nickname: nickname, peerId: peerId);
    contacts[contact.id()] = contact;

    saveContacts();
    return contact;
  }

  Contact? getContact(String id) {
    return contacts[id];
  }

  void removeContact(Contact contact) {
    contacts.remove(contact.id());
    saveContacts();
  }

  /// Saves the contacts for activeProfile
  Future<void> saveContacts() async {
    // notify listeners right away because the contacts are already updated
    notifyListeners();

    // serialized contacts
    Map<String, Map<String, dynamic>> contactsMap = {};

    for (MapEntry<String, Contact> entry in contacts.entries) {
      Map<String, dynamic> contact = {};
      contact['nickname'] = entry.value.nickname();
      contact['peerId'] = entry.value.peerId();
      contactsMap[entry.key] = contact;
    }

    await storage.write(
      key: '$activeProfile-contacts',
      value: jsonEncode(contactsMap),
    );
  }

  Future<void> updateOutputVolume(double volume) async {
    outputVolume = volume;
    await options.setDouble('outputVolume', volume);
    notifyListeners();
  }

  Future<void> updateInputVolume(double volume) async {
    inputVolume = volume;
    await options.setDouble('inputVolume', volume);
    notifyListeners();
  }

  Future<void> updateSoundVolume(double volume) async {
    soundVolume = volume;
    await options.setDouble('soundVolume', volume);
    notifyListeners();
  }

  Future<void> updateInputSensitivity(double sensitivity) async {
    inputSensitivity = sensitivity;
    await options.setDouble('inputSensitivity', sensitivity);
    notifyListeners();
  }

  Future<void> updateUseDenoise(bool use) async {
    useDenoise = use;
    await options.setBool('useDenoise', use);
    notifyListeners();
  }

  Future<void> updateOutputDevice(String? device) async {
    outputDevice = device;

    if (device != null) {
      await options.setString('outputDevice', device);
    } else {
      await options.remove('outputDevice');
    }

    notifyListeners();
  }

  Future<void> updateInputDevice(String? device) async {
    inputDevice = device;

    if (device != null) {
      await options.setString('inputDevice', device);
    } else {
      await options.remove('inputDevice');
    }

    notifyListeners();
  }

  Future<void> updatePlayCustomRingtones(bool play) async {
    playCustomRingtones = play;
    await options.setBool('playCustomRingtones', play);
    notifyListeners();
  }

  Future<void> updateCustomRingtoneFile(String? file) async {
    customRingtoneFile = file;

    if (file != null) {
      await options.setString('customRingtoneFile', file);
    } else {
      await options.remove('customRingtoneFile');
    }

    notifyListeners();
  }

  Future<String> createProfile(String nickname) async {
    String peerId;
    Uint8List keypair;

    (peerId, keypair) = generateKeys();
    String id = const Uuid().v4();

    await storage.write(key: '$id-keypair', value: base64Encode(keypair));
    await storage.write(key: '$id-peerId', value: peerId);
    await storage.write(key: '$id-contacts', value: jsonEncode({}));
    await storage.write(key: '$id-nickname', value: nickname);

    profiles[id] = Profile(
      id: id,
      nickname: nickname,
      peerId: peerId,
      keypair: keypair,
      contacts: {},
    );

    await options.setStringList('profiles', profiles.keys.toList());
    notifyListeners();

    return id;
  }

  Future<void> removeProfile(String id) async {
    profiles.remove(id);
    await options.setStringList('profiles', profiles.keys.toList());

    await storage.delete(key: '$id-keypair');
    await storage.delete(key: '$id-peerId');
    await storage.delete(key: '$id-contacts');
    await storage.delete(key: '$id-nickname');

    if (activeProfile == id) {
      await setActiveProfile(profiles.keys.first);
    } else {
      notifyListeners();
    }
  }

  Future<void> setActiveProfile(String id) async {
    activeProfile = id;
    await options.setString('activeProfile', id);
    notifyListeners();
  }

  Future<void> setDenoiseModel(String? model) async {
    denoiseModel = model;

    if (model != null) {
      await options.setString('denoiseModel', model);
    } else {
      await options.remove('denoiseModel');
    }

    notifyListeners();
  }

  Future<Map<String, Contact>> loadContacts(String id) async {
    Map<String, Contact> contacts = {};
    String? contactsStr = await storage.read(key: '$id-contacts');

    if (contactsStr != null) {
      Map<String, dynamic> contactsMap = jsonDecode(contactsStr);
      contactsMap.forEach((id, value) {
        String nickname = value['nickname'];
        String peerId = value['peerId'];

        try {
          contacts[id] =
              Contact.fromParts(id: id, nickname: nickname, peerId: peerId);
        } on DartError catch (e) {
          DebugConsole.warn('invalid contact format: $e');
          return;
        }
      });
    }

    return contacts;
  }

  NetworkConfig loadNetworkConfig() {
    try {
      return NetworkConfig(
        relayAddress: options.getString('relayAddress') ?? defaultRelayAddress,
        relayId: options.getString('relayId') ?? defaultRelayId,
      );
    } on DartError catch (e) {
      DebugConsole.warn('invalid network config values: $e');
      return NetworkConfig(
          relayAddress: defaultRelayAddress, relayId: defaultRelayId);
    }
  }

  Future<void> saveNetworkConfig() async {
    await options.setString(
        'relayAddress', await networkConfig.getRelayAddress());
    await options.setString('relayId', await networkConfig.getRelayId());
  }

  Future<ScreenshareConfig> loadScreenshareConfig() async {
    return await ScreenshareConfig.newInstance(
      configStr: options.getString('screenshareConfig') ?? '',
    );
  }

  Future<void> saveScreenshareConfig() async {
    await options.setString('screenshareConfig', screenshareConfig.toString());
  }

  CodecConfig loadCodecConfig() {
    return CodecConfig(
      enabled: options.getBool('codecEnabled') ?? true,
      vbr: options.getBool('codecVbr') ?? true,
      residualBits: options.getDouble('codecResidualBits') ?? 5.0,
    );
  }

  Future<void> saveCodecConfig() async {
    (bool, bool, double) values = codecConfig.toValues();
    await options.setBool('codecEnabled', values.$1);
    await options.setBool('codecVbr', values.$2);
    await options.setDouble('codecResidualBits', values.$3);
  }

  OverlayConfig loadOverlayConfig() {
    try {
      return OverlayConfig(
        enabled: options.getBool('overlayEnabled') ?? defaultOverlayEnabled,
        x: options.getDouble('overlayX') ?? defaultOverlayX,
        y: options.getDouble('overlayY') ?? defaultOverlayY,
        width: options.getDouble('overlayWidth') ?? defaultOverlayWidth,
        height: options.getDouble('overlayHeight') ?? defaultOverlayHeight,
        fontFamily:
            options.getString('overlayFontFamily') ?? defaultOverlayFontFamily,
        fontColor: Color(
            options.getInt('overlayFontColor') ?? defaultOverlayFontColor),
        fontHeight:
            options.getInt('overlayFontHeight') ?? defaultOverlayFontHeight,
        backgroundColor: Color(options.getInt('overlayBackgroundColor') ??
            defaultOverlayFontBackgroundColor),
      );
    } on DartError catch (e) {
      DebugConsole.warn('invalid overlay config format: $e');

      return OverlayConfig(
        enabled: defaultOverlayEnabled,
        x: defaultOverlayX,
        y: defaultOverlayY,
        width: defaultOverlayWidth,
        height: defaultOverlayHeight,
        fontFamily: defaultOverlayFontFamily,
        fontColor: const Color(defaultOverlayFontColor),
        fontHeight: defaultOverlayFontHeight,
        backgroundColor: const Color(defaultOverlayFontBackgroundColor),
      );
    }
  }

  Future<void> saveOverlayConfig() async {
    await options.setBool('overlayEnabled', overlayConfig.enabled);
    await options.setDouble('overlayX', overlayConfig.x);
    await options.setDouble('overlayY', overlayConfig.y);
    await options.setDouble('overlayWidth', overlayConfig.width);
    await options.setDouble('overlayHeight', overlayConfig.height);
    await options.setString('overlayFontFamily', overlayConfig.fontFamily);
    await options.setInt('overlayFontColor', overlayConfig.fontColor.value);
    await options.setInt('overlayFontHeight', overlayConfig.fontHeight);
    await options.setInt(
        'overlayBackgroundColor', overlayConfig.backgroundColor.value);
  }
}

class OverlayConfig {
  bool enabled;
  double x;
  double y;
  double width;
  double height;
  String fontFamily;
  Color fontColor;
  int fontHeight;
  Color backgroundColor;

  OverlayConfig({
    required this.enabled,
    required this.x,
    required this.y,
    required this.width,
    required this.height,
    required this.fontFamily,
    required this.fontColor,
    required this.fontHeight,
    required this.backgroundColor,
  });
}

class Profile {
  final String id;
  final String nickname;
  final String peerId;
  final List<int> keypair;
  final Map<String, Contact> contacts;

  Profile({
    required this.id,
    required this.nickname,
    required this.peerId,
    required this.keypair,
    required this.contacts,
  });
}

int argb(Color color) {
  return (color.alpha << 24) |
      (color.red << 16) |
      (color.green << 8) |
      color.blue;
}
