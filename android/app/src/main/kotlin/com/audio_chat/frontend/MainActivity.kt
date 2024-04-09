package com.audio_chat.frontend

import io.flutter.embedding.android.FlutterActivity

class MainActivity: FlutterActivity() {
    init {
        System.loadLibrary("audio_chat")
    }
}
