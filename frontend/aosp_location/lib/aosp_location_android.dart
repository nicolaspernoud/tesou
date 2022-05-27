import 'dart:async';
import 'dart:io';

import 'package:flutter/services.dart';

import 'aosp_location.dart';

AospLocation getAospLocationProvider() => AospLocationAndroid();

class AospLocationAndroid extends AospLocation {
  static const MethodChannel _channel = MethodChannel('aosp_location');
  static const EventChannel messageChannel =
      EventChannel('aosp_location_stream');

  @override
  Future<String> get getPositionFromGPS async {
    if (Platform.isAndroid) {
      final String pos = await _channel.invokeMethod('getPositionFromGPS');
      return pos;
    }
    throw UnimplementedError();
  }

  @override
  Future<String> get getCellInfo async {
    if (Platform.isAndroid) {
      final String pos = await _channel.invokeMethod('getCellInfo');
      return pos;
    }
    throw UnimplementedError();
  }

  @override
  Stream<String> get getPositionStream async* {
    if (Platform.isAndroid) {
      await for (String pos
          in messageChannel.receiveBroadcastStream().map((pos) => pos)) {
        yield pos;
      }
    }
    throw UnimplementedError();
  }
}
