import 'dart:async';
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

  final StateController stateController = StateController();
  final StatisticsController statisticsController = StatisticsController();

  final soundPlayer = SoundPlayer(outputVolume: settingsController.soundVolume);
  soundPlayer.updateOutputDevice(name: settingsController.outputDevice);
  soundPlayer.updateOutputVolume(volume: settingsController.outputVolume);

  var host = soundPlayer.host();

  final messageBus = MessageBus();

  final audioChat = await AudioChat.newInstance(
      identity: settingsController.keypair,
      host: host,
      networkConfig: settingsController.networkConfig,
      // called when there is an incoming call
      acceptCall: (String id, Uint8List? ringtone, DartNotify cancel) async {
        Contact? contact = settingsController.getContact(id);

        if (stateController.isCallActive) {
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

        Future acceptedFuture =
            acceptCallPrompt(navigatorKey.currentState!.context, contact);
        Future cancelFuture = cancel.notified();

        final result = await Future.any([acceptedFuture, cancelFuture]);

        handle.cancel();

        if (result == null) {
          DebugConsole.debug('cancelled');

          if (navigatorKey.currentState != null &&
              navigatorKey.currentState!.mounted) {
            Navigator.pop(navigatorKey.currentState!.context);
          }

          return false; // cancelled
        } else if (result) {
          stateController.setStatus('Connecting');
          stateController.setActiveContact(contact);
        }

        return result;
      },
      // called when a call ends
      callEnded: (String message, bool remote) async {
        if (!stateController.isCallActive) {
          DebugConsole.warning(
              "call ended entered but there is no active call");
          return;
        }

        outgoingSoundHandle?.cancel();

        stateController.setActiveContact(null);
        stateController.setStatus('Inactive');
        stateController.disableCallsTemporarily();

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
      getContact: (Uint8List peerId) {
        try {
          Contact? contact = settingsController.contacts.values
              .firstWhere((Contact contact) => contact.idEq(id: peerId));
          return contact?.pubClone();
        } catch (_) {
          return null;
        }
      },
      // called when the call initially connects
      connected: () async {
        outgoingSoundHandle?.cancel();

        List<int> bytes = await readWavBytes('connected');
        await soundPlayer.play(bytes: bytes);

        stateController.setStatus('Active');
      },
      // called when the call disconnects or reconnects
      callState: (bool disconnected) async {
        if (disconnected && stateController.isCallActive) {
          List<int> bytes = await readWavBytes('disconnected');
          await soundPlayer.play(bytes: bytes);

          stateController.setStatus('Reconnecting');
        } else if (!disconnected && stateController.isCallActive) {
          List<int> bytes = await readWavBytes('reconnected');
          await soundPlayer.play(bytes: bytes);

          stateController.setStatus('Active');
        } else {
          DebugConsole.warning('callState entered but there is no active call');
        }
      },
      // called when a session changes status
      sessionStatus: stateController.updateSession,
      // called when the backend wants to start sessions
      startSessions: (AudioChat audioChat) {
        for (Contact contact in settingsController.contacts.values) {
          audioChat.startSession(contact: contact);
        }
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
      statistics: statisticsController.setStatistics,
      // called when a new chat message is received by the backend
      messageReceived: messageBus.sendMessage,
      // called when the session manager state changes
      managerActive: (bool active, bool restartable) {
        stateController.setSessionManager(active, restartable);
      },
      // called when the backend is starting a call on its own
      callStarted: (Contact contact) async {
        stateController.setStatus('Connecting');
        List<int> bytes = await readWavBytes('outgoing');
        outgoingSoundHandle = await soundPlayer.play(bytes: bytes);
        stateController.setActiveContact(contact);
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

  if (settingsController.denoiseModel != null) {
    updateDenoiseModel(settingsController.denoiseModel!, audioChat);
  }

  runApp(AudioChatApp(
      audioChat: audioChat,
      settingsController: settingsController,
      callStateController: stateController,
      player: soundPlayer,
      messageBus: messageBus,
      statisticsController: statisticsController));
}

/// The main app
class AudioChatApp extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController callStateController;
  final StatisticsController statisticsController;
  final SoundPlayer player;
  final MessageBus messageBus;

  const AudioChatApp(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.callStateController,
      required this.player,
      required this.messageBus,
      required this.statisticsController});

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
          stateController: callStateController,
          player: player,
          messageBus: messageBus,
          statisticsController: statisticsController),
    );
  }
}

/// The main body of the app
class HomePage extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController stateController;
  final StatisticsController statisticsController;
  final SoundPlayer player;
  final MessageBus messageBus;

  const HomePage(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.stateController,
      required this.player,
      required this.messageBus,
      required this.statisticsController});

  @override
  Widget build(BuildContext context) {
    ListenableBuilder contactForm = ListenableBuilder(
        listenable: stateController,
        builder: (BuildContext context, Widget? child) {
          if (stateController.isCallActive) {
            return CallDetailsWidget(
                statisticsController: statisticsController,
                stateController: stateController);
          } else {
            return ContactForm(
                audioChat: audioChat, settingsController: settingsController);
          }
        });

    ListenableBuilder contactsList = ListenableBuilder(
        listenable: settingsController,
        builder: (BuildContext context, Widget? child) {
          return ListenableBuilder(
              listenable: stateController,
              builder: (BuildContext context, Widget? child) {
                List<Contact> contacts =
                    settingsController.contacts.values.toList();

                // sort contacts by session status then nickname
                contacts.sort((a, b) {
                  String aStatus = stateController.sessionStatus(a);
                  String bStatus = stateController.sessionStatus(b);

                  if (aStatus == bStatus) {
                    return a.nickname().compareTo(b.nickname());
                  } else if (aStatus == 'Connected') {
                    return -1;
                  } else if (bStatus == 'Connected') {
                    return 1;
                  } else if (aStatus == 'Connecting') {
                    return -1;
                  } else if (bStatus == 'Connecting') {
                    return 1;
                  } else {
                    return 0;
                  }
                });

                return ContactsList(
                    audioChat: audioChat,
                    contacts: contacts,
                    stateController: stateController,
                    settingsController: settingsController,
                    player: player);
              });
        });

    PeriodicNotifier notifier = PeriodicNotifier();

    CallControls callControls = CallControls(
        audioChat: audioChat,
        settingsController: settingsController,
        stateController: stateController,
        statisticsController: statisticsController,
        player: player,
        notifier: notifier);

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
                    constraints: const BoxConstraints(maxHeight: 250),
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
                      child: Row(mainAxisSize: MainAxisSize.min, children: [
                        Container(
                            constraints: const BoxConstraints(maxWidth: 260),
                            child: callControls),
                        const SizedBox(width: 20),
                        Flexible(
                            fit: FlexFit.loose,
                            child: ChatWidget(
                                audioChat: audioChat,
                                stateController: stateController,
                                messageBus: messageBus,
                                player: player))
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
  State<ContactForm> createState() => ContactFormState();
}

/// The state for ContactForm
class ContactFormState extends State<ContactForm> {
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

/// A widget which displays a list of ContactWidgets
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
    String status = controller.sessions[contact.peerId()] ?? 'Unknown';

    List<Widget> widgets = [
      const CircleAvatar(
        maxRadius: 17,
        child: Icon(Icons.person),
      ),
      const SizedBox(width: 10),
      Text(contact.nickname(), style: const TextStyle(fontSize: 16)),
      const Spacer(),
    ];

    if (status == 'Inactive') {
      widgets.add(IconButton(
          onPressed: () {
            audioChat.startSession(contact: contact);
          },
          icon: const Icon(Icons.refresh)));
      widgets.add(const SizedBox(width: 4));
    } else if (status == 'Connecting') {
      widgets.add(const SizedBox(
          width: 20,
          height: 20,
          child: CircularProgressIndicator(strokeWidth: 3)));
      widgets.add(const SizedBox(width: 10));
    }

    if (!online) {
      widgets.add(const Padding(
          padding: EdgeInsets.symmetric(horizontal: 7, vertical: 8),
          child: Icon(Icons.dark_mode_outlined)));
    } else if (active) {
      widgets.add(IconButton(
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
      ));
    } else {
      widgets.add(IconButton(
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
      ));
    }

    return Container(
      margin: const EdgeInsets.all(5.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6.5),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: widgets,
      ),
    );
  }
}

/// A widget with commonly used controls for a call
class CallControls extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController stateController;
  final StatisticsController statisticsController;
  final SoundPlayer player;
  final PeriodicNotifier notifier;

  const CallControls(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.stateController,
      required this.player,
      required this.statisticsController,
      required this.notifier});

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
                  if (stateController.isCallActive) {
                    body = ListenableBuilder(
                        listenable: notifier,
                        builder: (BuildContext context, Widget? child) {
                          return Text(stateController.callDuration,
                              style: const TextStyle(fontSize: 20));
                        });
                  } else {
                    body = Text(stateController.status,
                        style: const TextStyle(fontSize: 20));
                  }
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
                                    stateController: stateController,
                                    statisticsController: statisticsController,
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
  State<StatefulWidget> createState() => EditContactWidgetState();
}

/// The state for EditContactWidget
class EditContactWidgetState extends State<EditContactWidget> {
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
  final SoundPlayer player;

  const ChatWidget(
      {super.key,
      required this.audioChat,
      required this.stateController,
      required this.messageBus,
      required this.player});

  @override
  State<StatefulWidget> createState() => ChatWidgetState();
}

// TODO switch from String to Chat (rust)
// TODO differentiate between sent and received messages
// TODO a halfway decent UI
class ChatWidgetState extends State<ChatWidget> {
  late TextEditingController _messageInput;
  List<String> messages = [];
  late StreamSubscription<String> messageSubscription;
  bool active = false;

  @override
  void initState() {
    super.initState();
    messageSubscription =
        widget.messageBus.messageStream.listen(receivedMessage);
    _messageInput = TextEditingController();
  }

  @override
  void dispose() {
    messageSubscription.cancel();
    _messageInput.dispose();
    super.dispose();
  }

  // TODO add a sound effect here too
  void sendMessage(String message) {
    if (!active) return;

    Contact contact = widget.stateController.activeContact!;

    try {
      widget.audioChat.sendChat(message: message, contact: contact);

      setState(() {
        messages.add(message);
        _messageInput.clear();
      });
    } on DartError catch (error) {
      showErrorDialog(context, 'Message Send Failed', error.message);
    }
  }

  void receivedMessage(String message) async {
    if (!active) return;

    // TODO add the sound effect
    // widget.player.play(bytes: await readWavBytes('message'));

    setState(() {
      messages.add(message);
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Flexible(
              fit: FlexFit.loose,
              child: ListView.builder(
                  itemCount: messages.length,
                  itemBuilder: (BuildContext context, int index) {
                    String message = messages[index];
                    return ListTile(
                      title: ListTile(
                        title: Text(message),
                      ),
                    );
                  })),
          ListenableBuilder(
              listenable: widget.stateController,
              builder: (BuildContext context, Widget? child) {
                // TODO these changes need to trigger rebuilds to prevent issues with the message display
                if (!widget.stateController.isCallActive && active) {
                  messages.clear();
                  _messageInput.clear();
                  active = false;
                } else if (widget.stateController.isCallActive && !active) {
                  active = true;
                }

                return Padding(
                    padding:
                        const EdgeInsets.only(bottom: 10, left: 20, right: 20),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Flexible(
                            fit: FlexFit.loose,
                            child: TextInput(
                              controller: _messageInput,
                              labelText: active ? 'Message' : 'Chat disabled',
                              enabled: active,
                              onSubmitted: (message) {
                                if (message.isEmpty) return;
                                sendMessage(message);
                              },
                            )),
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

/// A widget which displays details about the call
class CallDetailsWidget extends StatelessWidget {
  final StatisticsController statisticsController;
  final StateController stateController;

  const CallDetailsWidget(
      {super.key,
      required this.statisticsController,
      required this.stateController});

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
          ListenableBuilder(
              listenable: stateController,
              builder: (BuildContext context, Widget? child) {
                return Text('Call ${stateController.status.toLowerCase()}',
                    style: const TextStyle(fontSize: 20));
              }),
          const Spacer(),
          const Text('Input level'),
          const SizedBox(height: 7),
          ListenableBuilder(
              listenable: statisticsController,
              builder: (BuildContext context, Widget? child) {
                return AudioLevel(
                    level: statisticsController.inputLevel, numRectangles: 20);
              }),
          const SizedBox(height: 9),
          const Text('Output level'),
          const SizedBox(height: 7),
          ListenableBuilder(
              listenable: statisticsController,
              builder: (BuildContext context, Widget? child) {
                return AudioLevel(
                    level: statisticsController.outputLevel, numRectangles: 20);
              }),
          const SizedBox(height: 12),
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              ListenableBuilder(
                  listenable: statisticsController,
                  builder: (BuildContext context, Widget? child) {
                    Color color;

                    if (statisticsController.latency < 50) {
                      color = Colors.green;
                    } else if (statisticsController.latency < 150) {
                      color = Colors.yellow;
                    } else {
                      color = Colors.red;
                    }

                    return Icon(Icons.monitor_heart_rounded, color: color);
                  }),
              const SizedBox(width: 7),
              ListenableBuilder(
                  listenable: statisticsController,
                  builder: (BuildContext context, Widget? child) {
                    return Text('${statisticsController.latency} ms',
                        style: const TextStyle(height: 0));
                  }),
              const Spacer(),
              Icon(Icons.upload, color: Theme.of(context).colorScheme.primary),
              const SizedBox(width: 4),
              ListenableBuilder(
                  listenable: statisticsController,
                  builder: (BuildContext context, Widget? child) {
                    return Text(statisticsController.upload,
                        style: const TextStyle(height: 0));
                  }),
              const Spacer(),
              Icon(Icons.download,
                  color: Theme.of(context).colorScheme.primary),
              const SizedBox(width: 4),
              ListenableBuilder(
                  listenable: statisticsController,
                  builder: (BuildContext context, Widget? child) {
                    return Text(statisticsController.download,
                        style: const TextStyle(height: 0));
                  }),
            ],
          ),
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
  bool _callEndedRecently = false;
  final Stopwatch _callTimer = Stopwatch();

  /// peerId, status
  final Map<String, String> sessions = {};

  /// active, restartable
  (bool, bool) _sessionManager = (false, false);

  Contact? get activeContact => _activeContact;
  bool get isCallActive => _activeContact != null;
  bool get isDeafened => _deafened;
  bool get isMuted => _muted;
  bool get callEndedRecently => _callEndedRecently;
  bool get blockAudioChanges => isCallActive || inAudioTest;
  bool get sessionManagerActive => _sessionManager.$1;
  bool get sessionManagerRestartable => _sessionManager.$2;
  String get callDuration =>
      formatElapsedTime(_callTimer.elapsed.inMilliseconds);

  void setActiveContact(Contact? contact) {
    _activeContact = contact;
    notifyListeners();
  }

  void setStatus(String status) {
    this.status = status;

    if (status == 'Inactive') {
      _activeContact = null;
      _callTimer.stop();
      _callTimer.reset();
    } else if (status == 'Active') {
      _callTimer.start();
    }

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
    String status = sessions[contact.peerId()] ?? 'Unknown';
    return status == 'Connected';
  }

  void updateSession(String peerId, String status) {
    sessions[peerId] = status;
    notifyListeners();
  }

  String sessionStatus(Contact contact) {
    return sessions[contact.peerId()] ?? 'Unknown';
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

    notifyListeners();
  }

  void disableCallsTemporarily() {
    _callEndedRecently = true;

    Timer(const Duration(seconds: 1), () {
      _callEndedRecently = false;
    });
  }
}

class StatisticsController extends ChangeNotifier {
  Statistics? _statistics;

  int get latency => _statistics == null ? 0 : _statistics!.latency;
  double get inputLevel => _statistics == null ? 0 : _statistics!.inputLevel;
  double get outputLevel => _statistics == null ? 0 : _statistics!.outputLevel;
  String get upload =>
      _statistics == null ? '?' : formatBandwidth(_statistics!.uploadBandwidth);
  String get download => _statistics == null
      ? '?'
      : formatBandwidth(_statistics!.downloadBandwidth);

  void setStatistics(Statistics statistics) {
    _statistics = statistics;
    notifyListeners();
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

/// Notifies listeners every second
class PeriodicNotifier extends ChangeNotifier {
  PeriodicNotifier() {
    Timer.periodic(const Duration(seconds: 1), (timer) {
      notifyListeners();
    });
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
Future<List<int>> readWavBytes(String assetName) {
  return readAssetBytes('sounds/$assetName.wav');
}

Future<void> updateDenoiseModel(String model, AudioChat audioChat) async {
  List<int> bytes = await readAssetBytes('models/$model.rnn');
  audioChat.setModel(model: bytes);
}

/// Reads the bytes of a file from the assets
Future<List<int>> readAssetBytes(String assetName) async {
  final ByteData data = await rootBundle.load('assets/$assetName');
  final List<int> bytes = data.buffer.asUint8List();
  return bytes;
}

/// Formats milliseconds into hours:minutes:seconds
String formatElapsedTime(int milliseconds) {
  int hundredths = (milliseconds / 10).truncate();
  int seconds = (hundredths / 100).truncate();
  int minutes = (seconds / 60).truncate();
  int hours = (minutes / 60).truncate();

  String hoursStr = (hours % 60).toString().padLeft(2, '0');
  String minutesStr = (minutes % 60).toString().padLeft(2, '0');
  String secondsStr = (seconds % 60).toString().padLeft(2, '0');

  return "$hoursStr:$minutesStr:$secondsStr";
}

String formatBandwidth(int bytes) {
  if (bytes < 100000000) {
    return '${(bytes / 1000000).toStringAsFixed(2)} MB';
  } else {
    return '${(bytes / 1000000000).toStringAsFixed(2)} GB';
  }
}
