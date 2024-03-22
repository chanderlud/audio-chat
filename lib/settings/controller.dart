import 'dart:convert';

import 'package:audio_chat/src/rust/api/error.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../src/rust/api/contact.dart';
import '../src/rust/api/crypto.dart';

class SettingsController with ChangeNotifier {
  final FlutterSecureStorage storage;
  final SharedPreferences options;

  SettingsController({required this.storage, required this.options});

  /// public key
  late List<int> verifyingKey;

  /// private key
  late List<int> signingKey;

  /// contacts
  late Map<String, Contact> contacts;

  /// the listen port (TCP)
  late int listenPort;

  /// the receive port (UDP)
  late int receivePort;

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

  Future<void> init() async {
    final signingKey = await storage.read(key: 'signingKey');
    final verifyingKey = await storage.read(key: 'verifyingKey');

    if (signingKey != null && verifyingKey != null) {
      this.signingKey = base64Decode(signingKey);
      this.verifyingKey = base64Decode(verifyingKey);
    } else {
      U8Array64 keypair = generateKeys();
      this.signingKey = keypair.getRange(0, 32).toList();
      this.verifyingKey = keypair.getRange(32, 64).toList();

      await storage.write(
          key: 'verifyingKey', value: base64Encode(this.verifyingKey));
      await storage.write(
          key: 'signingKey', value: base64Encode(this.signingKey));
    }

    contacts = {};

    options.getStringList('contacts')?.forEach((contactStr) async {
      try {
        Contact contact = Contact.parse(s: contactStr);
        contacts[contact.id()] = contact;
      } on DartError catch (e) {
        debugPrint('error adding contact: $e');
        return;
      }
    });

    listenPort = options.getInt('listenPort') ?? 45000;
    receivePort = options.getInt('receivePort') ?? 45001;
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

  Future<Contact> addContact(
    String nickname,
    String verifyingKey,
    String address,
  ) async {
    Contact contact = Contact(
        nickname: nickname, verifyingKey: verifyingKey, address: address);
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

  Future<void> updateContactAddress(Contact contact, String address) async {
    contact.setAddress(address: address);
    await saveContacts();
    notifyListeners();
  }

  Future<void> removeContact(Contact contact) async {
    contacts.remove(contact.id());
    await saveContacts();
    notifyListeners();
  }

  Future<void> saveContacts() async {
    await options.setStringList(
        'contacts',
        contacts.values.map((entry) {
          return entry.store();
        }).toList());
  }

  Future<void> updateListenPort(int port) async {
    listenPort = port;
    await options.setInt('listenPort', port);
    notifyListeners();
  }

  Future<void> updateReceivePort(int port) async {
    receivePort = port;
    await options.setInt('receivePort', port);
    notifyListeners();
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
}
