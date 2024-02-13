import 'dart:convert';

import 'package:audio_chat/settings/controller.dart';
import 'package:flutter/material.dart';

import '../main.dart';
import '../src/rust/api/audio_chat.dart';
import '../src/rust/api/error.dart';

class SettingsPage extends StatefulWidget {
  final SettingsController controller;
  final AudioChat audioChat;
  final StateController callStateController;

  const SettingsPage(
      {Key? key,
      required this.controller,
      required this.audioChat,
      required this.callStateController})
      : super(key: key);

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
        showErrorDialog(context, e.message);
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
      body: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            children: [
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
          )),
    );
  }
}
