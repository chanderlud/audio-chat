# Audio Chat
Audio Chat is a peer to peer chat application focused on privacy and security. It supports encrypted audio calls with encrypted text messages, files, and screen shares in the call.

### Contributing
Contributions are welcome! Please open an issue or pull request if you have any suggestions or bug reports.

### Building RNNoise
Audio Chat supports using RNNoise through [this VST plug in](https://github.com/werman/noise-suppression-for-voice) that can be compiled for Windows, Linux, and macOS. The Windows [release](https://github.com/chanderlud/audio-chat/releases) of Audio Chat has the DLL precompiled. For other platforms the plug in must be compiled along with the [cython vst loader](https://github.com/hq9000/cython-vst-loader). The Audio Chat code in this repository restricts use of RNNoise to Windows, so if you compile for another platform you must edit the code a bit.