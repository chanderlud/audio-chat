// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.10.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

// These functions are ignored because they are not marked as `pub`: `_disable`, `_enable`, `_hide`, `_move_overlay`, `_show`, `controller`, `redraw`, `start_overlay`
// These function are ignored because they are on traits that is not defined in current crate (put an empty `#[frb]` on it to unignore): `clone`

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<Overlay>>
abstract class Overlay implements RustOpaqueInterface {
  /// disable the overlay
  Future<void> disable();

  /// enable the overlay
  Future<void> enable();

  /// hide the overlay window irrespective of platform
  Future<void> hide_();

  /// move and resize the overlay window
  Future<void> moveOverlay(
      {required int x,
      required int y,
      required int width,
      required int height});

  // HINT: Make it `#[frb(sync)]` to let it become the default constructor of Dart class.
  static Future<Overlay> newInstance(
          {required bool enabled,
          required int x,
          required int y,
          required int width,
          required int height,
          required int fontHeight,
          required int backgroundColor,
          required int fontColor}) =>
      RustLib.instance.api.crateApiOverlayOverlayOverlayNew(
          enabled: enabled,
          x: x,
          y: y,
          width: width,
          height: height,
          fontHeight: fontHeight,
          backgroundColor: backgroundColor,
          fontColor: fontColor);

  /// access the screen resolution for overlay positioning in the front end
  (int, int) screenResolution();

  /// change the background color of the overlay
  Future<void> setBackgroundColor({required int backgroundColor});

  /// change the font color of the overlay
  Future<void> setFontColor({required int fontColor});

  /// change the font height (size) of the overlay
  Future<void> setFontHeight({required int height});

  /// show the overlay window irrespective of platform
  Future<void> show_();
}
