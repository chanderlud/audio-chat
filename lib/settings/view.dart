import 'dart:convert';

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
  final StateController callStateController;
  final SoundPlayer player;

  const SettingsPage(
      {super.key,
      required this.controller,
      required this.audioChat,
      required this.callStateController,
      required this.player});

  @override
  SettingsPageState createState() => SettingsPageState();
}

class SettingsPageState extends State<SettingsPage> {
  late final TextEditingController _listenPortInput;
  late final TextEditingController _receivePortInput;
  late bool _unsavedChanges = false;

  Future<void> onSave() async {
    int? listenPort = validatePort(_listenPortInput.text);
    int? receivePort = validatePort(_receivePortInput.text);

    if (listenPort != null && !widget.callStateController.isCallActive) {
      widget.controller.updateListenPort(listenPort);
      widget.audioChat.setListenPort(port: listenPort);

      try {
        widget.audioChat.restartListener();
      } on DartError catch (e) {
        // if there is an active call, the listener cannot be restarted
        showErrorDialog(context, 'Action blocked', e.message);
      }
    }

    if (receivePort != null) {
      widget.controller.updateReceivePort(receivePort);
      widget.audioChat.setReceivePort(port: receivePort);
    }
  }

  int? validatePort(String port) {
    var portNumber = int.tryParse(port);

    if (portNumber == null) {
      return null;
    } else if (portNumber >= 0 && portNumber <= 65535) {
      return portNumber;
    } else {
      return null;
    }
  }

  @override
  void initState() {
    super.initState();
    _listenPortInput =
        TextEditingController(text: widget.controller.listenPort.toString());
    _receivePortInput =
        TextEditingController(text: widget.controller.receivePort.toString());
  }

  @override
  void dispose() {
    _listenPortInput.dispose();
    _receivePortInput.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // TODO it would be nice if there was a way to update when the devices change
    var (inputDevices, outputDevices) = widget.audioChat.listDevices();

    // default devices map to null
    inputDevices.insert(0, 'Default');
    outputDevices.insert(0, 'Default');

    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () {
            if (_unsavedChanges) {
              showDialog(
                context: context,
                builder: (BuildContext context) {
                  return AlertDialog(
                    title: const Text('Unsaved Changes'),
                    content: const Text(
                        'You have unsaved changes. Are you sure you want to leave?'),
                    actions: <Widget>[
                      TextButton(
                        onPressed: () {
                          Navigator.of(context).pop();
                        },
                        child: const Text('Cancel'),
                      ),
                      TextButton(
                        onPressed: () {
                          Navigator.of(context).pop();
                        },
                        child: const Text('Leave'),
                      ),
                    ],
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                  );
                },
              );
            } else {
              Navigator.of(context).pop();
            }
          },
        ),
        backgroundColor: Theme.of(context).colorScheme.background,
      ),
      body: Center(
        child: Padding(
            padding: const EdgeInsets.all(20),
            child: SizedBox(
              width: 650,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  ListenableBuilder(
                      listenable: widget.callStateController,
                      builder: (BuildContext context, Widget? child) {
                        return Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            DropdownMenu<String>(
                              width: 310,
                              label: const Text('Input device'),
                              enabled:
                                  !widget.callStateController.blockAudioChanges,
                              dropdownMenuEntries: inputDevices
                                  .map<DropdownMenuEntry<String>>((device) {
                                return DropdownMenuEntry(
                                  value: device,
                                  label: device,
                                );
                              }).toList(),
                              onSelected: (String? value) {
                                if (value == 'Default') value = null;
                                // TODO set audioPlayer input device
                                widget.controller.updateInputDevice(value);
                                widget.audioChat.setInputDevice(device: value);
                              },
                              initialSelection:
                                  widget.controller.inputDevice ?? 'Default',
                            ),
                            DropdownMenu<String>(
                              width: 310,
                              label: const Text('Output device'),
                              enabled:
                                  !widget.callStateController.blockAudioChanges,
                              dropdownMenuEntries: outputDevices
                                  .map<DropdownMenuEntry<String>>((device) {
                                return DropdownMenuEntry(
                                  value: device,
                                  label: device,
                                );
                              }).toList(),
                              onSelected: (String? value) {
                                if (value == 'Default') value = null;
                                // TODO set audioPlayer output device
                                widget.controller.updateOutputDevice(value);
                                widget.audioChat.setOutputDevice(device: value);
                              },
                              initialSelection:
                                  widget.controller.outputDevice ?? 'Default',
                            ),
                          ],
                        );
                      }),
                  const SizedBox(height: 20),
                  ListenableBuilder(
                      listenable: widget.callStateController,
                      builder: (BuildContext context, Widget? child) {
                        return Row(children: [
                          Button(
                            text: widget.callStateController.inAudioTest
                                ? 'End Test'
                                : 'Sound Test',
                            width: 75,
                            onPressed: () async {
                              if (widget.callStateController.isCallActive) {
                                showErrorDialog(context, 'Action blocked',
                                    'Cannot test audio while a call is active');
                                return;
                              }

                              if (widget.callStateController.inAudioTest) {
                                widget.callStateController.setInAudioTest();
                                widget.audioChat.endCall();
                              } else {
                                widget.callStateController.setInAudioTest();
                                try {
                                  await widget.audioChat.audioTest();
                                } on DartError catch (e) {
                                  if (!context.mounted) return;
                                  showErrorDialog(context,
                                      'Error in Audio Test', e.message);
                                  widget.callStateController.setInAudioTest();
                                }
                              }
                            },
                          ),
                          const SizedBox(width: 20),
                          AudioLevel(level: widget.callStateController.rms)
                        ]);
                      }),
                  const SizedBox(height: 10),
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
                                listenable: widget.callStateController,
                                builder: (BuildContext context, Widget? child) {
                                  return CustomSwitch(
                                      value: widget.controller.useDenoise,
                                      disabled: widget.callStateController
                                          .blockAudioChanges,
                                      onChanged: (use) {
                                        widget.controller.updateUseDenoise(use);
                                        widget.audioChat
                                            .setDenoise(denoise: use);
                                      });
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
                  Button(
                      text: 'View Verifying Key',
                      onPressed: () {
                        showDialog(
                          context: context,
                          builder: (BuildContext context) {
                            return AlertDialog(
                              title: const Text('Verifying Key'),
                              content: SelectableText(
                                  base64Encode(widget.controller.verifyingKey)),
                              actions: <Widget>[
                                TextButton(
                                    onPressed: () {
                                      Clipboard.setData(ClipboardData(
                                          text: base64Encode(
                                              widget.controller.verifyingKey)));
                                      Navigator.of(context).pop();
                                    },
                                    child: const Text('Copy')),
                                TextButton(
                                  onPressed: () {
                                    Navigator.of(context).pop();
                                  },
                                  child: const Text('Close'),
                                ),
                              ],
                              shape: RoundedRectangleBorder(
                                borderRadius: BorderRadius.circular(10),
                              ),
                            );
                          },
                        );
                      }),
                  const SizedBox(height: 20),
                  ListenableBuilder(
                      listenable: widget.callStateController,
                      builder: (BuildContext context, Widget? child) {
                        return TextInput(
                            // disable ths input when a call is active
                            enabled: !widget.callStateController.isCallActive,
                            labelText: 'Listen Port',
                            controller: _listenPortInput,
                            onChanged: (_) {
                              setState(() {
                                _unsavedChanges = true;
                              });
                            });
                      }),
                  const SizedBox(height: 20),
                  TextInput(
                      labelText: 'Receive Port',
                      controller: _receivePortInput,
                      onChanged: (_) {
                        setState(() {
                          _unsavedChanges = true;
                        });
                      }),
                  const Spacer(),
                  Button(
                      text: 'Save Changes',
                      onPressed: () async {
                        await onSave();
                        setState(() {
                          _unsavedChanges = false;
                        });
                      }),
                ],
              ),
            )),
      ),
    );
  }
}

class AudioLevel extends StatelessWidget {
  final double level;
  static const int numRectangles = 39;
  static const Color grey = Color(0xFF80848e);
  static const Color quietColor = Colors.green;
  static const Color mediumColor = Colors.yellow;
  static const Color loudColor = Colors.red;

  const AudioLevel({super.key, required this.level});

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
