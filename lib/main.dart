import 'package:audio_chat/src/rust/api/audio_chat.dart';
import 'package:audio_chat/src/rust/frb_generated.dart';
import 'package:flutter/material.dart';

Future<void> main() async {
  await RustLib.init();

  final ac = await AudioChat.newAudioChat(listenPort: 4700, receivePort: 4701);

  runApp(AudioChatView(audioChat: ac));
}

class AudioChatView extends StatefulWidget {
  final AudioChat audioChat;

  const AudioChatView({super.key, required this.audioChat});

  @override
  State<StatefulWidget> createState() => _AudioChatViewState();
}

class _AudioChatViewState extends State<AudioChatView> {
  final TextEditingController _controllerAddress = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
        home: Scaffold(
      appBar: AppBar(title: const Text('Audio Chat')),
      body: Column(
        children: [
          TextField(
            controller: _controllerAddress,
            decoration: const InputDecoration(
              labelText: 'Address',
              hintText: 'Enter an address',
            ),
          ),
          ElevatedButton(
            onPressed: () async {
              await widget.audioChat.sayHello(address: _controllerAddress.text);
            },
            child: const Text('Connect'),
          ),
          ElevatedButton(
              onPressed: () async {
                await widget.audioChat.endCall();
              },
              child: const Text('Disconnect'))
        ],
      ),
    ));
  }
}
