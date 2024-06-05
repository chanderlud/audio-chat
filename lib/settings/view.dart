import 'dart:async';
import 'dart:core';
import 'dart:io';

import 'package:audio_chat/settings/controller.dart';
import 'package:audio_chat/src/rust/api/player.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart' hide Overlay;
import 'package:flutter/services.dart' show Clipboard, ClipboardData;
import 'package:flutter_colorpicker/flutter_colorpicker.dart';

import '../audio_level.dart';
import '../main.dart';
import '../src/rust/api/audio_chat.dart';
import '../src/rust/api/error.dart';
import '../src/rust/api/overlay/overlay.dart';

class SettingsPage extends StatefulWidget {
  final SettingsController controller;
  final AudioChat audioChat;
  final StateController stateController;
  final StatisticsController statisticsController;
  final SoundPlayer player;
  final Overlay overlay;

  const SettingsPage(
      {super.key,
      required this.controller,
      required this.audioChat,
      required this.stateController,
      required this.player,
      required this.statisticsController,
      required this.overlay});

  @override
  SettingsPageState createState() => SettingsPageState();
}

class SettingsPageState extends State<SettingsPage> {
  /// 0 = audio, 1 = profiles, 2 = networking, 3 = interface, 4 = overlay
  int route = 0;

  @override
  Widget build(BuildContext context) {
    double width;

    if (route == 4) {
      width = 1000;
    } else {
      width = 650;
    }

    return Scaffold(
      body: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Container(
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
            ),
            padding: const EdgeInsets.only(left: 20, top: 15, right: 20),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.start,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                IconButton(
                  icon: const Icon(Icons.arrow_back),
                  onPressed: Navigator.of(context).pop,
                ),
                const SizedBox(height: 20),
                // TODO some on hover effects could be nice for these items
                InkWell(
                  onTap: () {
                    setState(() {
                      route = 0;
                    });
                  },
                  child: Container(
                    padding:
                        const EdgeInsets.symmetric(vertical: 5, horizontal: 10),
                    width: 175,
                    decoration: BoxDecoration(
                      color: route == 0
                          ? Theme.of(context).colorScheme.primary
                          : Theme.of(context).colorScheme.surface,
                      borderRadius: BorderRadius.circular(5),
                    ),
                    child: const Text('Audio & Video', style: TextStyle(fontSize: 18)),
                  ),
                ),
                const SizedBox(height: 12),
                InkWell(
                  onTap: () {
                    setState(() {
                      route = 1;
                    });
                  },
                  child: Container(
                    padding:
                        const EdgeInsets.symmetric(vertical: 5, horizontal: 10),
                    width: 175,
                    decoration: BoxDecoration(
                      color: route == 1
                          ? Theme.of(context).colorScheme.primary
                          : Theme.of(context).colorScheme.surface,
                      borderRadius: BorderRadius.circular(5),
                    ),
                    child:
                        const Text('Profiles', style: TextStyle(fontSize: 18)),
                  ),
                ),
                const SizedBox(height: 12),
                InkWell(
                  onTap: () {
                    setState(() {
                      route = 2;
                    });
                  },
                  child: Container(
                    padding:
                        const EdgeInsets.symmetric(vertical: 5, horizontal: 10),
                    width: 175,
                    decoration: BoxDecoration(
                      color: route == 2
                          ? Theme.of(context).colorScheme.primary
                          : Theme.of(context).colorScheme.surface,
                      borderRadius: BorderRadius.circular(5),
                    ),
                    child: const Text('Networking',
                        style: TextStyle(fontSize: 18)),
                  ),
                ),
                const SizedBox(height: 12),
                InkWell(
                  onTap: () {
                    setState(() {
                      route = 3;
                    });
                  },
                  child: Container(
                    padding:
                    const EdgeInsets.symmetric(vertical: 5, horizontal: 10),
                    width: 175,
                    decoration: BoxDecoration(
                      color: route == 3
                          ? Theme.of(context).colorScheme.primary
                          : Theme.of(context).colorScheme.surface,
                      borderRadius: BorderRadius.circular(5),
                    ),
                    child: const Text('Interface',
                        style: TextStyle(fontSize: 18)),
                  ),
                ),
                // overlay is only available on windows
                if (Platform.isWindows)
                  const SizedBox(height: 12),
                if (Platform.isWindows)
                  InkWell(
                    onTap: () {
                      setState(() {
                        route = 4;
                      });
                    },
                    child: Container(
                      padding:
                      const EdgeInsets.symmetric(vertical: 5, horizontal: 10),
                      width: 175,
                      decoration: BoxDecoration(
                        color: route == 4
                            ? Theme.of(context).colorScheme.primary
                            : Theme.of(context).colorScheme.surface,
                        borderRadius: BorderRadius.circular(5),
                      ),
                      child:
                      const Text('Overlay', style: TextStyle(fontSize: 18)),
                    ),
                  ),
              ],
            ),
          ),
          Flexible(
              child: Align(
            alignment: Alignment.topCenter,
            child: SingleChildScrollView(
              child: Padding(
                  padding: const EdgeInsets.only(left: 20, right: 20, top: 30),
                  child: SizedBox(
                    width: width,
                    child: LayoutBuilder(builder:
                        (BuildContext context, BoxConstraints constraints) {
                      if (route == 0) {
                        return AVSettings(
                            controller: widget.controller,
                            audioChat: widget.audioChat,
                            stateController: widget.stateController,
                            player: widget.player,
                            statisticsController: widget.statisticsController,
                            constraints: constraints);
                      } else if (route == 1) {
                        return ProfileSettings(
                            controller: widget.controller,
                            audioChat: widget.audioChat,
                            stateController: widget.stateController);
                      } else if (route == 2) {
                        return NetworkSettings(
                            controller: widget.controller,
                            audioChat: widget.audioChat,
                            stateController: widget.stateController);
                      } else if (route == 4) {
                        return OverlaySettings(
                            overlay: widget.overlay,
                            controller: widget.controller,
                            stateController: widget.stateController);
                      } else {
                        return const SizedBox();
                      }
                    }),
                  )),
            ),
          )),
        ],
      ),
    );
  }
}

class AVSettings extends StatefulWidget {
  final SettingsController controller;
  final AudioChat audioChat;
  final StateController stateController;
  final StatisticsController statisticsController;
  final SoundPlayer player;
  final BoxConstraints constraints;

  const AVSettings(
      {super.key,
      required this.controller,
      required this.audioChat,
      required this.stateController,
      required this.player,
      required this.statisticsController,
      required this.constraints});

  @override
  AVSettingsState createState() => AVSettingsState();
}

class AVSettingsState extends State<AVSettings> {
  List<String> inputDevices = [];
  List<String> outputDevices = [];

  /// triggers periodic device updates
  Timer? _timer;

  @override
  void initState() {
    super.initState();
    updateDevices();

    // update the audio devices every 500ms
    _timer = Timer.periodic(const Duration(milliseconds: 500), (timer) {
      updateDevices();
    });
  }

  @override
  void dispose() {
    // cancel the timer when the widget is disposed
    _timer?.cancel();
    super.dispose();
  }

  void updateDevices() {
    var (inputDevices, outputDevices) = widget.audioChat.listDevices();

    // default devices map to null
    inputDevices.insert(0, 'Default');
    outputDevices.insert(0, 'Default');

    if (this.inputDevices != inputDevices) {
      setState(() {
        this.inputDevices = inputDevices;
      });
    }

    if (this.outputDevices != outputDevices) {
      setState(() {
        this.outputDevices = outputDevices;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ListenableBuilder(
            listenable: widget.stateController,
            builder: (BuildContext context, Widget? child) {
              String inputInitialSelection;

              if (widget.controller.inputDevice == null) {
                inputInitialSelection = 'Default';
              } else if (inputDevices.contains(widget.controller.inputDevice)) {
                inputInitialSelection = widget.controller.inputDevice!;
              } else {
                inputInitialSelection = 'Default';
              }

              String outputInitialSelection;

              if (widget.controller.outputDevice == null) {
                outputInitialSelection = 'Default';
              } else if (outputDevices
                  .contains(widget.controller.outputDevice)) {
                outputInitialSelection = widget.controller.outputDevice!;
              } else {
                outputInitialSelection = 'Default';
              }

              double width = widget.constraints.maxWidth < 650
                  ? widget.constraints.maxWidth
                  : (widget.constraints.maxWidth - 20) / 2;

              return Center(
                child: Wrap(
                  spacing: 20,
                  runSpacing: 20,
                  children: [
                    DropdownMenu<String>(
                      width: width,
                      label: const Text('Input Device'),
                      enabled: !widget.stateController.blockAudioChanges,
                      dropdownMenuEntries:
                          inputDevices.map<DropdownMenuEntry<String>>((device) {
                        return DropdownMenuEntry(
                          value: device,
                          label: device,
                        );
                      }).toList(),
                      onSelected: (String? value) {
                        if (value == 'Default') value = null;
                        widget.controller.updateInputDevice(value);
                        widget.audioChat.setInputDevice(device: value);
                      },
                      initialSelection: inputInitialSelection,
                    ),
                    DropdownMenu<String>(
                      width: width,
                      label: const Text('Output Device'),
                      enabled: !widget.stateController.blockAudioChanges,
                      dropdownMenuEntries: outputDevices
                          .map<DropdownMenuEntry<String>>((device) {
                        return DropdownMenuEntry(
                          value: device,
                          label: device,
                        );
                      }).toList(),
                      onSelected: (String? value) {
                        if (value == 'Default') value = null;
                        widget.controller.updateOutputDevice(value);
                        widget.audioChat.setOutputDevice(device: value);
                        widget.player.updateOutputDevice(name: value);
                      },
                      initialSelection: outputInitialSelection,
                    ),
                  ],
                ),
              );
            }),
        const SizedBox(height: 20),
        Row(children: [
          ListenableBuilder(
              listenable: widget.stateController,
              builder: (BuildContext context, Widget? child) {
                return Button(
                  text: widget.stateController.inAudioTest
                      ? 'End Test'
                      : 'Sound Test',
                  width: 80,
                  height: 25,
                  disabled: widget.stateController.isCallActive,
                  onPressed: () async {
                    if (widget.stateController.inAudioTest) {
                      widget.stateController.setInAudioTest();
                      widget.audioChat.endCall();
                    } else {
                      widget.stateController.setInAudioTest();
                      try {
                        await widget.audioChat.audioTest();
                      } on DartError catch (e) {
                        if (!context.mounted) return;
                        showErrorDialog(
                            context, 'Error in Audio Test', e.message);
                        widget.stateController.setInAudioTest();
                      }
                    }
                  },
                );
              }),
          const SizedBox(width: 20),
          ListenableBuilder(
              listenable: widget.statisticsController,
              builder: (BuildContext context, Widget? child) {
                return AudioLevel(
                    level: widget.statisticsController.inputLevel,
                    numRectangles: (widget.constraints.maxWidth - 145) ~/ 13.5);
              }),
        ]),
        const SizedBox(height: 20),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          mainAxisSize: MainAxisSize.max,
          children: [
            const Text('Noise Suppression', style: TextStyle(fontSize: 18)),
            // const SizedBox(width: 55),
            ListenableBuilder(
                listenable: widget.controller,
                builder: (BuildContext context, Widget? child) {
                  return ListenableBuilder(
                      listenable: widget.stateController,
                      builder: (BuildContext context, Widget? child) {
                        return DropdownMenu<String>(
                          enabled: !widget.stateController.blockAudioChanges,
                          dropdownMenuEntries: const [
                            DropdownMenuEntry(value: 'off', label: 'Off'),
                            DropdownMenuEntry(
                                value: 'vanilla', label: 'Vanilla'),
                            DropdownMenuEntry(
                                value: 'hogwash', label: 'Hogwash'),
                          ],
                          initialSelection: widget.controller.useDenoise
                              ? widget.controller.denoiseModel ?? 'vanilla'
                              : 'off',
                          onSelected: (String? value) {
                            if (value == 'off') {
                              widget.controller.updateUseDenoise(false);
                              widget.audioChat.setDenoise(denoise: false);
                            } else if (value == 'vanilla') {
                              widget.controller.updateUseDenoise(true);
                              widget.controller.setDenoiseModel(null);
                              widget.audioChat.setDenoise(denoise: true);
                              widget.audioChat.setModel(model: []);
                            } else {
                              widget.controller.updateUseDenoise(true);
                              widget.controller.setDenoiseModel(value);
                              widget.audioChat.setDenoise(denoise: true);
                              updateDenoiseModel(value!, widget.audioChat);
                            }
                          },
                        );
                      });
                }),
          ],
        ),
        const SizedBox(height: 5),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          mainAxisSize: MainAxisSize.max,
          children: [
            const Text('Play Custom Ringtones', style: TextStyle(fontSize: 18)),
            // const SizedBox(width: 20),
            ListenableBuilder(
                listenable: widget.controller,
                builder: (BuildContext context, Widget? child) {
                  return CustomSwitch(
                      value: widget.controller.playCustomRingtones,
                      onChanged: (play) {
                        widget.controller.updatePlayCustomRingtones(play);
                        widget.audioChat.setPlayCustomRingtones(play: play);
                      });
                }),
          ],
        ),
        const SizedBox(height: 15),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          mainAxisSize: MainAxisSize.max,
          children: [
            Button(
                text: 'Select custom ringtone file',
                disabled: false,
                onPressed: () async {
                  FilePickerResult? result =
                      await FilePicker.platform.pickFiles(
                    type: FileType.custom,
                    allowedExtensions: ['wav'],
                  );

                  if (result != null) {
                    String? path = result.files.single.path;
                    widget.controller.updateCustomRingtoneFile(path);
                  } else {
                    widget.controller.updateCustomRingtoneFile(null);
                  }
                }),
            ListenableBuilder(
                listenable: widget.controller,
                builder: (BuildContext context, Widget? child) {
                  return Text(
                      widget.controller.customRingtoneFile ??
                          'No file selected',
                      style: const TextStyle(fontSize: 16));
                }),
          ],
        ),
        const SizedBox(height: 20),
        const Text('Sound Effect Volume', style: TextStyle(fontSize: 16)),
        ListenableBuilder(
            listenable: widget.controller,
            builder: (BuildContext context, Widget? child) {
              return Slider(
                  value: widget.controller.soundVolume,
                  onChanged: (value) {
                    widget.controller.updateSoundVolume(value);
                    widget.player.updateOutputVolume(volume: value);
                  },
                  min: -20,
                  max: 20,
                  label:
                      '${widget.controller.soundVolume.toStringAsFixed(2)} db');
            }),
        const Divider(),
        const SizedBox(height: 20),
        const Text('Screen share settings')
      ],
    );
  }
}

class ProfileSettings extends StatefulWidget {
  final SettingsController controller;
  final AudioChat audioChat;
  final StateController stateController;

  const ProfileSettings(
      {super.key,
      required this.controller,
      required this.audioChat,
      required this.stateController});

  @override
  ProfileSettingsState createState() => ProfileSettingsState();
}

class ProfileSettingsState extends State<ProfileSettings> {
  final TextEditingController _profileNameInput = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(5),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.end,
        mainAxisSize: MainAxisSize.min,
        children: [
          ListenableBuilder(
              listenable: widget.controller,
              builder: (BuildContext context, Widget? child) {
                bool even = widget.controller.profiles.length % 2 == 0;

                Color colorPicker(int index) {
                  if (even ? index % 2 == 0 : index % 2 != 0) {
                    return Colors.transparent;
                  } else {
                    return Theme.of(context).colorScheme.secondaryContainer;
                  }
                }

                return ListView.builder(
                    shrinkWrap: true,
                    itemCount: widget.controller.profiles.length,
                    itemBuilder: (BuildContext context, int index) {
                      Profile profile =
                          widget.controller.profiles.values.elementAt(index);

                      return Container(
                        decoration: BoxDecoration(
                          color: colorPicker(index),
                          borderRadius: index == 0
                              ? const BorderRadius.only(
                                  topLeft: Radius.circular(5),
                                  topRight: Radius.circular(5))
                              : null,
                        ),
                        padding: const EdgeInsets.only(
                            top: 5, bottom: 5, left: 20, right: 10),
                        child: Row(
                          children: [
                            Text(
                              profile.nickname,
                              style: const TextStyle(fontSize: 18),
                            ),
                            const Spacer(),
                            ListenableBuilder(
                                listenable: widget.stateController,
                                builder: (BuildContext context, Widget? child) {
                                  // TODO the animation on this button is bad
                                  return Button(
                                    text: (widget.controller.activeProfile ==
                                            profile.id)
                                        ? 'Active'
                                        : 'Set Active',
                                    width: 75,
                                    height: 25,
                                    disabled:
                                        widget.stateController.isCallActive ||
                                            widget.controller.activeProfile ==
                                                profile.id,
                                    onPressed: () {
                                      widget.controller
                                          .setActiveProfile(profile.id);
                                      widget.audioChat
                                          .setIdentity(key: profile.keypair);
                                      widget.audioChat.restartManager();
                                    },
                                    disabledColor: widget
                                                    .controller.activeProfile ==
                                                profile.id &&
                                            widget.stateController.isCallActive
                                        ? Theme.of(context)
                                            .colorScheme
                                            .tertiaryContainer
                                        : null,
                                  );
                                }),
                            const SizedBox(width: 10),
                            IconButton(
                              tooltip: 'Copy Peer ID',
                              onPressed: () {
                                Clipboard.setData(
                                    ClipboardData(text: profile.peerId));
                              },
                              icon: const Icon(Icons.copy),
                            ),
                            IconButton(
                              tooltip: 'Delete Profile',
                              onPressed: () {
                                showDialog(
                                    context: context,
                                    builder: (BuildContext context) {
                                      return AlertDialog(
                                        title: const Text('Delete Profile'),
                                        content: const Text(
                                            'Are you sure you want to delete this profile?'),
                                        actions: [
                                          Button(
                                            text: 'Cancel',
                                            onPressed: () {
                                              Navigator.of(context).pop();
                                            },
                                            disabled: false,
                                          ),
                                          Button(
                                            text: 'Delete',
                                            onPressed: () {
                                              widget.controller
                                                  .removeProfile(profile.id);
                                              Navigator.of(context).pop();
                                            },
                                            disabled: false,
                                          )
                                        ],
                                      );
                                    });
                              },
                              icon: const Icon(Icons.delete),
                            ),
                          ],
                        ),
                      );
                    });
              }),
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 5, horizontal: 20),
            child: IconButton(
              onPressed: () {
                showDialog(
                    context: context,
                    builder: (BuildContext context) {
                      return SimpleDialog(
                        title: const Text('Create Profile'),
                        contentPadding: const EdgeInsets.only(
                            bottom: 25, left: 25, right: 25),
                        titlePadding: const EdgeInsets.only(
                            top: 25, left: 25, right: 25, bottom: 15),
                        children: [
                          TextField(
                            decoration: const InputDecoration(
                              labelText: 'Name',
                            ),
                            controller: _profileNameInput,
                          ),
                          const SizedBox(height: 20),
                          Button(
                            text: 'Create',
                            onPressed: () {
                              widget.controller
                                  .createProfile(_profileNameInput.text);
                              _profileNameInput.clear();
                              Navigator.of(context).pop();
                            },
                            disabled: false,
                          )
                        ],
                      );
                    });
              },
              icon: const Icon(
                Icons.add,
                size: 40,
              ),
              tooltip: 'Create Profile',
            ),
          ),
        ],
      ),
    );
  }
}

class NetworkSettings extends StatefulWidget {
  final SettingsController controller;
  final AudioChat audioChat;
  final StateController stateController;

  const NetworkSettings(
      {super.key,
      required this.controller,
      required this.audioChat,
      required this.stateController});

  @override
  NetworkSettingsState createState() => NetworkSettingsState();
}

class NetworkSettingsState extends State<NetworkSettings> {
  late String _relayAddress;
  late String _relayPeerId;

  final TextEditingController _relayAddressInput = TextEditingController();
  final TextEditingController _relayPeerIdInput = TextEditingController();

  @override
  void initState() {
    super.initState();
    _initialize();
  }

  Future<void> _initialize() async {
    _relayAddress = await widget.controller.networkConfig.getRelay();
    _relayPeerId = await widget.controller.networkConfig.getRelayId();

    _relayAddressInput.text = _relayAddress;
    _relayPeerIdInput.text = _relayPeerId;
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Flexible(
                child: TextInput(
                    labelText: 'Relay Address',
                    controller: _relayAddressInput)),
            const SizedBox(width: 20),
            Flexible(
                child: TextInput(
                    labelText: 'Relay Peer ID', controller: _relayPeerIdInput)),
          ],
        )
      ],
    );
  }
}

class OverlaySettings extends StatefulWidget {
  final Overlay overlay;
  final SettingsController controller;
  final StateController stateController;

  const OverlaySettings(
      {super.key,
      required this.overlay,
      required this.controller,
      required this.stateController});

  @override
  OverlaySettingsState createState() => OverlaySettingsState();
}

class OverlaySettingsState extends State<OverlaySettings> {
  bool overlayVisible = false;

  @override
  void dispose() {
    if (!widget.stateController.isCallActive) {
      widget.overlay.hide();
    }

    widget.controller.saveOverlayConfig();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    (int, int) size = widget.overlay.screenResolution();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Button(
              text: overlayVisible ? "Hide overlay" : "Show overlay",
              onPressed: () {
                if (widget.stateController.isCallActive || !widget.controller.overlayConfig.enabled) {
                  return;
                } else if (overlayVisible) {
                  widget.overlay.hide();
                } else {
                  widget.overlay.show();
                }

                setState(() {
                  overlayVisible = !overlayVisible;
                });
              },
              disabled: widget.stateController.isCallActive || !widget.controller.overlayConfig.enabled,
              width: 100,
            ),
            const SizedBox(width: 20),
            Button(
              text: widget.controller.overlayConfig.enabled ? "Disable overlay" : "Enable overlay",
              onPressed: () async {
                if (widget.controller.overlayConfig.enabled) {
                  await widget.overlay.disable();
                  widget.controller.overlayConfig.enabled = false;

                  // the overlay is never visible when it is disabled
                  setState(() {
                    overlayVisible = false;
                  });
                } else {
                  await widget.overlay.enable();
                  widget.controller.overlayConfig.enabled = true;

                  if (widget.stateController.isCallActive) {
                    // if the call is active, the overlay should be shown
                    widget.overlay.show();

                    setState(() {
                      overlayVisible = true;
                    });
                  } else {
                    // if the call is not active, the overlay should be hidden
                    setState(() {
                      overlayVisible = false;
                    });
                  }
                }

                // save the config
                widget.controller.saveOverlayConfig();
              },
              disabled: false,
              width: 150,
            ),
          ],
        ),
        const SizedBox(height: 20),
        const Text('Font Size', style: TextStyle(fontSize: 18)),
        Slider(
            value: widget.controller.overlayConfig.fontHeight.toDouble(),
            onChanged: (value) {
              widget.overlay.setFontHeight(height: value.round());
              widget.controller.overlayConfig.fontHeight = value.round();
              widget.controller.saveOverlayConfig();
              setState(() {});
            },
            min: 0,
            max: 70),
        const SizedBox(height: 15),
        Row(
          children: [
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('Background Color', style: TextStyle(fontSize: 18)),
                const SizedBox(height: 10),
                Button(text: 'Change', onPressed: () {
                  colorPicker(context, (Color color) {
                    widget.overlay.setBackgroundColor(backgroundColor: argb(color));
                    widget.controller.overlayConfig.backgroundColor = color;
                    widget.controller.saveOverlayConfig();
                    setState(() {});
                  }, widget.controller.overlayConfig.backgroundColor);
                }, disabled: false),
              ],
            ),
            const SizedBox(width: 40),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('Primary Font Color', style: TextStyle(fontSize: 18)),
                const SizedBox(height: 10),
                Button(text: 'Change', onPressed: () {
                  colorPicker(context, (Color color) {
                    widget.overlay.setFontColor(fontColor: argb(color));
                    widget.controller.overlayConfig.fontColor = color;
                    widget.controller.saveOverlayConfig();
                    setState(() {});
                  }, widget.controller.overlayConfig.fontColor);
                }, disabled: false),
              ],
            )
          ],
        ),
        const SizedBox(height: 35),
        if (size.$1 > 0 && size.$2 > 0)
          OverlayPositionWidget(
            overlay: widget.overlay,
            controller: widget.controller,
            realMaxX: size.$1.toDouble(),
            realMaxY: size.$2.toDouble(),
          ),
      ],
    );
  }
}

class OverlayPositionWidget extends StatefulWidget {
  final Overlay overlay;
  final SettingsController controller;

  final double realMaxX;
  final double realMaxY;

  const OverlayPositionWidget(
      {super.key,
      required this.overlay,
      required this.realMaxX,
      required this.realMaxY,
      required this.controller});

  @override
  OverlayPositionWidgetState createState() => OverlayPositionWidgetState();
}

class OverlayPositionWidgetState extends State<OverlayPositionWidget> {
  late double _maxX;
  late double _maxY;
  late double _x;
  late double _y;
  late double _width;
  late double _height;

  bool _isDragging = false;
  bool _isResizing = false;

  @override
  void initState() {
    super.initState();

    _maxX = 650.0;
    _updatePositions();
  }

  void _updatePositions() {
    _maxY = _maxX / (widget.realMaxX / widget.realMaxY);

    _x = widget.controller.overlayConfig.x / widget.realMaxX * _maxX;
    _y = widget.controller.overlayConfig.y / widget.realMaxY * _maxY;
    _width = widget.controller.overlayConfig.width / widget.realMaxX * _maxX;
    _height = widget.controller.overlayConfig.height / widget.realMaxY * _maxY;
  }

  void _updateOverlay() {
    double realX = _x / _maxX * widget.realMaxX;
    double realY = _y / _maxY * widget.realMaxY;
    double realWidth = _width / _maxX * widget.realMaxX;
    double realHeight = _height / _maxY * widget.realMaxY;

    widget.overlay.moveOverlay(
      x: realX.round(),
      y: realY.round(),
      width: realWidth.round(),
      height: realHeight.round(),
    );

    widget.controller.overlayConfig.x = realX;
    widget.controller.overlayConfig.y = realY;
    widget.controller.overlayConfig.width = realWidth;
    widget.controller.overlayConfig.height = realHeight;
  }

  void _onDragUpdate(DragUpdateDetails details) {
    if (_isDragging) {
      setState(() {
        _x += details.delta.dx;
        _y += details.delta.dy;

        if (_x < 0) {
          _x = 0;
        } else if (_x + _width > _maxX) {
          _x = _maxX - _width;
        }

        if (_y < 0) {
          _y = 0;
        } else if (_y + _height > _maxY) {
          _y = _maxY - _height;
        }

        _updateOverlay();
      });
    }
  }

  void _onResizeUpdate(DragUpdateDetails details) {
    if (_isResizing) {
      setState(() {
        _width += details.delta.dx;
        _height += details.delta.dy;

        if (_width + _x > _maxX) {
          _width = _maxX - _x;
        } else if (_width < 10) {
          _width = 10;
        }

        if (_height + _y > _maxY) {
          _height = _maxY - _y;
        } else if (_height < 10) {
          _height = 10;
        }

        _updateOverlay();
      });
    }
  }

  void _startDragging() {
    setState(() {
      _isDragging = true;
    });
  }

  void _startResizing() {
    setState(() {
      _isResizing = true;
    });
  }

  void _stopDragging() {
    setState(() {
      _isDragging = false;
    });
  }

  void _stopResizing() {
    setState(() {
      _isResizing = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        _maxX = constraints.maxWidth;
        _updatePositions();

        return Container(
          decoration: BoxDecoration(
            border: Border.all(color: Colors.black, width: 2),
            color: Theme.of(context).colorScheme.surface,
          ),
          height: _maxY,
          child: Stack(
            children: [
              Positioned(
                left: _x,
                top: _y,
                child: GestureDetector(
                  onPanUpdate: _onDragUpdate,
                  onPanStart: (_) => _startDragging(),
                  onPanEnd: (_) => _stopDragging(),
                  child: Container(
                    decoration: BoxDecoration(
                      color: widget.controller.overlayConfig.backgroundColor,
                      border: Border.all(color: Colors.yellow.shade400, width: 2),
                    ),
                    child: MouseRegion(
                      cursor: SystemMouseCursors.move,
                      child: SizedBox(
                        width: _width,
                        height: _height,
                      ),
                    )
                  ),
                ),
              ),
              Positioned(
                left: _x + _width - 10,
                top: _y + _height - 10,
                child: GestureDetector(
                  onPanUpdate: _onResizeUpdate,
                  onPanStart: (_) => _startResizing(),
                  onPanEnd: (_) => _stopResizing(),
                  child: const MouseRegion(
                    cursor: SystemMouseCursors.resizeDownRight,
                    child: SizedBox(width: 20, height: 20,),
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

void colorPicker(BuildContext context, void Function(Color) changeColor, Color currentColor) {
  showDialog(
    context: context,
    builder: (BuildContext context) {
      return AlertDialog(
        title: const Text('Pick a color'),
        content: SingleChildScrollView(
          child: ColorPicker(
            pickerColor: currentColor,
            onColorChanged: changeColor,
          ),
        ),
        actions: <Widget>[
          Button(
            text: 'Close',
            onPressed: () {
              Navigator.of(context).pop();
            },
            disabled: false,
          ),
        ],
      );
    },
  );
}
