import 'dart:async';
import 'dart:core';

import 'package:audio_chat/settings/controller.dart';
import 'package:audio_chat/src/rust/api/player.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show Clipboard, ClipboardData;

import '../main.dart';
import '../src/rust/api/audio_chat.dart';
import '../src/rust/api/error.dart';

class SettingsPage extends StatefulWidget {
  final SettingsController controller;
  final AudioChat audioChat;
  final StateController stateController;
  final StatisticsController statisticsController;
  final SoundPlayer player;

  const SettingsPage(
      {super.key,
      required this.controller,
      required this.audioChat,
      required this.stateController,
      required this.player,
      required this.statisticsController});

  @override
  SettingsPageState createState() => SettingsPageState();
}

class SettingsPageState extends State<SettingsPage> {
  late List<String> inputDevices;
  late List<String> outputDevices;
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

    setState(() {
      this.inputDevices = inputDevices;
      this.outputDevices = outputDevices;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () {
            Navigator.of(context).pop();
          },
        ),
        backgroundColor: Theme.of(context).colorScheme.background,
      ),
      body: Center(
        child: Padding(
            padding: const EdgeInsets.all(20),
            child: SizedBox(
              width: 650,
              child: LayoutBuilder(builder: (context, constraints) {
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    ListenableBuilder(
                        listenable: widget.stateController,
                        builder: (BuildContext context, Widget? child) {
                          String inputInitialSelection;

                          if (widget.controller.inputDevice == null) {
                            inputInitialSelection = 'Default';
                          } else if (inputDevices
                              .contains(widget.controller.inputDevice)) {
                            inputInitialSelection =
                                widget.controller.inputDevice!;
                          } else {
                            inputInitialSelection = 'Default';
                          }

                          String outputInitialSelection;

                          if (widget.controller.outputDevice == null) {
                            outputInitialSelection = 'Default';
                          } else if (outputDevices
                              .contains(widget.controller.outputDevice)) {
                            outputInitialSelection =
                                widget.controller.outputDevice!;
                          } else {
                            outputInitialSelection = 'Default';
                          }

                          double width = constraints.maxWidth < 650
                              ? constraints.maxWidth
                              : (constraints.maxWidth - 20) / 2;

                          return Center(
                            child: Wrap(
                              spacing: 20,
                              runSpacing: 20,
                              children: [
                                DropdownMenu<String>(
                                  width: width,
                                  label: const Text('Input device'),
                                  enabled:
                                      !widget.stateController.blockAudioChanges,
                                  dropdownMenuEntries: inputDevices
                                      .map<DropdownMenuEntry<String>>((device) {
                                    return DropdownMenuEntry(
                                      value: device,
                                      label: device,
                                    );
                                  }).toList(),
                                  onSelected: (String? value) {
                                    if (value == 'Default') value = null;
                                    widget.controller.updateInputDevice(value);
                                    widget.audioChat
                                        .setInputDevice(device: value);
                                  },
                                  initialSelection: inputInitialSelection,
                                ),
                                DropdownMenu<String>(
                                  width: width,
                                  label: const Text('Output device'),
                                  enabled:
                                      !widget.stateController.blockAudioChanges,
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
                                    widget.audioChat
                                        .setOutputDevice(device: value);
                                    widget.player
                                        .updateOutputDevice(name: value);
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
                                    showErrorDialog(context,
                                        'Error in Audio Test', e.message);
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
                                numRectangles:
                                    (constraints.maxWidth - 145) ~/ 13.5);
                          }),
                    ]),
                    const SizedBox(height: 20),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      mainAxisSize: MainAxisSize.max,
                      children: [
                        const Text('Noise Suppression',
                            style: TextStyle(fontSize: 18)),
                        // const SizedBox(width: 55),
                        ListenableBuilder(
                            listenable: widget.controller,
                            builder: (BuildContext context, Widget? child) {
                              return ListenableBuilder(
                                  listenable: widget.stateController,
                                  builder:
                                      (BuildContext context, Widget? child) {
                                    return DropdownMenu<String>(
                                      enabled: !widget
                                          .stateController.blockAudioChanges,
                                      dropdownMenuEntries: const [
                                        DropdownMenuEntry(
                                            value: 'off', label: 'Off'),
                                        DropdownMenuEntry(
                                            value: 'vanilla', label: 'Vanilla'),
                                        DropdownMenuEntry(
                                            value: 'hogwash', label: 'Hogwash'),
                                      ],
                                      initialSelection: widget
                                              .controller.useDenoise
                                          ? widget.controller.denoiseModel ??
                                              'vanilla'
                                          : 'off',
                                      onSelected: (String? value) {
                                        if (value == 'off') {
                                          widget.controller
                                              .updateUseDenoise(false);
                                          widget.audioChat
                                              .setDenoise(denoise: false);
                                        } else if (value == 'vanilla') {
                                          widget.controller
                                              .updateUseDenoise(true);
                                          widget.controller
                                              .setDenoiseModel(null);
                                          widget.audioChat
                                              .setDenoise(denoise: true);
                                          widget.audioChat.setModel(model: []);
                                        } else {
                                          widget.controller
                                              .updateUseDenoise(true);
                                          widget.controller
                                              .setDenoiseModel(value);
                                          widget.audioChat
                                              .setDenoise(denoise: true);
                                          updateDenoiseModel(
                                              value!, widget.audioChat);
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
                        const Text('Play Custom Ringtones',
                            style: TextStyle(fontSize: 18)),
                        // const SizedBox(width: 20),
                        ListenableBuilder(
                            listenable: widget.controller,
                            builder: (BuildContext context, Widget? child) {
                              return CustomSwitch(
                                  value: widget.controller.playCustomRingtones,
                                  onChanged: (play) {
                                    widget.controller
                                        .updatePlayCustomRingtones(play);
                                    widget.audioChat
                                        .setPlayCustomRingtones(play: play);
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
                                widget.controller
                                    .updateCustomRingtoneFile(path);
                              } else {
                                widget.controller
                                    .updateCustomRingtoneFile(null);
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
                    const Text('Sound Effect Volume',
                        style: TextStyle(fontSize: 16)),
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
                    const Divider(height: 30),
                    // TODO finish the profile UI
                    Button(
                        text: 'Create profile',
                        disabled: false,
                        onPressed: () {
                          widget.controller.createProfile(
                              widget.controller.profiles.length.toString());
                        }),
                    const SizedBox(height: 20),
                    Expanded(
                        child: ListenableBuilder(
                            listenable: widget.controller,
                            builder: (BuildContext context, Widget? child) {
                              return ListView.builder(
                                  itemCount: widget.controller.profiles.length,
                                  itemBuilder: (context, index) {
                                    Profile profile = widget
                                        .controller.profiles.values
                                        .elementAt(index);

                                    Widget leading;

                                    if (widget.stateController.isCallActive ||
                                        widget.controller.activeProfile ==
                                            profile.id) {
                                      leading = Text(
                                          widget.controller.activeProfile ==
                                                  profile.id
                                              ? 'Active'
                                              : 'Set Active');
                                    } else {
                                      leading = Button(
                                          text: (widget.controller
                                                      .activeProfile ==
                                                  profile.id)
                                              ? 'Active'
                                              : 'Set Active',
                                          width: 75,
                                          height: 25,
                                          disabled: false,
                                          onPressed: () {
                                            widget.controller
                                                .setActiveProfile(profile.id);
                                            widget.audioChat.setIdentity(
                                                key: profile.keypair);
                                            widget.audioChat.restartManager();
                                          });
                                    }

                                    return ListTile(
                                      leading: leading,
                                      title: Text(profile.nickname),
                                      trailing: Button(
                                          text: 'View Peer ID',
                                          disabled: false,
                                          onPressed: () {
                                            showDialog(
                                              context: context,
                                              builder: (BuildContext context) {
                                                return AlertDialog(
                                                  title: const Text('Peer ID'),
                                                  content: SelectableText(
                                                      profile.peerId),
                                                  actions: <Widget>[
                                                    TextButton(
                                                        onPressed: () {
                                                          Clipboard.setData(
                                                              ClipboardData(
                                                                  text: profile
                                                                      .peerId));
                                                          Navigator.of(context)
                                                              .pop();
                                                        },
                                                        child:
                                                            const Text('Copy')),
                                                    TextButton(
                                                      onPressed: () {
                                                        Navigator.of(context)
                                                            .pop();
                                                      },
                                                      child:
                                                          const Text('Close'),
                                                    ),
                                                  ],
                                                  shape: RoundedRectangleBorder(
                                                    borderRadius:
                                                        BorderRadius.circular(
                                                            10),
                                                  ),
                                                );
                                              },
                                            );
                                          }),
                                    );
                                  });
                            })),
                  ],
                );
              }),
            )),
      ),
    );
  }
}

class AudioLevel extends StatelessWidget {
  final double level;
  final int numRectangles;
  static const Color grey = Color(0xFF80848e);
  static const Color quietColor = Colors.green;
  static const Color mediumColor = Colors.yellow;
  static const Color loudColor = Colors.red;

  const AudioLevel(
      {super.key, required this.level, required this.numRectangles});

  /// Calculates a color for the given index
  Color getColor(int index, int maxIndex) {
    // calculate the fraction of the index in relation to the max index
    double fraction = index / maxIndex;

    // determine the color based on the fraction
    if (fraction <= 0.5) {
      // scale fraction to [0, 1] for the first half
      double scaledFraction = fraction * 2;
      return Color.lerp(quietColor, mediumColor, scaledFraction)!;
    } else {
      // scale fraction to [0, 1] for the second half
      double scaledFraction = (fraction - 0.5) * 2;
      return Color.lerp(mediumColor, loudColor, scaledFraction)!;
    }
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        double threshold = level * numRectangles;

        // generate the rectangles
        List<Widget> rectangles = List.generate(numRectangles, (index) {
          return Container(
            width: 8,
            height: 25,
            margin: const EdgeInsets.only(right: 5),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(5),
              color: index >= threshold
                  ? grey
                  : getColor(index, numRectangles - 1),
            ),
          );
        });

        return Row(
          children: rectangles,
        );
      },
    );
  }
}
