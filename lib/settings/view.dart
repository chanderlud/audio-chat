import 'dart:convert';

import 'package:audio_chat/settings/controller.dart';
import 'package:flutter/material.dart';

import '../main.dart';
import '../src/rust/api/audio_chat.dart';
import '../src/rust/api/error.dart';

class SettingsPage extends StatefulWidget {
  final SettingsController controller;
  final AudioChat audioChat;
  final CallStateController callStateController;

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
  final TextEditingController _listenPortInput = TextEditingController();
  final TextEditingController _receivePortInput = TextEditingController();

  @override
  Widget build(BuildContext context) {
    _listenPortInput.text = widget.controller.listenPort.toString();
    _receivePortInput.text = widget.controller.receivePort.toString();

    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
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
                        onEditingComplete: () {
                          var port = int.tryParse(_listenPortInput.text);

                          if (port != null) {
                            if (port < 0 || port > 65535) {
                              _listenPortInput.text =
                                  widget.controller.listenPort.toString();
                            } else {
                              widget.controller.updateListenPort(port);
                              widget.audioChat.setListenPort(port: port);

                              try {
                                widget.audioChat.restartListener();
                              } on DartError catch (e) {
                                // if there is an active call, the listener cannot be restarted
                                showErrorDialog(context, e.message);
                              }
                            }
                          } else {
                            _listenPortInput.text =
                                widget.controller.listenPort.toString();
                          }
                        });
                  }),
              const SizedBox(height: 20),
              TextInput(
                  labelText: 'Receive Port',
                  controller: _receivePortInput,
                  onEditingComplete: () {
                    var port = int.tryParse(_receivePortInput.text);

                    if (port != null) {
                      if (port < 0 || port > 65535) {
                        _receivePortInput.text =
                            widget.controller.receivePort.toString();
                      } else {
                        widget.controller.updateReceivePort(port);
                        widget.audioChat.setReceivePort(port: port);
                      }
                    } else {
                      _receivePortInput.text =
                          widget.controller.receivePort.toString();
                    }
                  }),
            ],
          )),
    );
  }
}
