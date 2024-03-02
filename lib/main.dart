import 'dart:async';
import 'dart:collection';

import 'package:audio_chat/settings/view.dart';
import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:audio_chat/src/rust/api/error.dart';
import 'package:audio_chat/src/rust/api/player.dart';
import 'package:audio_chat/src/rust/frb_generated.dart';
import 'package:audio_chat/settings/controller.dart';
import 'package:debug_console/debug_console.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:mutex/mutex.dart';

final GlobalKey<NavigatorState> navigatorKey = GlobalKey<NavigatorState>();
SoundHandle? outgoingSoundHandle;

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

  final soundPlayer =
      SoundPlayer.newSoundPlayer(outputVolume: settingsController.soundVolume);

  final inPrompt = Mutex();

  final audioChat = await AudioChat.newAudioChat(
      listenPort: settingsController.listenPort,
      receivePort: settingsController.receivePort,
      signingKey: settingsController.signingKey,
      rmsThreshold: settingsController.inputSensitivity,
      inputVolume: settingsController.inputVolume,
      outputVolume: settingsController.outputVolume,
      denoise: settingsController.useDenoise,
      // called when there is an incoming call
      acceptCall: (String id) async {
        bool accepted = false;

        // only allow one prompt at a time
        await inPrompt.protect(() async {
          Contact? contact = settingsController.getContact(id);

          if (callStateController.isCallActive) {
            return false;
          } else if (navigatorKey.currentState == null || !navigatorKey.currentState!.mounted) {
            DebugConsole.warning('navigatorKey.currentState is null');
            return false;
          } else if (contact == null) {
            DebugConsole.warning('contact is null');
            return false;
          }

          accepted = await acceptCallPrompt(
              navigatorKey.currentState!.context, contact, soundPlayer);

          if (accepted) {
            callStateController.setStatus('Connecting');
            callStateController.setActiveContact(contact);
          }
        });

        return accepted;
      },
      // called when a call ends
      callEnded: (message) async {
        outgoingSoundHandle?.cancel();

        callStateController.setActiveContact(null);
        callStateController.setStatus('Inactive');

        List<int> bytes = await readWavBytes('call_ended');
        await soundPlayer.play(bytes: bytes);

        if (message.isNotEmpty &&
            navigatorKey.currentState != null &&
            navigatorKey.currentState!.mounted) {
          showErrorDialog(
              navigatorKey.currentState!.context, 'Call failed', message);
        }
      },
      // called when a contact is needed in the backend
      getContact: (addressStr) {
        Contact? contact = settingsController.contacts.values
            .firstWhere((contact) => contact.addressStr() == addressStr);
        return contact.pubClone();
      },
      // called when the call initially connects
      connected: () async {
        outgoingSoundHandle?.cancel();

        List<int> bytes = await readWavBytes('connected');
        await soundPlayer.play(bytes: bytes);

        callStateController.setStatus('Active');
      },
      // called when the call disconnects or reconnects
      callState: (disconnected) async {
        if (disconnected) {
          List<int> bytes = await readWavBytes('disconnected');
          await soundPlayer.play(bytes: bytes);

          callStateController.setStatus('Reconnecting');
        } else {
          List<int> bytes = await readWavBytes('reconnected');
          await soundPlayer.play(bytes: bytes);

          callStateController.setStatus('Active');
        }
      },
      // called when a contact goes online or offline
      contactStatus: (String id, bool online) {
        if (online) {
          callStateController.addOnlineContact(id);
        } else {
          callStateController.removeOnlineContact(id);
        }
      },
      // called when the backend wants the frontend to start controllers
      startControllers: (AudioChat audioChat) {
        for (Contact contact in settingsController.contacts.values) {
          audioChat.connect(contact: contact);
        }
      });

  runApp(AudioChatApp(
      audioChat: audioChat,
      settingsController: settingsController,
      callStateController: callStateController,
      player: soundPlayer));
}

class AudioChatApp extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController callStateController;
  final SoundPlayer player;

  const AudioChatApp(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.callStateController,
      required this.player});

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
          callStateController: callStateController,
          player: player),
    );
  }
}

class HomePage extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController callStateController;
  final SoundPlayer player;

  const HomePage(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.callStateController,
      required this.player});

  @override
  Widget build(BuildContext context) {
    return DebugConsolePopup(
        child: Scaffold(
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
                              settingsController: settingsController,
                              player: player);
                        }))
              ],
            ),
            const SizedBox(height: 20),
            Expanded(
                child: Row(children: [
              CallControls(
                  audioChat: audioChat,
                  settingsController: settingsController,
                  stateController: callStateController,
                  player: player),
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
                    showErrorDialog(
                        context, 'Validation error', 'All fields are required');
                    return;
                  }

                  Contact contact = await widget.settingsController.addContact(
                      _nicknameInput.text,
                      _verifyingKeyInput.text,
                      _contactAddressInput.text);

                  widget.audioChat.connect(contact: contact);

                  _contactAddressInput.clear();
                  _nicknameInput.clear();
                  _verifyingKeyInput.clear();
                } on DartError catch (e) {
                  if (!context.mounted) return;
                  showErrorDialog(context, 'Failed to add contact', e.message);
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
  final SoundPlayer player;

  const ContactsList(
      {super.key,
      required this.audioChat,
      required this.contacts,
      required this.stateController,
      required this.settingsController,
      required this.player});

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
                                controller: stateController,
                                player: player);
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
  final SoundPlayer player;

  const CallControls(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.stateController,
      required this.player});

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
                    min: -16,
                    max: 130,
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
                    ListenableBuilder(
                        listenable: stateController,
                        builder: (BuildContext context, Widget? child) {
                          return IconButton(
                              onPressed: () async {
                                if (stateController.isDeafened) {
                                  return;
                                }

                                List<int> bytes = stateController.isMuted
                                    ? await readWavBytes('unmute')
                                    : await readWavBytes('mute');
                                player.play(bytes: bytes);

                                stateController.mute();
                                audioChat.setMuted(
                                    muted: stateController.isMuted);
                              },
                              icon: stateController.isMuted |
                                      stateController.isDeafened
                                  ? const Icon(Icons.mic_off)
                                  : const Icon(Icons.mic));
                        }),
                    ListenableBuilder(
                        listenable: stateController,
                        builder: (BuildContext context, Widget? child) {
                          return IconButton(
                              onPressed: () async {
                                List<int> bytes = stateController.isDeafened
                                    ? await readWavBytes('deafen')
                                    : await readWavBytes('undeafen');
                                player.play(bytes: bytes);

                                stateController.deafen();
                                audioChat.setDeafened(
                                    deafened: stateController.isDeafened);

                                if (stateController.isDeafened &&
                                    stateController.isMuted) {
                                  audioChat.setMuted(muted: true);
                                } else {
                                  audioChat.setMuted(muted: false);
                                }
                              },
                              icon: stateController.isDeafened
                                  ? const Icon(Icons.volume_off)
                                  : const Icon(Icons.volume_up));
                        }),
                    IconButton(
                        onPressed: () {
                          Navigator.push(
                              context,
                              MaterialPageRoute(
                                builder: (context) => SettingsPage(
                                    controller: settingsController,
                                    audioChat: audioChat,
                                    callStateController: stateController,
                                    player: player),
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
  final SoundPlayer player;

  const ContactWidget(
      {super.key,
      required this.contact,
      required this.audioChat,
      required this.controller,
      required this.player});

  @override
  Widget build(BuildContext context) {
    bool online = controller.isOnlineContact(contact);

    Widget trailing;

    if (!online) {
      trailing = const Padding(padding: EdgeInsets.symmetric(horizontal: 7), child: Icon(Icons.dark_mode_outlined));
    } else if (online && controller.isActiveContact(contact)) {
      trailing = IconButton(
        icon: const Icon(Icons.call_end, color: Colors.red),
        onPressed: () async {
          outgoingSoundHandle?.cancel();

          audioChat.endCall();
          controller.setActiveContact(null);
          controller.setStatus('Inactive');

          List<int> bytes = await readWavBytes('call_ended');
          await player.play(bytes: bytes);
        },
      );
    } else {
      trailing = IconButton(
        icon: const Icon(Icons.call),
        onPressed: () async {
          if (controller.isCallActive) {
            showErrorDialog(context, 'Call failed',
                'There is a call already active');
            return;
          }

          controller.setStatus('Connecting');
          List<int> bytes = await readWavBytes('outgoing');
          outgoingSoundHandle = await player.play(bytes: bytes);

          try {
            await audioChat.sayHello(contact: contact);
            controller.setActiveContact(contact);
          } on DartError catch (e) {
            controller.setStatus('Inactive');
            outgoingSoundHandle?.cancel();
            if (!context.mounted) return;
            showErrorDialog(context, 'Call failed', e.message);
          }
        },
      );
    }

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
        trailing: trailing,
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
                  DebugConsole.debug(
                      'nickname updated ${widget.contact.nickname()}');
                }),
          ),
          subtitle: TextInput(
              enabled: !widget.stateController.isActiveContact(widget.contact),
              controller: _addressInput,
              labelText: 'Address',
              onChanged: (value) {
                try {
                  widget.settingsController
                      .updateContactAddress(widget.contact, value);

                  // TODO maybe it is not ideal to start and stop the controller on every change
                  widget.audioChat.stopController(contact: widget.contact);
                  widget.audioChat.connect(contact: widget.contact);
                } on DartError catch (_) {
                  return; // ignore
                }
              }),
          trailing: IconButton(
              onPressed: () {
                if (!widget.stateController.isActiveContact(widget.contact)) {
                  widget.settingsController.removeContact(widget.contact);
                  widget.audioChat.stopController(contact: widget.contact);
                } else {
                  showErrorDialog(context, 'Warning',
                      'Cannot delete a contact while in an active call');
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
  bool muted = false;
  final HashSet<String> _onlineContacts = HashSet();

  Contact? get activeContact => _activeContact;
  String get status => _status;
  bool get isCallActive => _activeContact != null;
  bool get isDeafened => deafened;
  bool get isMuted => muted;

  void setActiveContact(Contact? contact) {
    _activeContact = contact;
    notifyListeners();
  }

  void setStatus(String status) {
    _status = status;
    notifyListeners();
  }

  bool isActiveContact(Contact contact) {
    return _activeContact?.id() == contact.id();
  }

  bool isOnlineContact(Contact contact) {
    return _onlineContacts.contains(contact.id());
  }

  void addOnlineContact(String id) {
    _onlineContacts.add(id);
    notifyListeners();
  }

  void removeOnlineContact(String id) {
    _onlineContacts.remove(id);
    notifyListeners();
  }

  void deafen() {
    deafened = !deafened;
    muted = deafened;
    notifyListeners();
  }

  void mute() {
    muted = !muted;
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
      required this.audioChat,
      required this.contacts});

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
void showErrorDialog(BuildContext context, String title, String errorMessage) {
  showDialog(
    context: context,
    builder: (BuildContext context) {
      return AlertDialog(
        title: Text(title),
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

Future<bool> acceptCallPrompt(
    BuildContext context, Contact contact, SoundPlayer player) async {
  const timeout = Duration(seconds: 10);
  List<int> bytes = await readWavBytes('incoming');
  SoundHandle handle = await player.play(bytes: bytes);

  if (!context.mounted) {
    return false;
  }

  bool? result = await showDialog<bool>(
    context: context,
    barrierDismissible: false,
    builder: (BuildContext context) {
      Timer(timeout, () {
        if (context.mounted) {
          handle.cancel();
          Navigator.of(context).pop(false);
        }
      });

      return AlertDialog(
        title: Text('Accept call from ${contact.nickname()}?'),
        actions: <Widget>[
          TextButton(
            child: const Text('Deny'),
            onPressed: () {
              handle.cancel();
              Navigator.of(context).pop(false);
            },
          ),
          TextButton(
            child: const Text('Accept'),
            onPressed: () {
              handle.cancel();
              Navigator.of(context).pop(true);
            },
          ),
        ],
      );
    },
  );

  handle.cancel();
  return result ?? false;
}

Future<List<int>> readWavBytes(String assetName) async {
  // Load the asset bytes
  final ByteData data = await rootBundle.load('assets/sounds/$assetName.wav');
  // Convert ByteData to Uint8List
  final List<int> bytes = data.buffer.asUint8List();
  return bytes;
}
