import 'dart:async';
import 'dart:collection';
import 'dart:io';

import 'package:audio_chat/settings/view.dart';
import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:audio_chat/src/rust/api/contact.dart';
import 'package:audio_chat/src/rust/api/error.dart';
import 'package:audio_chat/src/rust/api/logger.dart';
import 'package:audio_chat/src/rust/api/player.dart';
import 'package:audio_chat/src/rust/frb_generated.dart';
import 'package:audio_chat/settings/controller.dart';
import 'package:debug_console/debug_console.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:mutex/mutex.dart';

final GlobalKey<NavigatorState> navigatorKey = GlobalKey<NavigatorState>();
SoundHandle? outgoingSoundHandle;

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();

  // get logs from rust
  rustSetUp();
  createLogStream().listen((message) {
    DebugConsole.log(message);
  });

  if (Platform.isAndroid) {
    PermissionStatus status = await Permission.microphone.request();

    if (!status.isGranted) {
      DebugConsole.error('Microphone permission not accepted');
    }
  }

  const storage = FlutterSecureStorage();
  final SharedPreferences options = await SharedPreferences.getInstance();

  final SettingsController settingsController =
      SettingsController(storage: storage, options: options);
  await settingsController.init();

  final StateController callStateController = StateController();

  final soundPlayer = SoundPlayer(outputVolume: settingsController.soundVolume);
  var host = soundPlayer.host();

  final messageBus = MessageBus();

  final inPrompt = Mutex();

  final audioChat = await AudioChat.newInstance(
      signingKey: settingsController.signingKey,
      host: host,
      // called when there is an incoming call
      acceptCall: (String id, ringtone) async {
        bool accepted = false;

        // only allow one prompt at a time
        await inPrompt.protect(() async {
          Contact? contact = settingsController.getContact(id);

          if (callStateController.isCallActive) {
            return false;
          } else if (contact == null) {
            DebugConsole.warning('contact is null');
            return false;
          }

          List<int> bytes;

          if (ringtone == null) {
            bytes = await readWavBytes('incoming');
          } else {
            bytes = ringtone;
          }

          SoundHandle handle = await soundPlayer.play(bytes: bytes);

          if (navigatorKey.currentState == null ||
              !navigatorKey.currentState!.mounted) return false;

          accepted = await acceptCallPrompt(
              navigatorKey.currentState!.context, contact);
          handle.cancel();

          if (accepted) {
            callStateController.setStatus('Connecting');
            callStateController.setActiveContact(contact);
          }
        });

        return accepted;
      },
      // called when a call ends
      callEnded: (String message, bool remote) async {
        if (!callStateController.isCallActive) {
          DebugConsole.warning(
              "call ended entered but there is no active call");
          return;
        }

        outgoingSoundHandle?.cancel();

        callStateController.setActiveContact(null);
        callStateController.setStatus('Inactive');
        callStateController.disableCallsTemporarily();

        List<int> bytes = await readWavBytes('call_ended');
        await soundPlayer.play(bytes: bytes);

        if (message.isNotEmpty &&
            navigatorKey.currentState != null &&
            navigatorKey.currentState!.mounted) {
          showErrorDialog(navigatorKey.currentState!.context,
              remote ? 'Call failed (remote)' : 'Call failed', message);
        }
      },
      // called when a contact is needed in the backend
      getContact: (verifyingKey) {
        Contact? contact = settingsController.contacts.values
            .firstWhere((Contact contact) => contact.keyEq(key: verifyingKey));
        return contact?.pubClone();
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
      // called when the backend wants to start sessions
      startSessions: (AudioChat audioChat) {
        for (Contact contact in settingsController.contacts.values) {
          audioChat.startSession(contact: contact);
        }
      },
      // TODO this callback could potentially be merged with the statisticsCallback
      // called during calls after a latency test
      callLatency: (latency) {
        callStateController.setLatency(latency);
      },
      // called when the backend wants a custom ringtone
      loadRingtone: () async {
        if (settingsController.customRingtoneFile == null) {
          return null;
        } else {
          return await File(settingsController.customRingtoneFile!)
              .readAsBytes();
        }
      },
      // called when the backend has updated statistics
      statistics: (Statistics statistics) {
        callStateController.setRms(statistics.rms);
      },
      // called when a new chat message is received by the backend
      messageReceived: messageBus.sendMessage,
      managerActive: (bool active, bool restartable) {
        callStateController.setSessionManager(active, restartable);
      });

  // apply options to the audio chat instance
  audioChat.setRmsThreshold(decimal: settingsController.inputSensitivity);
  audioChat.setInputVolume(decibel: settingsController.inputVolume);
  audioChat.setOutputVolume(decibel: settingsController.outputVolume);
  audioChat.setDenoise(denoise: settingsController.useDenoise);
  audioChat.setPlayCustomRingtones(
      play: settingsController.playCustomRingtones);
  audioChat.setInputDevice(device: settingsController.inputDevice);
  audioChat.setOutputDevice(device: settingsController.outputDevice);

  runApp(AudioChatApp(
      audioChat: audioChat,
      settingsController: settingsController,
      callStateController: callStateController,
      player: soundPlayer,
      messageBus: messageBus));
}

/// The main app
class AudioChatApp extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController callStateController;
  final SoundPlayer player;
  final MessageBus messageBus;

  const AudioChatApp(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.callStateController,
      required this.player,
      required this.messageBus});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      navigatorKey: navigatorKey,
      theme: ThemeData(
        dialogTheme: const DialogTheme(
          surfaceTintColor: Color(0xFF27292A),
        ),
        sliderTheme: SliderThemeData(
          showValueIndicator: ShowValueIndicator.always,
          overlayColor: Colors.transparent,
          trackShape: CustomTrackShape(),
          activeTrackColor: const Color(0xFFdb5c5c),
        ),
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFFFD6D6D),
          secondary: Color(0xFFdb5c5c),
          brightness: Brightness.dark,
          background: Color(0xFF222425),
          secondaryContainer: Color(0xFF191919),
          tertiaryContainer: Color(0xFF27292A),
        ),
        switchTheme: SwitchThemeData(
          trackOutlineWidth: MaterialStateProperty.all(0),
          trackOutlineColor: MaterialStateProperty.all(Colors.transparent),
          overlayColor: MaterialStateProperty.all(Colors.transparent),
          thumbColor:
              MaterialStateProperty.all(Theme.of(context).indicatorColor),
        ),
        dropdownMenuTheme: DropdownMenuThemeData(
          menuStyle: MenuStyle(
            backgroundColor: MaterialStateProperty.all(const Color(0xFF191919)),
            surfaceTintColor:
                MaterialStateProperty.all(const Color(0xFF191919)),
          ),
        ),
      ),
      home: HomePage(
          audioChat: audioChat,
          settingsController: settingsController,
          callStateController: callStateController,
          player: player,
          messageBus: messageBus),
    );
  }
}

/// The main body of the app
class HomePage extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController callStateController;
  final SoundPlayer player;
  final MessageBus messageBus;

  const HomePage(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.callStateController,
      required this.player,
      required this.messageBus});

  @override
  Widget build(BuildContext context) {
    ContactForm contactForm = ContactForm(
        audioChat: audioChat, settingsController: settingsController);

    ListenableBuilder contactsList = ListenableBuilder(
        listenable: settingsController,
        builder: (BuildContext context, Widget? child) {
          return ContactsList(
              audioChat: audioChat,
              contacts: settingsController.contacts.values.toList(),
              stateController: callStateController,
              settingsController: settingsController,
              player: player);
        });

    CallControls callControls = CallControls(
        audioChat: audioChat,
        settingsController: settingsController,
        stateController: callStateController,
        player: player);

    return DebugConsolePopup(
        child: Scaffold(
      body: Padding(
          padding: const EdgeInsets.all(20.0),
          child: LayoutBuilder(
              builder: (BuildContext context, BoxConstraints constraints) {
            if (constraints.maxWidth > 600) {
              return Column(
                children: [
                  Container(
                    constraints: BoxConstraints(
                        maxHeight: 250, maxWidth: constraints.maxWidth),
                    child: Row(mainAxisSize: MainAxisSize.min, children: [
                      Container(
                        constraints: const BoxConstraints(maxWidth: 300),
                        child: contactForm,
                      ),
                      const SizedBox(width: 20),
                      Flexible(fit: FlexFit.loose, child: contactsList)
                    ]),
                  ),
                  const SizedBox(height: 20),
                  Flexible(
                      fit: FlexFit.loose,
                      child: Row(children: [
                        Flexible(fit: FlexFit.loose, child: callControls),
                        const SizedBox(width: 20),
                        // Expanded(
                        //     child: ChatWidget(
                        //         audioChat: audioChat,
                        //         stateController: callStateController,
                        //         messageBus: messageBus))
                      ])),
                ],
              );
            } else {
              return Column(children: [
                Container(
                  constraints: BoxConstraints(
                      maxHeight: 250, maxWidth: constraints.maxWidth),
                  child: contactsList,
                ),
                const SizedBox(height: 20),
                Flexible(fit: FlexFit.loose, child: callControls),
              ]);
            }
          })),
    ));
  }
}

/// A widget which allows the user to add a contact
class ContactForm extends StatefulWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;

  const ContactForm(
      {super.key, required this.audioChat, required this.settingsController});

  @override
  State<ContactForm> createState() => _ContactFormState();
}

/// The state for ContactForm
class _ContactFormState extends State<ContactForm> {
  final TextEditingController _nicknameInput = TextEditingController();
  final TextEditingController _verifyingKeyInput = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 15.0, horizontal: 20.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text("Add Contact", style: TextStyle(fontSize: 20)),
          const SizedBox(height: 21),
          TextInput(controller: _nicknameInput, labelText: 'Nickname'),
          const SizedBox(height: 15),
          TextInput(
              controller: _verifyingKeyInput,
              labelText: 'Verifying Key',
              hintText: 'base64 encoded verifying key',
              obscureText: true),
          const SizedBox(height: 26),
          Center(
            child: Button(
              text: 'Add Contact',
              disabled: false,
              onPressed: () async {
                if (_nicknameInput.text.isEmpty ||
                    _verifyingKeyInput.text.isEmpty) {
                  showErrorDialog(context, 'Failed to add contact',
                      'Nickname and verifying key cannot be empty');
                  return;
                }

                try {
                  Contact contact = await widget.settingsController
                      .addContact(_nicknameInput.text, _verifyingKeyInput.text);

                  widget.audioChat.startSession(contact: contact);

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

/// A widget which displays a list of ContactWidget's
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
    return Container(
      padding: const EdgeInsets.only(
          bottom: 15.0, left: 12.0, right: 12.0, top: 8.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
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
          Flexible(
            fit: FlexFit.loose,
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
  }
}

/// A widget which displays a single contact
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
    bool active = controller.isActiveContact(contact);

    Widget trailing;

    if (!online) {
      trailing = const Padding(
          padding: EdgeInsets.symmetric(horizontal: 7),
          child: Icon(Icons.dark_mode_outlined));
    } else if (active) {
      trailing = IconButton(
        icon: const Icon(Icons.call_end, color: Colors.red),
        onPressed: () async {
          outgoingSoundHandle?.cancel();

          audioChat.endCall();
          controller.setActiveContact(null);
          controller.setStatus('Inactive');
          controller.disableCallsTemporarily();

          List<int> bytes = await readWavBytes('call_ended');
          await player.play(bytes: bytes);
        },
      );
    } else {
      trailing = IconButton(
        icon: const Icon(Icons.call),
        onPressed: () async {
          if (controller.isCallActive) {
            showErrorDialog(
                context, 'Call failed', 'There is a call already active');
            return;
          } else if (controller.inAudioTest) {
            showErrorDialog(context, 'Call failed',
                'Cannot make a call while in an audio test');
            return;
          } else if (controller.callEndedRecently) {
            // if the call button is pressed right after a call ended, we assume the user did not want to make a call
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
        trailing: trailing,
      ),
    );
  }
}

/// A widget with commonly used controls for a call
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
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.tertiaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Column(
        children: [
          const SizedBox(height: 10),
          ListenableBuilder(
              listenable: stateController,
              builder: (BuildContext context, Widget? child) {
                Widget body;

                if (stateController.sessionManagerActive) {
                  body = Text(
                      '${stateController.status}${stateController.latency}',
                      style: const TextStyle(fontSize: 20));
                } else {
                  body = Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const SizedBox(width: 15),
                      const Text('Session Manager Inactive',
                          style: TextStyle(fontSize: 16, color: Colors.red)),
                      stateController.sessionManagerRestartable
                          ? const Spacer()
                          : const SizedBox(width: 10),
                      stateController.sessionManagerRestartable
                          ? IconButton(
                              onPressed: () {
                                audioChat.restartManager();
                              },
                              icon: const Icon(Icons.restart_alt,
                                  color: Colors.red))
                          : Container(),
                      const SizedBox(width: 5),
                    ],
                  );
                }

                return SizedBox(
                  height: 40,
                  child: Center(child: body),
                );
              }),
          Padding(
            padding: const EdgeInsets.only(left: 25, right: 25, top: 20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('Output Volume', style: TextStyle(fontSize: 15)),
                ListenableBuilder(
                    listenable: settingsController,
                    builder: (BuildContext context, Widget? child) {
                      return Slider(
                          value: settingsController.outputVolume,
                          onChanged: (value) async {
                            await settingsController.updateOutputVolume(value);
                            audioChat.setOutputVolume(decibel: value);
                          },
                          min: -15,
                          max: 15,
                          label:
                              '${settingsController.outputVolume.toStringAsFixed(2)} db');
                    }),
                const SizedBox(height: 2),
                const Text('Input Volume', style: TextStyle(fontSize: 15)),
                ListenableBuilder(
                    listenable: settingsController,
                    builder: (BuildContext context, Widget? child) {
                      return Slider(
                          value: settingsController.inputVolume,
                          onChanged: (value) async {
                            await settingsController.updateInputVolume(value);
                            audioChat.setInputVolume(decibel: value);
                          },
                          min: -15,
                          max: 15,
                          label:
                              '${settingsController.inputVolume.toStringAsFixed(2)} db');
                    }),
                const SizedBox(height: 2),
                const Text('Input Sensitivity', style: TextStyle(fontSize: 15)),
                ListenableBuilder(
                    listenable: settingsController,
                    builder: (BuildContext context, Widget? child) {
                      return Slider(
                          value: settingsController.inputSensitivity,
                          onChanged: (value) async {
                            await settingsController
                                .updateInputSensitivity(value);
                            audioChat.setRmsThreshold(decimal: value);
                          },
                          min: -16,
                          max: 50,
                          label:
                              '${settingsController.inputSensitivity.toStringAsFixed(2)} db');
                    }),
              ],
            ),
          ),
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

/// A widget which contains a EditContactWidget for each contact
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
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Flexible(
                  fit: FlexFit.loose,
                  child: ListenableBuilder(
                      listenable: settingsController,
                      builder: (BuildContext context, Widget? child) {
                        return ListView.builder(
                            itemCount: contacts.length,
                            itemBuilder: (BuildContext context, int index) {
                              Contact contact = contacts[index];

                              return EditContactWidget(
                                  contact: contact,
                                  audioChat: audioChat,
                                  settingsController: settingsController,
                                  stateController: stateController);
                            });
                      })),
              ContactForm(
                  audioChat: audioChat, settingsController: settingsController)
            ],
          )),
    );
  }
}

/// A widget which allows the user to edit a contact
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

/// The state for EditContactWidget
class _EditContactWidgetState extends State<EditContactWidget> {
  late final TextEditingController _nicknameInput;

  @override
  void initState() {
    super.initState();
    _nicknameInput = TextEditingController(text: widget.contact.nickname());
  }

  @override
  void dispose() {
    _nicknameInput.dispose();
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
          trailing: IconButton(
              onPressed: () {
                if (!widget.stateController.isActiveContact(widget.contact)) {
                  widget.settingsController.removeContact(widget.contact);
                  widget.audioChat.stopSession(contact: widget.contact);
                } else {
                  showErrorDialog(context, 'Warning',
                      'Cannot delete a contact while in an active call');
                }
              },
              icon: const Icon(Icons.delete))),
    );
  }
}

class ChatWidget extends StatefulWidget {
  final AudioChat audioChat;
  final StateController stateController;
  final MessageBus messageBus;

  const ChatWidget(
      {super.key,
      required this.audioChat,
      required this.stateController,
      required this.messageBus});

  @override
  State<StatefulWidget> createState() => _ChatWidgetState();
}

// TODO switch from String to Chat (rust)
// TODO differentiate between sent and received messages
// TODO a halfway decent UI
class _ChatWidgetState extends State<ChatWidget> {
  late TextEditingController _messageInput;
  List<String> messages = [];
  late StreamSubscription<String> messageSubscription;
  bool active = false;

  @override
  void initState() {
    super.initState();

    messageSubscription = widget.messageBus.messageStream.listen((message) {
      DebugConsole.debug('messageSubscription got $message | active=$active');

      if (active) {
        setState(() {
          messages.add(message);
        });
      }
    });

    _messageInput = TextEditingController();
  }

  @override
  void dispose() {
    messageSubscription.cancel();
    _messageInput.dispose();
    super.dispose();
  }

  void sendMessage(String message) {
    if (!active) return;

    String id = widget.stateController.activeContact!.id();

    try {
      widget.audioChat.sendChat(message: message, id: id);

      setState(() {
        messages.add(message);
        _messageInput.clear();
      });
    } on DartError catch (error) {
      showErrorDialog(context, 'Message Send Failed', error.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Column(
        children: [
          Expanded(
              child: ListView.builder(
                  itemCount: messages.length,
                  itemBuilder: (BuildContext context, int index) {
                    DebugConsole.debug('chat widget got $index');
                    String message = messages[index];
                    return ListTile(
                      title: Text(message),
                    );
                  })),
          ListenableBuilder(
              listenable: widget.stateController,
              builder: (BuildContext context, Widget? child) {
                if (!widget.stateController.isCallActive && active) {
                  DebugConsole.debug('chat widget is inactive');
                  setState(() {
                    messages.clear();
                    _messageInput.clear();
                    active = false;
                  });
                } else if (widget.stateController.isCallActive && !active) {
                  DebugConsole.debug('chat widget is active');
                  setState(() {
                    active = true;
                  });
                }

                return Padding(
                    padding:
                        const EdgeInsets.only(bottom: 10, left: 20, right: 20),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        TextInput(
                          controller: _messageInput,
                          labelText: widget.stateController.isCallActive
                              ? 'Message'
                              : 'Chat disabled',
                          enabled: widget.stateController.isCallActive,
                          onSubmitted: (message) {
                            if (message.isEmpty) return;
                            sendMessage(message);
                          },
                        ),
                        IconButton(
                          onPressed: () {
                            String message = _messageInput.text;
                            if (message.isEmpty) return;
                            sendMessage(message);
                          },
                          icon: const Icon(Icons.send),
                        )
                      ],
                    ));
              }),
        ],
      ),
    );
  }
}

/// Custom Button Widget
class Button extends StatelessWidget {
  final String text;
  final VoidCallback onPressed;
  final double? width;
  final double? height;
  final bool disabled;

  const Button(
      {super.key,
      required this.text,
      required this.onPressed,
      this.width,
      this.height,
      required this.disabled});

  @override
  Widget build(BuildContext context) {
    Widget child;

    if (width == null || height == null) {
      child = Text(text);
    } else {
      child = SizedBox(
        width: width!,
        height: height,
        child: Center(child: Text(text)),
      );
    }

    return ElevatedButton(
      onPressed: () {
        if (!disabled) {
          onPressed();
        }
      },
      style: ButtonStyle(
        backgroundColor: disabled
            ? MaterialStateProperty.all(Colors.grey)
            : MaterialStateProperty.all(Theme.of(context).colorScheme.primary),
        foregroundColor: MaterialStateProperty.all(Colors.white),
        overlayColor: disabled
            ? MaterialStateProperty.all(Colors.grey)
            : MaterialStateProperty.all(
                Theme.of(context).colorScheme.secondary),
        surfaceTintColor: MaterialStateProperty.all(Colors.transparent),
        mouseCursor: disabled
            ? MaterialStateProperty.all(SystemMouseCursors.basic)
            : MaterialStateProperty.all(SystemMouseCursors.click),
      ),
      child: child,
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
  final void Function(String)? onSubmitted;

  const TextInput(
      {super.key,
      required this.labelText,
      this.hintText,
      required this.controller,
      this.obscureText,
      this.enabled,
      this.onChanged,
      this.onSubmitted});

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      obscureText: obscureText ?? false,
      enabled: enabled,
      onChanged: onChanged,
      onSubmitted: onSubmitted,
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

/// Custom Switch widget
class CustomSwitch extends StatelessWidget {
  final bool value;
  final bool? disabled;
  final void Function(bool)? onChanged;

  const CustomSwitch(
      {super.key, required this.value, required this.onChanged, this.disabled});

  @override
  Widget build(BuildContext context) {
    return Transform.scale(
      scale: 0.85,
      child: Switch(
        value: value,
        onChanged: disabled == true ? null : onChanged,
        inactiveTrackColor: const Color(0xFF80848e),
        activeTrackColor: disabled == true
            ? const Color(0xFF80848e)
            : Theme.of(context).colorScheme.secondary,
      ),
    );
  }
}

/// A controller which helps bridge the gap between the UI and backend
class StateController extends ChangeNotifier {
  Contact? _activeContact;
  String status = 'Inactive';
  bool _deafened = false;
  bool _muted = false;
  bool inAudioTest = false;
  final HashSet<String> _onlineContacts = HashSet();
  int? _latency;
  bool _callEndedRecently = false;
  double rms = 0;

  /// active, restartable
  (bool, bool) _sessionManager = (false, false);

  Contact? get activeContact => _activeContact;
  bool get isCallActive => _activeContact != null;
  bool get isDeafened => _deafened;
  bool get isMuted => _muted;
  String get latency => _latency == null ? '' : ' $_latency ms';
  bool get callEndedRecently => _callEndedRecently;
  bool get blockAudioChanges => isCallActive || inAudioTest;
  bool get sessionManagerActive => _sessionManager.$1;
  bool get sessionManagerRestartable => _sessionManager.$2;

  void setActiveContact(Contact? contact) {
    _activeContact = contact;

    if (contact == null) {
      _latency = null;
      rms = 0; // reset rms
    }

    notifyListeners();
  }

  void setStatus(String status) {
    this.status = status;
    notifyListeners();
  }

  void setSessionManager(bool active, bool restartable) {
    _sessionManager = (active, restartable);
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
    _deafened = !_deafened;
    _muted = _deafened;
    notifyListeners();
  }

  void mute() {
    _muted = !_muted;
    notifyListeners();
  }

  void setInAudioTest() {
    inAudioTest = !inAudioTest;
    status = inAudioTest ? 'In Audio Test' : 'Inactive';
    rms = 0; // reset rms

    notifyListeners();
  }

  void setLatency(int latency) {
    _latency = latency;
    notifyListeners();
  }

  void disableCallsTemporarily() {
    _callEndedRecently = true;

    Timer(const Duration(seconds: 1), () {
      _callEndedRecently = false;
    });
  }

  void setRms(double rms) {
    // currently the rms is only used when in an audio test
    if (inAudioTest) {
      this.rms = rms;
      notifyListeners();
    }
  }
}

/// Removes the padding from a Slider
class CustomTrackShape extends RoundedRectSliderTrackShape {
  @override
  Rect getPreferredRect({
    required RenderBox parentBox,
    Offset offset = Offset.zero,
    required SliderThemeData sliderTheme,
    bool isEnabled = false,
    bool isDiscrete = false,
  }) {
    final trackHeight = sliderTheme.trackHeight;
    final trackLeft = offset.dx;
    final trackTop = offset.dy + (parentBox.size.height - trackHeight!) / 2;
    final trackWidth = parentBox.size.width;
    return Rect.fromLTWH(trackLeft, trackTop, trackWidth, trackHeight);
  }
}

/// Sends messages from the backend callback to the chat widget
class MessageBus {
  final _messageController = StreamController<String>.broadcast();

  // for widgets to listen to the stream
  Stream<String> get messageStream => _messageController.stream;

  // called to send a message
  void sendMessage(String message) {
    _messageController.sink.add(message);
  }

  // close the stream controller when it's no longer needed
  void dispose() {
    _messageController.close();
  }
}

/// Shows an error modal
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

/// Prompts the user to accept an incoming call
Future<bool> acceptCallPrompt(BuildContext context, Contact contact) async {
  const timeout = Duration(seconds: 10);

  if (!context.mounted) {
    return false;
  }

  bool? result = await showDialog<bool>(
    context: context,
    barrierDismissible: false,
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
            onPressed: () {
              Navigator.of(context).pop(false);
            },
          ),
          TextButton(
            child: const Text('Accept'),
            onPressed: () {
              Navigator.of(context).pop(true);
            },
          ),
        ],
      );
    },
  );

  return result ?? false;
}

/// Reads the bytes of a wav file from the assets
Future<List<int>> readWavBytes(String assetName) async {
  // Load the asset bytes
  final ByteData data = await rootBundle.load('assets/sounds/$assetName.wav');
  // Convert ByteData to Uint8List
  final List<int> bytes = data.buffer.asUint8List();
  return bytes;
}
