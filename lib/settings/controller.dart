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

  late List<int> verifyingKey;
  late List<int> signingKey;
  late Map<String, Contact> contacts;
  late int listenPort;
  late int receivePort;
  late double outputVolume;
  late double inputVolume;
  late double inputSensitivity;

  Future<void> init() async {
    final signingKey = await readSecureData('signingKey');
    final verifyingKey = await readSecureData('verifyingKey');

    if (signingKey != null && verifyingKey != null) {
      this.signingKey = base64Decode(signingKey);
      this.verifyingKey = base64Decode(verifyingKey);
    } else {
      var keypair = generateKeys();
      this.signingKey = keypair.getRange(0, 32).toList();
      this.verifyingKey = keypair.getRange(32, 64).toList();

      await writeSecureData('verifyingKey', base64Encode(this.verifyingKey));
      await writeSecureData('signingKey', base64Encode(this.signingKey));
    }

    contacts = {};

    options.getStringList('contacts')?.forEach((contactStr) async {
      try {
        Contact contact = Contact.parse(s: contactStr);
        contacts[contact.nickname()] = contact;
      } on DartError catch (e) {
        debugPrint('error adding contact: $e');
        return;
      }
    });

    listenPort = options.getInt('listenPort') ?? 45000;
    receivePort = options.getInt('receivePort') ?? 45001;
    outputVolume = options.getDouble('outputVolume') ?? 0;
    inputVolume = options.getDouble('inputVolume') ?? 0;
    inputSensitivity = options.getDouble('inputSensitivity') ?? 0;

    notifyListeners();
  }

  Future<Contact> addContact(
      String nickname, String verifyingKey, String address) async {
    contacts[nickname] = Contact.newContact(
        nickname: nickname, verifyingKey: verifyingKey, address: address);

    await options.setStringList(
        'contacts',
        contacts.values.map((entry) {
          return entry.store();
        }).toList());

    notifyListeners();
    return contacts[nickname]!;
  }

  Future<void> removeContact(String nickname) async {
    contacts.remove(nickname);
    await options.setStringList(
        'contacts',
        contacts.values.map((entry) {
          return entry.store();
        }).toList());
    notifyListeners();
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

  Future<void> updateInputSensitivity(double sensitivity) async {
    inputSensitivity = sensitivity;
    await options.setDouble('inputSensitivity', sensitivity);
    notifyListeners();
  }

  Future<void> writeSecureData(String key, String value) async {
    try {
      await storage.write(key: key, value: value);
    } catch (e) {
      // Handle the exception, maybe log it or show a user-friendly error message
    }
  }

  Future<String?> readSecureData(String key) async {
    try {
      return await storage.read(key: key);
    } catch (e) {
      // Handle the exception
      return null;
    }
  }
}
