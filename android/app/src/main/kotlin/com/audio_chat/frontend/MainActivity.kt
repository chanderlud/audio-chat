package com.cflm-studios.telepathy

import io.flutter.embedding.android.FlutterActivity

class MainActivity: FlutterActivity() {
    init {
        System.loadLibrary("telepathy")
    }
}
