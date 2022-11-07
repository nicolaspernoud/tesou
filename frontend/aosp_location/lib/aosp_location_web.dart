import 'dart:async';
// ignore: avoid_web_libraries_in_flutter
import 'dart:html' as html;

import 'aosp_location.dart';

AospLocation getAospLocationProvider() => AospLocationWeb();

class AospLocationWeb extends AospLocation {
  @override
  Future<String> get getPositionFromGPS async {
    html.Geolocation geolocation = html.window.navigator.geolocation;
    try {
      final geoPosition = await geolocation.getCurrentPosition(
        enableHighAccuracy: true,
        timeout: const Duration(seconds: 30),
      );
      //final html.BatteryManager batteryManager = await html.window.navigator.getBattery();
      return '${geoPosition.coords!.latitude.toString()}:${geoPosition.coords!.longitude.toString()}:-1';
    } on Exception {
      rethrow;
    }
  }

  @override
  Future<String> get getCellInfo async {
    throw UnimplementedError();
  }

  @override
  Stream<String> get getPositionStream async* {
    throw UnimplementedError();
  }
}
