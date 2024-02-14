import 'dart:convert';

import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:audio_chat/src/rust/api/error.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

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

    notifyListeners();
  }

  Future<void> addContact(
    String nickname,
    String verifyingKey,
    String address,
  ) async {
    Contact contact = Contact.newContact(
        nickname: nickname, verifyingKey: verifyingKey, address: address);
    contacts[contact.id()] = contact;

    await saveContacts();
    notifyListeners();
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
}
