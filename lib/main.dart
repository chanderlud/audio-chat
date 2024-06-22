import 'dart:async';
import 'dart:io';

import 'package:audio_chat/settings/view.dart';
import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:audio_chat/src/rust/api/contact.dart';
import 'package:audio_chat/src/rust/api/error.dart';
import 'package:audio_chat/src/rust/api/logger.dart';
import 'package:audio_chat/src/rust/api/player.dart';
import 'package:audio_chat/src/rust/api/overlay/overlay.dart';
import 'package:audio_chat/src/rust/frb_generated.dart';
import 'package:audio_chat/settings/controller.dart';
import 'package:flutter/material.dart' hide Overlay;
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_svg/svg.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'audio_level.dart';
import 'console.dart';

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

  final Overlay overlay = await Overlay.newInstance(
    enabled: settingsController.overlayConfig.enabled,
    x: settingsController.overlayConfig.x.round(),
    y: settingsController.overlayConfig.y.round(),
    width: settingsController.overlayConfig.width.round(),
    height: settingsController.overlayConfig.height.round(),
    fontHeight: settingsController.overlayConfig.fontHeight,
    backgroundColor: argb(settingsController.overlayConfig.backgroundColor),
    fontColor: argb(settingsController.overlayConfig.fontColor),
  );

  final soundPlayer = SoundPlayer(outputVolume: settingsController.soundVolume);
  soundPlayer.updateOutputDevice(name: settingsController.outputDevice);
  soundPlayer.updateOutputVolume(volume: settingsController.soundVolume);

  var host = soundPlayer.host();

  final messageBus = MessageBus();

  final audioChat = await AudioChat.newInstance(
      identity: settingsController.keypair,
      host: host,
      networkConfig: settingsController.networkConfig,
      overlay: overlay,
      // called when there is an incoming call
      acceptCall: (String id, Uint8List? ringtone, DartNotify cancel) async {
        Contact? contact = settingsController.getContact(id);

        if (stateController.isCallActive) {
          return false;
        } else if (contact == null) {
          DebugConsole.warn('contact is null');
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
          DebugConsole.warn("call ended entered but there is no active call");
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
          return contact.pubClone();
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
        stateController.callDisconnected = false;
      },
      // TODO use a handle to cancel the sound as needed
      // called when the call disconnects or reconnects
      callState: (bool disconnected) async {
        if (disconnected &&
            stateController.isCallActive &&
            !stateController.callDisconnected) {
          List<int> bytes = await readWavBytes('disconnected');
          await soundPlayer.play(bytes: bytes);

          stateController.setStatus('Reconnecting');
          stateController.callDisconnected = true;
        } else if (!disconnected &&
            stateController.isCallActive &&
            stateController.callDisconnected) {
          List<int> bytes = await readWavBytes('reconnected');
          await soundPlayer.play(bytes: bytes);

          stateController.setStatus('Active');
          stateController.callDisconnected = false;
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

  final audioDevices = AudioDevices(audioChat: audioChat);

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
    statisticsController: statisticsController,
    overlay: overlay,
    audioDevices: audioDevices,
  ));
}

/// The main app
class AudioChatApp extends StatelessWidget {
  final AudioChat audioChat;
  final SettingsController settingsController;
  final StateController callStateController;
  final StatisticsController statisticsController;
  final SoundPlayer player;
  final MessageBus messageBus;
  final Overlay overlay;
  final AudioDevices audioDevices;

  const AudioChatApp(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.callStateController,
      required this.player,
      required this.messageBus,
      required this.statisticsController,
      required this.overlay,
      required this.audioDevices});

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
          trackOutlineWidth: WidgetStateProperty.all(0),
          trackOutlineColor: WidgetStateProperty.all(Colors.transparent),
          overlayColor: WidgetStateProperty.all(Colors.transparent),
          thumbColor: WidgetStateProperty.all(Theme.of(context).indicatorColor),
        ),
        dropdownMenuTheme: DropdownMenuThemeData(
          menuStyle: MenuStyle(
            backgroundColor: WidgetStateProperty.all(const Color(0xFF191919)),
            surfaceTintColor: WidgetStateProperty.all(const Color(0xFF191919)),
          ),
        ),
      ),
      home: HomePage(
        audioChat: audioChat,
        settingsController: settingsController,
        stateController: callStateController,
        player: player,
        messageBus: messageBus,
        statisticsController: statisticsController,
        overlay: overlay,
        audioDevices: audioDevices,
      ),
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
  final Overlay overlay;
  final AudioDevices audioDevices;

  const HomePage(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.stateController,
      required this.player,
      required this.messageBus,
      required this.statisticsController,
      required this.overlay,
      required this.audioDevices});

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
      notifier: notifier,
      overlay: overlay,
      audioDevices: audioDevices,
    );

    return Scaffold(
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
                                player: player,
                                settingsController: settingsController))
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
    );
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
  final TextEditingController _peerIdInput = TextEditingController();

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
              controller: _peerIdInput,
              labelText: 'Peer ID',
              hintText: 'string encoded peer ID',
              obscureText: true),
          const SizedBox(height: 26),
          Center(
            child: Button(
              text: 'Add Contact',
              onPressed: () async {
                String nickname = _nicknameInput.text;
                String peerId = _peerIdInput.text;

                if (nickname.isEmpty || peerId.isEmpty) {
                  showErrorDialog(context, 'Failed to add contact',
                      'Nickname and peer id cannot be empty');
                  return;
                } else if (widget.settingsController.contacts.keys
                    .contains(peerId)) {
                  showErrorDialog(context, 'Failed to add contact',
                      'Contact for peer ID already exists');
                  return;
                } else if (widget.settingsController.peerId == peerId) {
                  showErrorDialog(context, 'Failed to add contact',
                      'Cannot add self as a contact');
                  return;
                }

                try {
                  Contact contact =
                      widget.settingsController.addContact(nickname, peerId);

                  widget.audioChat.startSession(contact: contact);

                  _nicknameInput.clear();
                  _peerIdInput.clear();
                } on DartError catch (_) {
                  showErrorDialog(
                      context, 'Failed to add contact', 'Invalid peer ID');
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
      padding: const EdgeInsets.only(bottom: 15, left: 12, right: 12, top: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(10.0),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          const Padding(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 7),
              child: Text("Contacts", style: TextStyle(fontSize: 20))),
          const SizedBox(height: 10.0),
          Flexible(
            fit: FlexFit.loose,
            child: Container(
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.tertiaryContainer,
                borderRadius: BorderRadius.circular(10),
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
                          stateController: stateController,
                          player: player,
                          settingsController: settingsController,
                        );
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
class ContactWidget extends StatefulWidget {
  final Contact contact;
  final AudioChat audioChat;
  final StateController stateController;
  final SettingsController settingsController;
  final SoundPlayer player;

  const ContactWidget(
      {super.key,
      required this.contact,
      required this.audioChat,
      required this.stateController,
      required this.player,
      required this.settingsController});

  @override
  State<StatefulWidget> createState() => ContactWidgetState();
}

class ContactWidgetState extends State<ContactWidget> {
  bool isHovered = false;
  late TextEditingController _nicknameInput;

  @override
  void initState() {
    super.initState();
    _nicknameInput = TextEditingController(text: widget.contact.nickname());
  }

  @override
  void didUpdateWidget(ContactWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.contact != oldWidget.contact) {
      _nicknameInput.text = widget.contact.nickname();
    }
  }

  @override
  void dispose() {
    _nicknameInput.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    bool online = widget.stateController.isOnlineContact(widget.contact);
    bool active = widget.stateController.isActiveContact(widget.contact);
    String status =
        widget.stateController.sessions[widget.contact.peerId()] ?? 'Unknown';

    return InkWell(
      onHover: (hover) {
        setState(() {
          isHovered = hover;
        });
      },
      // on tap is required for on hover to work for some reason
      onTap: () {},
      child: Container(
        margin: const EdgeInsets.all(5.0),
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.secondaryContainer,
          borderRadius: BorderRadius.circular(10.0),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6.5),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircleAvatar(
              maxRadius: 17,
              child: SvgPicture.asset('assets/icons/Profile.svg',
                  semanticsLabel: 'A profile icon'),
            ),
            const SizedBox(width: 10),
            Text(widget.contact.nickname(),
                style: const TextStyle(fontSize: 16)),
            if (isHovered) const SizedBox(width: 10),
            if (isHovered)
              IconButton(
                  onPressed: () {
                    showDialog(
                        barrierDismissible: false,
                        context: context,
                        builder: (BuildContext context) {
                          return SimpleDialog(
                            title: Row(
                              mainAxisAlignment: MainAxisAlignment.spaceBetween,
                              children: [
                                const Text('Edit Contact'),
                                IconButton(
                                  onPressed: () async {
                                    if (!widget.stateController
                                        .isActiveContact(widget.contact)) {
                                      bool confirm = await showDialog<bool>(
                                              context: context,
                                              builder: (BuildContext context) {
                                                return SimpleDialog(
                                                  title: const Text('Warning'),
                                                  contentPadding:
                                                      const EdgeInsets.only(
                                                          bottom: 25,
                                                          left: 25,
                                                          right: 25),
                                                  titlePadding:
                                                      const EdgeInsets.only(
                                                          top: 25,
                                                          left: 25,
                                                          right: 25,
                                                          bottom: 20),
                                                  children: [
                                                    const Text(
                                                        'Are you sure you want to delete this contact?'),
                                                    const SizedBox(height: 20),
                                                    Row(
                                                      mainAxisAlignment:
                                                          MainAxisAlignment.end,
                                                      children: [
                                                        Button(
                                                          text: 'Cancel',
                                                          onPressed: () {
                                                            Navigator.pop(
                                                                context, false);
                                                          },
                                                        ),
                                                        const SizedBox(
                                                            width: 10),
                                                        Button(
                                                          text: 'Delete',
                                                          onPressed: () {
                                                            Navigator.pop(
                                                                context, true);
                                                          },
                                                        ),
                                                      ],
                                                    ),
                                                  ],
                                                );
                                              }) ??
                                          false;

                                      if (confirm) {
                                        widget.settingsController
                                            .removeContact(widget.contact);
                                        widget.audioChat.stopSession(
                                            contact: widget.contact);
                                        widget.settingsController
                                            .saveContacts();
                                      }

                                      if (context.mounted) {
                                        Navigator.pop(context);
                                      }
                                    } else {
                                      showErrorDialog(context, 'Warning',
                                          'Cannot delete a contact while in an active call');
                                    }
                                  },
                                  icon: SvgPicture.asset(
                                      'assets/icons/Trash.svg',
                                      semanticsLabel: 'Delete contact icon'),
                                ),
                              ],
                            ),
                            contentPadding: const EdgeInsets.only(
                                bottom: 25, left: 25, right: 25),
                            titlePadding: const EdgeInsets.only(
                                top: 25, left: 25, right: 25, bottom: 20),
                            children: [
                              TextInput(
                                  enabled: !widget.stateController
                                      .isActiveContact(widget.contact),
                                  controller: _nicknameInput,
                                  labelText: 'Nickname',
                                  onChanged: (value) {
                                    widget.contact.setNickname(nickname: value);
                                  }),
                              const SizedBox(height: 20),
                              Button(
                                text: 'Save',
                                onPressed: () {
                                  widget.settingsController.saveContacts();
                                  Navigator.pop(context);
                                },
                              ),
                            ],
                          );
                        });
                  },
                  icon: SvgPicture.asset('assets/icons/Edit.svg',
                      semanticsLabel: 'Edit contact icon')),
            const Spacer(),
            if (status == 'Inactive')
              IconButton(
                  onPressed: () {
                    widget.audioChat.startSession(contact: widget.contact);
                  },
                  icon: SvgPicture.asset('assets/icons/Restart.svg',
                      semanticsLabel: 'Retry the session initiation')),
            if (status == 'Inactive') const SizedBox(width: 4),
            if (status == 'Connecting')
              const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 3)),
            if (status == 'Connecting') const SizedBox(width: 10),
            if (!online)
              Padding(
                  padding: const EdgeInsets.only(left: 7, right: 10),
                  child: SvgPicture.asset(
                    'assets/icons/Offline.svg',
                    semanticsLabel: 'Offline icon',
                    width: 26,
                  )),
            if (active)
              IconButton(
                visualDensity: VisualDensity.compact,
                icon: SvgPicture.asset(
                  'assets/icons/PhoneOff.svg',
                  semanticsLabel: 'End call icon',
                  width: 28,
                ),
                onPressed: () async {
                  outgoingSoundHandle?.cancel();

                  widget.audioChat.endCall();
                  widget.stateController.setActiveContact(null);
                  widget.stateController.setStatus('Inactive');
                  widget.stateController.disableCallsTemporarily();

                  List<int> bytes = await readWavBytes('call_ended');
                  await widget.player.play(bytes: bytes);
                },
              ),
            if (!active && online)
              IconButton(
                visualDensity: VisualDensity.compact,
                icon: SvgPicture.asset(
                  'assets/icons/Phone.svg',
                  semanticsLabel: 'Call icon',
                  width: 28,
                ),
                onPressed: () async {
                  if (widget.stateController.isCallActive) {
                    showErrorDialog(context, 'Call failed',
                        'There is a call already active');
                    return;
                  } else if (widget.stateController.inAudioTest) {
                    showErrorDialog(context, 'Call failed',
                        'Cannot make a call while in an audio test');
                    return;
                  } else if (widget.stateController.callEndedRecently) {
                    // if the call button is pressed right after a call ended, we assume the user did not want to make a call
                    return;
                  }

                  widget.stateController.setStatus('Connecting');
                  List<int> bytes = await readWavBytes('outgoing');
                  outgoingSoundHandle = await widget.player.play(bytes: bytes);

                  try {
                    await widget.audioChat.sayHello(contact: widget.contact);
                    widget.stateController.setActiveContact(widget.contact);
                  } on DartError catch (e) {
                    widget.stateController.setStatus('Inactive');
                    outgoingSoundHandle?.cancel();
                    if (!context.mounted) return;
                    showErrorDialog(context, 'Call failed', e.message);
                  }
                },
              )
          ],
        ),
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
  final Overlay overlay;
  final AudioDevices audioDevices;

  const CallControls(
      {super.key,
      required this.audioChat,
      required this.settingsController,
      required this.stateController,
      required this.player,
      required this.statisticsController,
      required this.notifier,
      required this.overlay,
      required this.audioDevices});

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
                          style: TextStyle(
                              fontSize: 16, color: Color(0xFFdc2626))),
                      stateController.sessionManagerRestartable
                          ? const Spacer()
                          : const SizedBox(width: 10),
                      stateController.sessionManagerRestartable
                          ? IconButton(
                              onPressed: () {
                                audioChat.restartManager();
                              },
                              icon: SvgPicture.asset('assets/icons/Restart.svg',
                                  colorFilter: const ColorFilter.mode(
                                      Color(0xFFdc2626), BlendMode.srcIn),
                                  semanticsLabel: 'Restart session manager'))
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
                              icon: SvgPicture.asset(
                                  stateController.isDeafened |
                                          stateController.isMuted
                                      ? 'assets/icons/MicrophoneOff.svg'
                                      : 'assets/icons/Microphone.svg',
                                  width: 24));
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
                              visualDensity: VisualDensity.comfortable,
                              icon: SvgPicture.asset(
                                  stateController.isDeafened
                                      ? 'assets/icons/SpeakerOff.svg'
                                      : 'assets/icons/Speaker.svg',
                                  width: 28));
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
                                  player: player,
                                  overlay: overlay,
                                  audioDevices: audioDevices,
                                ),
                              ));
                        },
                        icon: SvgPicture.asset('assets/icons/Settings.svg')),
                    const SizedBox(width: 1),
                    IconButton(
                        onPressed: () {},
                        icon: SvgPicture.asset('assets/icons/ScreenShare.svg')),
                  ],
                )),
              ))
        ],
      ),
    );
  }
}

class ChatWidget extends StatefulWidget {
  final AudioChat audioChat;
  final StateController stateController;
  final SettingsController settingsController;
  final MessageBus messageBus;
  final SoundPlayer player;

  const ChatWidget(
      {super.key,
      required this.audioChat,
      required this.stateController,
      required this.messageBus,
      required this.player,
      required this.settingsController});

  @override
  State<StatefulWidget> createState() => ChatWidgetState();
}

class ChatWidgetState extends State<ChatWidget> {
  late TextEditingController _messageInput;
  List<ChatMessage> messages = [];
  late StreamSubscription<ChatMessage> messageSubscription;
  bool active = false;

  @override
  void initState() {
    super.initState();
    messageSubscription =
        widget.messageBus.messageStream.listen(receivedMessage);
    _messageInput = TextEditingController();
    widget.stateController.addListener(() {
      if (widget.stateController.isCallActive == active) {
        return;
      } else if (!widget.stateController.isCallActive && active) {
        messages.clear();
        _messageInput.clear();
      }

      setState(() {
        active = widget.stateController.isCallActive;
      });
    });
  }

  @override
  void dispose() {
    messageSubscription.cancel();
    _messageInput.dispose();
    super.dispose();
  }

  // TODO add a sound effect here too
  void sendMessage(String text) async {
    if (!active) return;

    Contact contact = widget.stateController.activeContact!;

    try {
      ChatMessage message =
          await widget.audioChat.buildChat(contact: contact, text: text);
      widget.audioChat.sendChat(message: message);

      setState(() {
        messages.add(message);
        _messageInput.clear();
      });
    } on DartError catch (error) {
      showErrorDialog(context, 'Message Send Failed', error.message);
    }
  }

  void receivedMessage(ChatMessage message) async {
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
      padding: const EdgeInsets.only(left: 10, right: 10, top: 5, bottom: 10),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Flexible(
              fit: FlexFit.loose,
              child: ListView.builder(
                  itemCount: messages.length,
                  itemBuilder: (BuildContext context, int index) {
                    ChatMessage message = messages[index];
                    bool sender = message.isSender(
                        identity: widget.settingsController.peerId);

                    return Align(
                      alignment:
                          sender ? Alignment.centerRight : Alignment.centerLeft,
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 10, vertical: 5),
                        margin: const EdgeInsets.symmetric(vertical: 5),
                        decoration: BoxDecoration(
                            color: sender
                                ? Theme.of(context).colorScheme.secondary
                                : Theme.of(context)
                                    .colorScheme
                                    .tertiaryContainer,
                            borderRadius: BorderRadius.only(
                                topLeft: const Radius.circular(10.0),
                                topRight: const Radius.circular(10.0),
                                bottomLeft: Radius.circular(sender ? 10.0 : 0),
                                bottomRight:
                                    Radius.circular(sender ? 0 : 10.0))),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          crossAxisAlignment: CrossAxisAlignment.end,
                          children: [
                            Text(message.text),
                            const SizedBox(width: 5),
                            Text(message.time(),
                                style: TextStyle(
                                    fontSize: 10,
                                    color:
                                        sender ? Colors.white60 : Colors.grey)),
                          ],
                        ),
                      ),
                    );
                  })),
          ListenableBuilder(
              listenable: widget.stateController,
              builder: (BuildContext context, Widget? child) {
                return Row(
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
                    if (active) const SizedBox(width: 10),
                    if (active)
                      IconButton(
                        onPressed: () {
                          String message = _messageInput.text;
                          if (message.isEmpty) return;
                          sendMessage(message);
                        },
                        icon: SvgPicture.asset(
                          'assets/icons/Send.svg',
                          semanticsLabel: 'Send button icon',
                          width: 36,
                        ),
                      )
                  ],
                );
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
          const SizedBox(height: 8),
          ListenableBuilder(
              listenable: statisticsController,
              builder: (BuildContext context, Widget? child) {
                Color color = getColor(statisticsController.loss);
                return Row(
                  children: [
                    const Text(
                      'Loss: ',
                      style: TextStyle(fontSize: 17),
                    ),
                    Text(
                      '${(statisticsController.loss * 100).toStringAsFixed(1)}%',
                      style: TextStyle(color: color, fontSize: 17),
                    ),
                  ],
                );
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
                    Color color = getColor(statisticsController.latency / 200);
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
  final Color? disabledColor;
  final bool noSplash;

  const Button(
      {super.key,
      required this.text,
      required this.onPressed,
      this.width,
      this.height,
      this.disabled = false,
      this.disabledColor,
      this.noSplash = false});

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
        splashFactory: noSplash ? NoSplash.splashFactory : null,
        backgroundColor: disabled
            ? WidgetStateProperty.all(disabledColor ?? Colors.grey)
            : WidgetStateProperty.all(Theme.of(context).colorScheme.primary),
        foregroundColor: WidgetStateProperty.all(Colors.white),
        overlayColor: disabled
            ? WidgetStateProperty.all(disabledColor ?? Colors.grey)
            : WidgetStateProperty.all(Theme.of(context).colorScheme.secondary),
        surfaceTintColor: WidgetStateProperty.all(Colors.transparent),
        mouseCursor: disabled
            ? WidgetStateProperty.all(SystemMouseCursors.basic)
            : WidgetStateProperty.all(SystemMouseCursors.click),
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
  final Widget? error;

  const TextInput(
      {super.key,
      required this.labelText,
      this.hintText,
      required this.controller,
      this.obscureText,
      this.enabled,
      this.onChanged,
      this.onSubmitted,
      this.error});

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
        error: error,
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
  bool callDisconnected = false;
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

  int get latency => _statistics == null ? 0 : _statistics!.latency.toInt();
  double get inputLevel => _statistics == null ? 0 : _statistics!.inputLevel;
  double get outputLevel => _statistics == null ? 0 : _statistics!.outputLevel;
  String get upload => _statistics == null
      ? '?'
      : formatBandwidth(_statistics!.uploadBandwidth.toInt());
  String get download => _statistics == null
      ? '?'
      : formatBandwidth(_statistics!.downloadBandwidth.toInt());
  double get loss => _statistics == null ? 0 : _statistics!.loss;

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
  final _messageController = StreamController<ChatMessage>.broadcast();

  // for widgets to listen to the stream
  Stream<ChatMessage> get messageStream => _messageController.stream;

  // called to send a message
  void sendMessage(ChatMessage message) {
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
  int seconds = (milliseconds / 1000).truncate();
  int minutes = (seconds / 60).truncate();
  int hours = (minutes / 60).truncate();

  String hoursStr = (hours % 60).toString().padLeft(2, '0');
  String minutesStr = (minutes % 60).toString().padLeft(2, '0');
  String secondsStr = (seconds % 60).toString().padLeft(2, '0');

  return "$hoursStr:$minutesStr:$secondsStr";
}

String formatBandwidth(int bytes) {
  if (bytes < 100000) {
    return '${roundToTotalDigits(bytes / 1000)} KB';
  } else if (bytes < 100000000) {
    return '${roundToTotalDigits(bytes / 1000000)} MB';
  } else {
    return '${roundToTotalDigits(bytes / 1000000000)} GB';
  }
}

String roundToTotalDigits(double number) {
  // get the number of digits before the decimal point
  int integerDigits = number.abs().toInt().toString().length;

  // calculate the number of fractional digits needed
  int fractionalDigits = 3 - integerDigits;
  if (fractionalDigits < 0) {
    fractionalDigits =
        0; // ff the total digits is less than the integer part, we round to the integer part
  }

  // round to the required number of fractional digits
  return number.toStringAsFixed(fractionalDigits).padRight(4, '0');
}
