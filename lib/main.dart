import 'dart:async';

import 'package:audio_chat/settings/view.dart';
import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:audio_chat/src/rust/api/error.dart';
import 'package:audio_chat/src/rust/frb_generated.dart';
import 'package:audio_chat/settings/controller.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

final GlobalKey<NavigatorState> navigatorKey = GlobalKey<NavigatorState>();

Future<void> main() async {
  await RustLib.init();

  // get logs from rust
  rustSetUp();
  createLogStream().listen((message) {
    if (kDebugMode) {
      print(message);
    }
  });

  const storage = FlutterSecureStorage();
  final SharedPreferences options = await SharedPreferences.getInstance();
  final SettingsController settingsController =
      SettingsController(storage: storage, options: options);
  await settingsController.init();

  final audioChat = await AudioChat.newAudioChat(
      listenPort: settingsController.listenPort,
      receivePort: settingsController.receivePort,
      signingKey: settingsController.signingKey,
      acceptCall: (contact) {
        debugPrint('acceptCall: ${contact.nickname()}');
        return acceptCallPrompt(navigatorKey.currentState!.context, contact);
      });

  for (var contact in settingsController.contacts.values) {
    await audioChat.addContact(contact: contact);
  }

  audioChat.setInputVolume(decibel: settingsController.inputVolume);
  audioChat.setOutputVolume(decibel: settingsController.outputVolume);
  audioChat.setRmsThreshold(decimal: settingsController.inputSensitivity);

  runApp(AudioChatApp(
      audioChat: audioChat, settingsController: settingsController));
}

class AudioChatApp extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;

  const AudioChatApp(
      {super.key, required this.audioChat, required this.settingsController});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      navigatorKey: navigatorKey,
      theme: ThemeData(
        dialogTheme: const DialogTheme(
          surfaceTintColor: Color(0xFF27292A),
        ),
        sliderTheme: const SliderThemeData(
          showValueIndicator: ShowValueIndicator.always,
          overlayColor: Colors.transparent,
        ),
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFFFD6D6D),
          secondary: Color(0xFF994747),
          brightness: Brightness.dark,
          background: Color(0xFF222425),
          secondaryContainer: Color(0xFF191919),
          tertiaryContainer: Color(0xFF27292A),
        ),
      ),
      home: HomePage(
          audioChat: audioChat, settingsController: settingsController),
    );
  }
}

class HomePage extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;

  const HomePage(
      {super.key, required this.audioChat, required this.settingsController});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Padding(
        padding: const EdgeInsets.all(20.0),
        child: Column(
          children: [
            Row(
              children: [
                ContactForm(
                    audioChat: audioChat,
                    settingsController: settingsController),
                const SizedBox(width: 20),
                Expanded(
                    child: ListenableBuilder(
                        listenable: settingsController,
                        builder: (BuildContext context, Widget? child) {
                          return ContactsList(
                              audioChat: audioChat,
                              contacts:
                                  settingsController.contacts.values.toList());
                        }))
              ],
            ),
            const SizedBox(height: 20),
            Expanded(
                child: Row(children: [
              CallControls(
                  audioChat: audioChat, settingsController: settingsController),
            ])),
          ],
        ),
      ),
    );
  }
}

// ContactForm
class ContactForm extends StatefulWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;

  const ContactForm(
      {super.key, required this.audioChat, required this.settingsController});

  @override
  State<ContactForm> createState() => _ContactFormState();
}

class _ContactFormState extends State<ContactForm> {
  final TextEditingController _nicknameInput = TextEditingController();
  final TextEditingController _verifyingKeyInput = TextEditingController();
  final TextEditingController _contactAddressInput = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 15.0, horizontal: 20.0),
      constraints: const BoxConstraints(maxWidth: 300),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text("Add Contact", style: TextStyle(fontSize: 20)),
          const SizedBox(height: 15),
          TextInput(controller: _nicknameInput, labelText: 'Nickname'),
          const SizedBox(height: 10),
          TextInput(
              controller: _contactAddressInput,
              labelText: 'Address',
              hintText: 'host:port or ip:port'),
          const SizedBox(height: 10),
          TextInput(
              controller: _verifyingKeyInput,
              labelText: 'Verifying Key',
              hintText: 'base64 encoded verifying (public) key',
              obscureText: true),
          const SizedBox(height: 10),
          Center(
            child: Button(
              text: 'Add Contact',
              onPressed: () async {
                try {
                  if (_nicknameInput.text.isEmpty ||
                      _verifyingKeyInput.text.isEmpty ||
                      _contactAddressInput.text.isEmpty) {
                    showErrorDialog(context, 'All fields are required');
                    return;
                  }

                  var contact = await widget.settingsController.addContact(
                      _nicknameInput.text,
                      _verifyingKeyInput.text,
                      _contactAddressInput.text);

                  await widget.audioChat.addContact(contact: contact);

                  _contactAddressInput.clear();
                  _nicknameInput.clear();
                  _verifyingKeyInput.clear();
                } on DartError catch (e) {
                  if (!context.mounted) return;
                  showErrorDialog(context, e.message);
                }
              },
            ),
          ),
        ],
      ),
    );
  }
}

// ContactsList
class ContactsList extends StatelessWidget {
  final AudioChat audioChat;
  final List<Contact> contacts;

  const ContactsList(
      {super.key, required this.audioChat, required this.contacts});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints viewportConstraints) {
        return Container(
          constraints: const BoxConstraints(maxHeight: 280),
          padding: const EdgeInsets.symmetric(vertical: 15.0, horizontal: 12.0),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.secondaryContainer,
            borderRadius: BorderRadius.circular(10.0),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Padding(
                  padding: EdgeInsets.only(left: 8.0),
                  child: Text("Contacts", style: TextStyle(fontSize: 20))),
              const SizedBox(height: 10.0),
              Expanded(
                child: ListView.builder(
                  itemCount: contacts.length,
                  itemBuilder: (BuildContext context, int index) {
                    return ContactWidget(
                        contact: contacts[index], audioChat: audioChat);
                  },
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}

/// CallControls
class CallControls extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;

  const CallControls(
      {super.key, required this.audioChat, required this.settingsController});

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: const BoxConstraints(maxWidth: 250.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.tertiaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Padding(
              padding: EdgeInsets.only(top: 10),
              child: Center(
                  child: Text('Call Status: ...',
                      style: TextStyle(fontSize: 18)))),
          const SizedBox(height: 20),
          const Padding(
              padding: EdgeInsets.only(left: 25),
              child: Text('Output Volume', style: TextStyle(fontSize: 15))),
          ListenableBuilder(
              listenable: settingsController,
              builder: (BuildContext context, Widget? child) {
                return Slider(
                    value: settingsController.outputVolume,
                    onChanged: (value) async {
                      await settingsController.updateOutputVolume(value);
                      audioChat.setOutputVolume(decibel: value);
                    },
                    min: -20,
                    max: 25,
                    label:
                        '${settingsController.outputVolume.toStringAsFixed(2)} db');
              }),
          const SizedBox(height: 2),
          const Padding(
              padding: EdgeInsets.only(left: 25),
              child: Text('Input Volume', style: TextStyle(fontSize: 15))),
          ListenableBuilder(
              listenable: settingsController,
              builder: (BuildContext context, Widget? child) {
                return Slider(
                    value: settingsController.inputVolume,
                    onChanged: (value) async {
                      await settingsController.updateInputVolume(value);
                      audioChat.setInputVolume(decibel: value);
                    },
                    min: -20,
                    max: 25,
                    label:
                        '${settingsController.inputVolume.toStringAsFixed(2)} db');
              }),
          const SizedBox(height: 2),
          const Padding(
              padding: EdgeInsets.only(left: 25),
              child: Text('Input Sensitivity', style: TextStyle(fontSize: 15))),
          ListenableBuilder(
              listenable: settingsController,
              builder: (BuildContext context, Widget? child) {
                return Slider(
                    value: settingsController.inputSensitivity,
                    onChanged: (value) async {
                      await settingsController.updateInputSensitivity(value);
                      audioChat.setRmsThreshold(decimal: value);
                    },
                    min: -15,
                    max: 50,
                    label:
                        '${settingsController.inputSensitivity.toStringAsFixed(2)} db');
              }),
          const Spacer(),
          Container(
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.secondaryContainer,
                borderRadius: const BorderRadius.only(
                    bottomLeft: Radius.circular(10.0),
                    bottomRight: Radius.circular(10.0)),
              ),
              child: Padding(
                padding: const EdgeInsets.all(5.0),
                child: Center(
                    child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    IconButton(onPressed: () {}, icon: const Icon(Icons.mic)),
                    IconButton(
                        onPressed: () {}, icon: const Icon(Icons.headphones)),
                    IconButton(
                        onPressed: () {
                          Navigator.push(
                              context,
                              MaterialPageRoute(
                                builder: (context) => SettingsPage(
                                    controller: settingsController,
                                    audioChat: audioChat),
                              ));
                        },
                        icon: const Icon(Icons.settings)),
                    IconButton(
                        onPressed: () {}, icon: const Icon(Icons.screen_share)),
                  ],
                )),
              ))
        ],
      ),
    );
  }
}

class ContactWidget extends StatefulWidget {
  final Contact contact;
  final AudioChat audioChat;

  const ContactWidget(
      {super.key, required this.contact, required this.audioChat});

  @override
  State<ContactWidget> createState() => _ContactWidgetState();
}

/// ContactWidget
class _ContactWidgetState extends State<ContactWidget> {
  late bool inCall = false;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.all(5.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.tertiaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: ListTile(
        leading: const CircleAvatar(
          child: Icon(Icons.person),
        ),
        title: Text(widget.contact.nickname()),
        subtitle: Text(widget.contact.addressStr()),
        trailing: inCall
            ? IconButton(
                icon: const Icon(Icons.call_end),
                onPressed: () async {
                  await widget.audioChat.endCall();
                  setState(() {
                    inCall = false;
                  });
                },
              )
            : IconButton(
                icon: const Icon(Icons.call),
                onPressed: () async {
                  try {
                    await widget.audioChat.sayHello(contact: widget.contact);
                    setState(() {
                      inCall = true;
                    });
                  } on DartError catch (e) {
                    if (!context.mounted) return;
                    showErrorDialog(context, e.message);
                  }
                },
              ),
      ),
    );
  }
}

/// Custom Button Widget
class Button extends StatelessWidget {
  final String text;
  final VoidCallback onPressed;

  const Button({super.key, required this.text, required this.onPressed});

  @override
  Widget build(BuildContext context) {
    return ElevatedButton(
      onPressed: onPressed,
      style: ButtonStyle(
        backgroundColor:
            MaterialStateProperty.all(Theme.of(context).colorScheme.primary),
        foregroundColor: MaterialStateProperty.all(Colors.white),
        overlayColor:
            MaterialStateProperty.all(Theme.of(context).colorScheme.secondary),
      ),
      child: Text(text),
    );
  }
}

/// Custom TextInput Widget
class TextInput extends StatelessWidget {
  final String labelText;
  final String? hintText;
  final TextEditingController controller;
  final bool? obscureText;
  final void Function()? onEditingComplete;

  const TextInput(
      {super.key,
      required this.labelText,
      this.hintText,
      required this.controller,
      this.obscureText,
      this.onEditingComplete});

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      obscureText: obscureText ?? false,
      onEditingComplete: onEditingComplete,
      decoration: InputDecoration(
        labelText: labelText,
        hintText: hintText,
        hintStyle: const TextStyle(fontSize: 13, fontStyle: FontStyle.normal),
        fillColor: Theme.of(context).colorScheme.tertiaryContainer,
        filled: true,
        border: const OutlineInputBorder(
          borderRadius: BorderRadius.all(Radius.circular(10.0)),
        ),
        contentPadding: const EdgeInsets.all(10.0),
      ),
    );
  }
}

// Function to show error modal dialog
void showErrorDialog(BuildContext context, String errorMessage) {
  showDialog(
    context: context,
    builder: (BuildContext context) {
      return AlertDialog(
        title: const Text('An Error Occurred'),
        content: Text(errorMessage),
        actions: <Widget>[
          TextButton(
            child: const Text('Close'),
            onPressed: () {
              Navigator.of(context).pop(); // Dismiss the dialog
            },
          ),
        ],
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(10),
        ),
      );
    },
  );
}

Future<bool> acceptCallPrompt(BuildContext context, Contact contact) async {
  const timeout = Duration(seconds: 10);
  bool? result = await showDialog<bool>(
    context: context,
    builder: (BuildContext context) {
      Timer(timeout, () {
        if (context.mounted) {
          Navigator.of(context).pop(false);
        }
      });

      return AlertDialog(
        title: Text('Accept call from ${contact.nickname()}?'),
        actions: <Widget>[
          TextButton(
            child: const Text('Deny'),
            onPressed: () =>
                Navigator.of(context).pop(false), // Return false (deny)
          ),
          TextButton(
            child: const Text('Accept'),
            onPressed: () =>
                Navigator.of(context).pop(true), // Return true (accept)
          ),
        ],
      );
    },
  );

  // If the dialog is dismissed by the timeout, result will be null. Treating null as false here.
  return result ?? false;
}
