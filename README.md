## Features

- [Flutter](https://flutter.dev/) UI with Windows, Linux, macOS, iOS, and Android support
- [libp2p](https://libp2p.io/) networking and cryptography, enables p2p networking without port forwarding
- Lossless raw audio and [SEA codec](https://github.com/Daninet/sea-codec) options
- [nnnoiseless](https://github.com/jneem/nnnoiseless) noise suppression
- Built-in text chat with attachments
- Efficient use of CPU and memory resources
- Low end-to-end latency

## Work in Progress

- Screensharing for Windows, macOS, and Linux
- Game overlay for Windows
- Web support (waiting on https://github.com/libp2p/rust-libp2p/pull/5564)

## Planned

- Lossless audio codec support
- Use SEA codec for audio assets & custom ringtones
- Chat rooms (group calls)

## History

- Began as a Python Tkinter app with simple UDP networking and AES cryptography
- Moved to the current Rust audio processing stack with improved stability
- Upgraded from custom networking and cryptography stack to libp2p for improved security, p2p networking without port forwarding, and p2p networking in web browsers

##
![screenshot of audio chat user interface](https://chanchan.dev/static/images/audio-chat.png)
