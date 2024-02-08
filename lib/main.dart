import 'package:audio_chat/settings/view.dart';
import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:audio_chat/src/rust/api/error.dart';
import 'package:audio_chat/src/rust/frb_generated.dart';
import 'package:audio_chat/settings/controller.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

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
      signingKey: settingsController.signingKey);

  for (var contact in settingsController.contacts.values) {
    await audioChat.addContact(contact: contact);
  }

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
      theme: ThemeData(
        dialogTheme: const DialogTheme(
          surfaceTintColor: Color(0xFF27292A),
        ),
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFFFD6D6D),
          secondary: Color(0xFF994747),
          brightness: Brightness.dark,
          background: Color(0xFF222425),
          secondaryContainer: Color(0xFF191919),
          tertiaryContainer: Color(0xFF27292A),
          // onSurface: Colors.red,
          // TODO surface is raised buttons
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
            Row(children: [
              CallControls(
                  audioChat: audioChat, settingsController: settingsController),
            ]),
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
      padding: const EdgeInsets.all(15.0),
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
          padding: const EdgeInsets.symmetric(vertical: 15.0, horizontal: 10.0),
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

// CallControls
class CallControls extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;

  const CallControls(
      {super.key, required this.audioChat, required this.settingsController});

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: const BoxConstraints(maxWidth: 500.0, maxHeight: 300.0),
      padding: const EdgeInsets.all(15.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Row(
        children: [
          Button(
              text: 'Disconnect',
              onPressed: () async {
                await audioChat.endCall();
              }),
          IconButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(
                      builder: (context) => SettingsPage(
                          controller: settingsController,
                          audioChat: audioChat)),
                );
              },
              icon: const Icon(Icons.settings)),
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

// ContactWidget
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
        subtitle: Text(widget.contact.ipStr()),
        trailing: inCall
            ? IconButton(
                icon: const Icon(Icons.call_end),
                onPressed: () async {
                  await widget.audioChat.endCall();
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
