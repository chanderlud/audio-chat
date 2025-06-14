// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.10.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'error.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

// These function are ignored because they are on traits that is not defined in current crate (put an empty `#[frb]` on it to unignore): `clone`, `fmt`

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::RustAutoOpaqueInner<Contact>>
abstract class Contact implements RustOpaqueInterface {
  static Contact fromParts(
          {required String id,
          required String nickname,
          required String peerId}) =>
      RustLib.instance.api.crateApiContactContactFromParts(
          id: id, nickname: nickname, peerId: peerId);

  String id();

  bool idEq({required List<int> id});

  factory Contact({required String nickname, required String peerId}) =>
      RustLib.instance.api
          .crateApiContactContactNew(nickname: nickname, peerId: peerId);

  String nickname();

  String peerId();

  Contact pubClone();

  void setNickname({required String nickname});
}
