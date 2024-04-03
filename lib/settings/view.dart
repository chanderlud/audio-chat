import 'dart:convert';

import 'package:audio_chat/settings/controller.dart';
import 'package:audio_chat/src/rust/api/player.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show Clipboard, ClipboardData;

import '../main.dart';
import '../src/rust/api/audio_chat.dart';
import '../src/rust/api/error.dart';

class SettingsPage extends StatelessWidget {
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
  Widget build(BuildContext context) {
    // TODO it would be nice if there was a way to update when the devices change
    var (inputDevices, outputDevices) = audioChat.listDevices();

    // default devices map to null
    inputDevices.insert(0, 'Default');
    outputDevices.insert(0, 'Default');

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
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  ListenableBuilder(
                      listenable: callStateController,
                      builder: (BuildContext context, Widget? child) {
                        String inputInitialSelection;

                        if (controller.inputDevice == null) {
                          inputInitialSelection = 'Default';
                        } else if (inputDevices
                            .contains(controller.inputDevice)) {
                          inputInitialSelection = controller.inputDevice!;
                        } else {
                          inputInitialSelection = 'Default';
                        }

                        String outputInitialSelection;

                        if (controller.outputDevice == null) {
                          outputInitialSelection = 'Default';
                        } else if (outputDevices
                            .contains(controller.outputDevice)) {
                          outputInitialSelection = controller.outputDevice!;
                        } else {
                          outputInitialSelection = 'Default';
                        }

                        return Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            DropdownMenu<String>(
                              width: 310,
                              label: const Text('Input device'),
                              enabled: !callStateController.blockAudioChanges,
                              dropdownMenuEntries: inputDevices
                                  .map<DropdownMenuEntry<String>>((device) {
                                return DropdownMenuEntry(
                                  value: device,
                                  label: device,
                                );
                              }).toList(),
                              onSelected: (String? value) {
                                if (value == 'Default') value = null;
                                controller.updateInputDevice(value);
                                audioChat.setInputDevice(device: value);
                              },
                              initialSelection: inputInitialSelection,
                            ),
                            DropdownMenu<String>(
                              width: 310,
                              label: const Text('Output device'),
                              enabled: !callStateController.blockAudioChanges,
                              dropdownMenuEntries: outputDevices
                                  .map<DropdownMenuEntry<String>>((device) {
                                return DropdownMenuEntry(
                                  value: device,
                                  label: device,
                                );
                              }).toList(),
                              onSelected: (String? value) {
                                if (value == 'Default') value = null;
                                controller.updateOutputDevice(value);
                                audioChat.setOutputDevice(device: value);
                                player.updateOutputDevice(name: value);
                              },
                              initialSelection: outputInitialSelection,
                            ),
                          ],
                        );
                      }),
                  const SizedBox(height: 20),
                  ListenableBuilder(
                      listenable: callStateController,
                      builder: (BuildContext context, Widget? child) {
                        return Row(children: [
                          Button(
                            text: callStateController.inAudioTest
                                ? 'End Test'
                                : 'Sound Test',
                            width: 75,
                            height: 25,
                            disabled: callStateController.isCallActive,
                            onPressed: () async {
                              if (callStateController.inAudioTest) {
                                callStateController.setInAudioTest();
                                audioChat.endCall();
                              } else {
                                callStateController.setInAudioTest();
                                try {
                                  await audioChat.audioTest();
                                } on DartError catch (e) {
                                  if (!context.mounted) return;
                                  showErrorDialog(context,
                                      'Error in Audio Test', e.message);
                                  callStateController.setInAudioTest();
                                }
                              }
                            },
                          ),
                          const SizedBox(width: 20),
                          AudioLevel(level: callStateController.rms)
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
                          listenable: controller,
                          builder: (BuildContext context, Widget? child) {
                            return ListenableBuilder(
                                listenable: callStateController,
                                builder: (BuildContext context, Widget? child) {
                                  return CustomSwitch(
                                      value: controller.useDenoise,
                                      disabled:
                                          callStateController.blockAudioChanges,
                                      onChanged: (use) {
                                        controller.updateUseDenoise(use);
                                        audioChat.setDenoise(denoise: use);
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
                          listenable: controller,
                          builder: (BuildContext context, Widget? child) {
                            return CustomSwitch(
                                value: controller.playCustomRingtones,
                                onChanged: (play) {
                                  controller.updatePlayCustomRingtones(play);
                                  audioChat.setPlayCustomRingtones(play: play);
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
                              controller.updateCustomRingtoneFile(path);
                            } else {
                              controller.updateCustomRingtoneFile(null);
                            }
                          }),
                      ListenableBuilder(
                          listenable: controller,
                          builder: (BuildContext context, Widget? child) {
                            return Text(
                                controller.customRingtoneFile ??
                                    'No file selected',
                                style: const TextStyle(fontSize: 16));
                          }),
                    ],
                  ),
                  const SizedBox(height: 20),
                  const Text('Sound Effect Volume',
                      style: TextStyle(fontSize: 16)),
                  ListenableBuilder(
                      listenable: controller,
                      builder: (BuildContext context, Widget? child) {
                        return Slider(
                            value: controller.soundVolume,
                            onChanged: (value) {
                              controller.updateSoundVolume(value);
                              player.updateOutputVolume(volume: value);
                            },
                            min: -20,
                            max: 20,
                            label:
                                '${controller.soundVolume.toStringAsFixed(2)} db');
                      }),
                  const Divider(height: 30),
                  Button(
                      text: 'Create profile',
                      disabled: false,
                      onPressed: () {
                        controller.createProfile(
                            controller.profiles.length.toString());
                      }),
                  const SizedBox(height: 20),
                  Expanded(
                      child: ListenableBuilder(
                          listenable: controller,
                          builder: (BuildContext context, Widget? child) {
                            return ListView.builder(
                                itemCount: controller.profiles.length,
                                itemBuilder: (context, index) {
                                  Profile profile = controller.profiles.values
                                      .elementAt(index);
                                  String verifyingKey =
                                      base64Encode(profile.verifyingKey);

                                  Widget leading;

                                  if (callStateController.isCallActive ||
                                      controller.activeProfile == profile.id) {
                                    leading = Text(
                                        controller.activeProfile == profile.id
                                            ? 'Active'
                                            : 'Set Active');
                                  } else {
                                    leading = Button(
                                        text: (controller.activeProfile ==
                                                profile.id)
                                            ? 'Active'
                                            : 'Set Active',
                                        width: 75,
                                        height: 25,
                                        disabled: false,
                                        onPressed: () {
                                          controller
                                              .setActiveProfile(profile.id);
                                          audioChat.setSigningKey(
                                              key: profile.signingKey);
                                          audioChat.restartManager();
                                        });
                                  }

                                  return ListTile(
                                    leading: leading,
                                    title: Text(profile.nickname),
                                    trailing: Button(
                                        text: 'View Verifying Key',
                                        disabled: false,
                                        onPressed: () {
                                          showDialog(
                                            context: context,
                                            builder: (BuildContext context) {
                                              return AlertDialog(
                                                title:
                                                    const Text('Verifying Key'),
                                                content: SelectableText(
                                                    verifyingKey),
                                                actions: <Widget>[
                                                  TextButton(
                                                      onPressed: () {
                                                        Clipboard.setData(
                                                            ClipboardData(
                                                                text:
                                                                    verifyingKey));
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
                                                    child: const Text('Close'),
                                                  ),
                                                ],
                                                shape: RoundedRectangleBorder(
                                                  borderRadius:
                                                      BorderRadius.circular(10),
                                                ),
                                              );
                                            },
                                          );
                                        }),
                                  );
                                });
                          })),
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
