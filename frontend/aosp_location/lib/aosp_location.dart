import 'dart:async';

import 'package:flutter/services.dart';

class AospLocation {
  static const MethodChannel _channel = MethodChannel('aosp_location');

  static Future<String> get getPositionFromGPS async {
    final String version = await _channel.invokeMethod('getPositionFromGPS');
    return version;
  }

  static Future<String> get getCellInfo async {
    final String version = await _channel.invokeMethod('getCellInfo');
    return version;
  }
}
