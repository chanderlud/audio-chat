import 'dart:async';

import 'package:audio_chat/settings/view.dart';
import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:audio_chat/src/rust/api/error.dart';
import 'package:audio_chat/src/rust/frb_generated.dart';
import 'package:audio_chat/settings/controller.dart';
import 'package:debug_console/debug_console.dart';
import 'package:flutter/material.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

final GlobalKey<NavigatorState> navigatorKey = GlobalKey<NavigatorState>();

Future<void> main() async {
  await RustLib.init();

  // get logs from rust
  rustSetUp();
  createLogStream().listen((message) {
    DebugConsole.log(message);
  });

  const storage = FlutterSecureStorage();
  final SharedPreferences options = await SharedPreferences.getInstance();

  final SettingsController settingsController =
      SettingsController(storage: storage, options: options);
  await settingsController.init();

  final StateController callStateController = StateController();

  final audioChat = await AudioChat.newAudioChat(
      listenPort: settingsController.listenPort,
      receivePort: settingsController.receivePort,
      signingKey: settingsController.signingKey,
      rmsThreshold: settingsController.inputSensitivity,
      inputVolume: settingsController.inputVolume,
      outputVolume: settingsController.outputVolume,
      acceptCall: (contact) async {
        if (callStateController.isCallActive) {
          return false;
        } else if (navigatorKey.currentState == null) {
          DebugConsole.warning('navigatorKey.currentState is null');
          return false;
        }

        bool accepted =
            await acceptCallPrompt(navigatorKey.currentState!.context, contact);

        if (accepted) {
          callStateController.setStatus('Active');
          callStateController.setActiveContact(contact);
        }

        return accepted;
      },
      callEnded: (message) {
        callStateController.setActiveContact(null);
        callStateController.setStatus('Inactive');

        if (message.isNotEmpty && navigatorKey.currentState != null) {
          showErrorDialog(navigatorKey.currentState!.context, message);
        } else {
          DebugConsole.debug('call ended, not showing popup');
        }
      },
      getContact: (ipStr) {
        Contact? contact = settingsController.contacts.values
            .firstWhere((contact) => contact.ipStr() == ipStr);
        return contact.pubClone();
      });

  runApp(AudioChatApp(
      audioChat: audioChat,
      settingsController: settingsController,
      callStateController: callStateController));
}

class AudioChatApp extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController callStateController;

  const AudioChatApp(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.callStateController});

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
          audioChat: audioChat,
          settingsController: settingsController,
          callStateController: callStateController),
    );
  }
}

class HomePage extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController callStateController;

  const HomePage(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.callStateController});

  @override
  Widget build(BuildContext context) {
    return DebugConsolePopup(child: Scaffold(
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
                              settingsController.contacts.values.toList(),
                              stateController: callStateController,
                              settingsController: settingsController);
                        }))
              ],
            ),
            const SizedBox(height: 20),
            Expanded(
                child: Row(children: [
                  CallControls(
                      audioChat: audioChat,
                      settingsController: settingsController,
                      stateController: callStateController),
                ])),
          ],
        ),
      ),
    ));
  }
}

/// ContactForm
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

                  await widget.settingsController.addContact(
                      _nicknameInput.text,
                      _verifyingKeyInput.text,
                      _contactAddressInput.text);

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

/// ContactsList
class ContactsList extends StatelessWidget {
  final AudioChat audioChat;
  final StateController stateController;
  final SettingsController settingsController;
  final List<Contact> contacts;

  const ContactsList(
      {super.key,
      required this.audioChat,
      required this.contacts,
      required this.stateController,
      required this.settingsController});

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
              Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 8.0),
                  child: Row(
                    children: [
                      const Text("Contacts", style: TextStyle(fontSize: 20)),
                      const SizedBox(width: 10),
                      IconButton(
                          onPressed: () {
                            Navigator.push(
                                context,
                                MaterialPageRoute(
                                  builder: (context) => EditContacts(
                                      settingsController: settingsController,
                                      stateController: stateController,
                                      audioChat: audioChat,
                                      contacts: contacts),
                                ));
                          },
                          icon: const Icon(Icons.edit))
                    ],
                  )),
              const SizedBox(height: 10.0),
              Expanded(
                child: Container(
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.tertiaryContainer,
                    borderRadius: BorderRadius.circular(10.0),
                  ),
                  child: ListView.builder(
                    itemCount: contacts.length,
                    itemBuilder: (BuildContext context, int index) {
                      return ListenableBuilder(
                          listenable: stateController,
                          builder: (BuildContext context, Widget? child) {
                            return ContactWidget(
                                contact: contacts[index],
                                audioChat: audioChat,
                                controller: stateController);
                          });
                    },
                  ),
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
  final StateController stateController;

  const CallControls(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.stateController});

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
          Padding(
              padding: const EdgeInsets.only(top: 10),
              child: Center(
                  child: ListenableBuilder(
                      listenable: stateController,
                      builder: (BuildContext context, Widget? child) {
                        return Text(stateController.status,
                            style: const TextStyle(fontSize: 20));
                      }))),
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
                    max: 35,
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
                    max: 35,
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
                    min: -100,
                    max: 0,
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
                        onPressed: () {
                          // TODO get this working
                          // stateController.deafen();
                          // audioChat.setDeafened(deafened: stateController.isDeafened);
                        }, icon: stateController.isDeafened ? const Icon(Icons.volume_off) : const Icon(Icons.volume_up)),
                    IconButton(
                        onPressed: () {
                          Navigator.push(
                              context,
                              MaterialPageRoute(
                                builder: (context) => SettingsPage(
                                    controller: settingsController,
                                    audioChat: audioChat,
                                    callStateController: stateController),
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

/// ContactWidget
class ContactWidget extends StatelessWidget {
  final Contact contact;
  final AudioChat audioChat;
  final StateController controller;

  const ContactWidget(
      {super.key,
      required this.contact,
      required this.audioChat,
      required this.controller});

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.all(5.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: ListTile(
        leading: const CircleAvatar(
          child: Icon(Icons.person),
        ),
        title: Text(contact.nickname()),
        subtitle: Text(contact.addressStr()),
        trailing: controller.isActiveContact(contact)
            ? IconButton(
                icon: const Icon(Icons.call_end),
                onPressed: () async {
                  await audioChat.endCall();
                  controller.setActiveContact(null);
                },
              )
            : IconButton(
                icon: const Icon(Icons.call),
                onPressed: () async {
                  if (controller.isCallActive) {
                    showErrorDialog(context, 'There is a call already active');
                    return;
                  }

                  controller.setStatus('Connecting');
                  SoundHandle handle = await audioChat.playSound(name: 'outgoing');

                  try {
                    if (await audioChat.sayHello(contact: contact)) {
                      controller.setActiveContact(contact);
                      controller.setStatus('Active');
                      handle.cancel();
                    } else {
                      controller.setStatus('Inactive');
                      handle.cancel();
                      if (!context.mounted) return;
                      showErrorDialog(context, 'Call failed');
                    }
                  } on DartError catch (e) {
                    controller.setStatus('Inactive');
                    handle.cancel();
                    if (!context.mounted) return;
                    showErrorDialog(context, e.message);
                  }
                },
              ),
      ),
    );
  }
}

class EditContactWidget extends StatefulWidget {
  final Contact contact;
  final SettingsController settingsController;
  final StateController stateController;
  final AudioChat audioChat;

  const EditContactWidget(
      {super.key,
      required this.contact,
      required this.settingsController,
      required this.audioChat,
      required this.stateController});

  @override
  State<StatefulWidget> createState() => _EditContactWidgetState();
}

class _EditContactWidgetState extends State<EditContactWidget> {
  late final TextEditingController _nicknameInput;
  late final TextEditingController _addressInput;

  @override
  void initState() {
    super.initState();
    _nicknameInput = TextEditingController(text: widget.contact.nickname());
    _addressInput = TextEditingController(text: widget.contact.addressStr());
  }

  @override
  void dispose() {
    _nicknameInput.dispose();
    _addressInput.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.all(5.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: ListTile(
          title: Padding(
            padding: const EdgeInsets.only(bottom: 10),
            child: TextInput(
                enabled:
                    !widget.stateController.isActiveContact(widget.contact),
                controller: _nicknameInput,
                labelText: 'Nickname',
                onChanged: (value) {
                  widget.settingsController
                      .updateContactNickname(widget.contact, value);
                  DebugConsole.debug('nickname updated ${widget.contact.nickname()}');
                }),
          ),
          subtitle: TextInput(
              enabled:
              !widget.stateController.isActiveContact(widget.contact),
              controller: _addressInput,
              labelText: 'Address',
              onChanged: (value) {
                try {
                  widget.settingsController
                      .updateContactAddress(widget.contact, value);
                } on DartError catch (_) {
                  return; // ignore
                }
              }),
          trailing: IconButton(
              onPressed: () {
                if (!widget.stateController.isActiveContact(widget.contact)) {
                  widget.settingsController.removeContact(widget.contact);
                } else {
                  showErrorDialog(context, 'Cannot delete a contact while in an active call');
                }
              },
              icon: const Icon(Icons.delete))),
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
  final bool? enabled;
  final void Function(String)? onChanged;

  const TextInput(
      {super.key,
      required this.labelText,
      this.hintText,
      required this.controller,
      this.obscureText,
      this.enabled,
      this.onChanged});

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      obscureText: obscureText ?? false,
      enabled: enabled,
      onChanged: onChanged,
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

/// A controller which helps bridge the gap between the UI and the audio chat
class StateController extends ChangeNotifier {
  Contact? _activeContact;
  String _status = 'Inactive';
  bool deafened = false;

  Contact? get activeContact => _activeContact;
  String get status => _status;
  bool get isCallActive => _activeContact != null;
  bool get isDeafened => deafened;

  void setActiveContact(Contact? contact) {
    _activeContact = contact;
    notifyListeners();
  }

  void setStatus(String status) {
    _status = status;
    notifyListeners();
  }

  bool isActiveContact(Contact contact) {
    return _activeContact?.equals(other: contact) ?? false;
  }

  void deafen() {
    deafened = !deafened;
    notifyListeners();
  }
}

class EditContacts extends StatelessWidget {
  final SettingsController settingsController;
  final StateController stateController;
  final AudioChat audioChat;
  final List<Contact> contacts;

  const EditContacts(
      {super.key,
      required this.settingsController,
      required this.stateController,
      required this.audioChat, required this.contacts});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Edit Contacts'),
        backgroundColor: Theme.of(context).colorScheme.background,
      ),
      body: Padding(
          padding: const EdgeInsets.all(20),
          child: ListView.builder(
              itemCount: contacts.length,
              itemBuilder: (BuildContext context, int index) {
                Contact contact = contacts[index];

                return EditContactWidget(
                    contact: contact,
                    audioChat: audioChat,
                    settingsController: settingsController,
                    stateController: stateController);
              })),
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

// TODO play incoming sound effect while prompt is open
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
