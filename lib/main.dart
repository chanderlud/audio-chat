import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:audio_chat/src/rust/api/error.dart';
import 'package:audio_chat/src/rust/frb_generated.dart';
import 'package:audio_chat/settings.dart';
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
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFFFD6D6D),
          brightness: Brightness.dark,
          background: Color(0xFF222425),
          secondaryContainer: Color(0xFF191919),
          tertiaryContainer: Color(0xFF27292A),
          // cardColor: const Color(0xFF191919),
          // backgroundColor: const Color.fromARGB(255, 34, 36, 37),
        ),
      ),
      home: AudioChatView(
          audioChat: audioChat, settingsController: settingsController),
    );
  }
}

class AudioChatView extends StatefulWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;

  const AudioChatView(
      {super.key, required this.audioChat, required this.settingsController});

  @override
  State<StatefulWidget> createState() => HomePage();
}

class HomePage extends State<AudioChatView> {
  final TextEditingController _nicknameInput = TextEditingController();
  final TextEditingController _verifyingKeyInput = TextEditingController();
  final TextEditingController _contactAddressInput = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
        builder: (BuildContext context, BoxConstraints viewportConstraints) {
      return Scaffold(
        body: Padding(
          padding: const EdgeInsets.all(20.0),
          child: Wrap(
            spacing: 20,
            runSpacing: 20,
            children: [
              Container(
                constraints: const BoxConstraints(maxWidth: 350.0),
                padding: const EdgeInsets.all(15.0),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.secondaryContainer,
                  borderRadius: BorderRadius.circular(10.0),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text("Create Contact", style: TextStyle(fontSize: 20)),
                    TextField(
                      controller: _nicknameInput,
                      decoration: const InputDecoration(
                        labelText: 'Nickname',
                      ),
                    ),
                    TextField(
                      controller: _verifyingKeyInput,
                      decoration: const InputDecoration(
                        labelText: 'Verifying Key',
                        hintText: 'base64 encoded verifying (public) key',
                      ),
                    ),
                    TextField(
                      controller: _contactAddressInput,
                      decoration: const InputDecoration(
                        labelText: 'Address',
                        hintText: 'host:port or ip:port',
                      ),
                    ),
                    const SizedBox(height: 10),
                    ElevatedButton(
                      onPressed: () async {
                        try {
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
                      child: const Text('Add Contact'),
                    ),
                  ],
                ),
              ),
              Container(
                constraints: const BoxConstraints(maxWidth: 500.0, maxHeight: 270.0),
                padding: const EdgeInsets.all(15.0),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.secondaryContainer,
                  borderRadius: BorderRadius.circular(10.0),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Text("Contacts", style: TextStyle(fontSize: 20)),
                    const SizedBox(height: 10),
                    Flexible(
                      child: SingleChildScrollView(
                        child: ConstrainedBox(
                          constraints: BoxConstraints(
                            minHeight: viewportConstraints.maxHeight,
                          ),
                          child: ListenableBuilder(
                            listenable: widget.settingsController,
                            builder: (BuildContext context, Widget? child) {
                              return Column(
                                children: widget.settingsController.contacts.values
                                    .map((contact) => ContactWidget(
                                    contact: contact,
                                    audioChat: widget.audioChat))
                                    .toList(),
                              );
                            },
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              Container(
                constraints: const BoxConstraints(maxWidth: 500.0, maxHeight: 300.0),
                padding: const EdgeInsets.all(15.0),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.secondaryContainer,
                  borderRadius: BorderRadius.circular(10.0),
                ),
                child: ElevatedButton(
                  onPressed: () async {
                    await widget.audioChat.endCall();
                  },
                  child: const Text('Disconnect'),
                ),
              )
            ],
          ),
        ),
      );
    });
  }
}

class ContactWidget extends StatefulWidget {
  final Contact contact;
  final AudioChat audioChat;

  const ContactWidget(
      {super.key, required this.contact, required this.audioChat});

  @override
  State<StatefulWidget> createState() => ContactWidgetState();
}

class ContactWidgetState extends State<ContactWidget> {
  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.all(5.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.tertiaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: ListTile(
        leading: const Icon(Icons.person),
        title: Text(widget.contact.nickname()),
        subtitle: Text(widget.contact.ipStr()),
        trailing: IconButton(
          icon: const Icon(Icons.call),
          onPressed: () async {
            try {
              await widget.audioChat.sayHello(contact: widget.contact);
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
        title: const Text('Error Occurred'),
        content: Text(errorMessage),
        actions: <Widget>[
          FloatingActionButton(
            child: const Text('Close'),
            onPressed: () {
              Navigator.of(context).pop(); // Dismiss the dialog
            },
          ),
        ],
      );
    },
  );
}
