import 'dart:convert';

import 'package:audio_chat/src/rust/api/error.dart';
import 'package:debug_console/debug_console.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

import '../src/rust/api/contact.dart';
import '../src/rust/api/crypto.dart';

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

  get contacts => profiles[activeProfile]!.contacts;

  get signingKey => profiles[activeProfile]!.signingKey;

  Future<void> init() async {
    // initialize an empty map for profiles
    profiles = {};
    // load a list of profile ids from the options storage
    List<String> profileIds = options.getStringList('profiles') ?? [];

    // load each profile from the secure storage
    for (String id in profileIds) {
      String? keyStr = await storage.read(key: '$id-key');

      // if the key is missing, skip this profile
      if (keyStr == null) {
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
        signingKey: keyBytes.getRange(0, 32).toList(),
        verifyingKey: keyBytes.getRange(32, 64).toList(),
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
    inputSensitivity = options.getDouble('inputSensitivity') ?? -50;
    useDenoise = options.getBool('useDenoise') ?? true;
    outputDevice = options.getString('outputDevice');
    inputDevice = options.getString('inputDevice');
    playCustomRingtones = options.getBool('playCustomRingtones') ?? true;
    customRingtoneFile = options.getString('customRingtoneFile');

    notifyListeners();
  }

  /// This function can raise [DartError] if the verifying key is invalid
  Future<Contact> addContact(
    String nickname,
    String verifyingKey,
  ) async {
    List<int> verifyingKeyBytes = base64Decode(verifyingKey);
    Contact contact = Contact(nickname: nickname, keyBytes: verifyingKeyBytes);
    contacts[contact.id()] = contact;

    await saveContacts();
    notifyListeners();

    return contact;
  }

  Contact? getContact(String id) {
    return contacts[id];
  }

  Future<void> updateContactNickname(Contact contact, String nickname) async {
    contact.setNickname(nickname: nickname);
    await saveContacts();
    notifyListeners();
  }

  Future<void> removeContact(Contact contact) async {
    contacts.remove(contact.id());
    await saveContacts();
    notifyListeners();
  }

  /// Saves the contacts for activeProfile
  Future<void> saveContacts() async {
    // serialized contacts
    Map<String, Map<String, dynamic>> contactsMap = {};

    for (MapEntry<String, Contact> entry in contacts.entries) {
      Map<String, dynamic> contact = {};
      contact['nickname'] = entry.value.nickname();
      contact['verifyingKey'] = entry.value.verifyingKey();
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
    U8Array64 keypair = generateKeys();
    String id = const Uuid().v4();

    await storage.write(key: '$id-key', value: base64Encode(keypair));
    await storage.write(key: '$id-contacts', value: jsonEncode({}));
    await storage.write(key: '$id-nickname', value: nickname);

    profiles[id] = Profile(
      id: id,
      nickname: nickname,
      signingKey: keypair.getRange(0, 32).toList(),
      verifyingKey: keypair.getRange(32, 64).toList(),
      contacts: {},
    );

    await options.setStringList('profiles', profiles.keys.toList());
    notifyListeners();

    return id;
  }

  Future<void> removeProfile(String id) async {
    profiles.remove(id);
    await options.setStringList('profiles', profiles.keys.toList());

    await storage.delete(key: '$id-key');
    await storage.delete(key: '$id-contacts');
    await storage.delete(key: '$id-name');

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

  Future<Map<String, Contact>> loadContacts(String id) async {
    Map<String, Contact> contacts = {};
    String? contactsStr = await storage.read(key: '$id-contacts');

    if (contactsStr != null) {
      Map<String, dynamic> contactsMap = jsonDecode(contactsStr);
      contactsMap.forEach((id, value) {
        String nickname = value['nickname'];
        List<int> verifyingKey = value['verifyingKey'].cast<int>();

        try {
          contacts[id] = Contact.fromParts(
              id: id, nickname: nickname, verifyingKey: verifyingKey);
        } on DartError catch (e) {
          DebugConsole.warning('invalid contact format: $e');
          return;
        }
      });
    }

    return contacts;
  }
}

class Profile {
  final String id;
  final String nickname;
  final List<int> signingKey;
  final List<int> verifyingKey;
  final Map<String, Contact> contacts;

  Profile({
    required this.id,
    required this.nickname,
    required this.signingKey,
    required this.verifyingKey,
    required this.contacts,
  });
}
