import 'dart:async';
import 'package:battery_plus/battery_plus.dart';
import 'package:tesou/globals.dart';
import 'package:tesou/models/position.dart';
import 'package:geolocator/geolocator.dart' as geolocator;

final geolocator.LocationSettings locationSettings =
    geolocator.LocationSettings(
      accuracy: geolocator.LocationAccuracy.high,
      distanceFilter: 10,
    );

Future<bool> getPositionAndPushToServer(bool sportMode) async {
  await App().init();

  await App().log("Getting position...");
  try {
    geolocator.Position position = await geolocator
        .Geolocator.getCurrentPosition(locationSettings: locationSettings);
    var battery = Battery();
    // Push a position to the position queue
    var pos = Position(
      id: 0,
      userId: App().prefs.userId,
      latitude: position.latitude,
      longitude: position.longitude,
      batteryLevel: await battery.batteryLevel,
      source: "GPS",
      time: DateTime.now(),
      sportMode: sportMode,
    );
    await App().log("Got position from GPS");
    return await App().pushPosition(pos);
  } on Exception catch (e) {
    await App().log(e.toString());
  }
  return false;
}

Future<Position?> createPositionFromStream(geolocator.Position position) async {
  await App().init();
  try {
    var battery = Battery();
    var pos = Position(
      id: 0,
      userId: App().prefs.userId,
      latitude: position.latitude,
      longitude: position.longitude,
      batteryLevel: await battery.batteryLevel,
      source: "GPS",
      time: DateTime.now(),
      sportMode: true,
    );
    App().pushPosition(pos);
    return pos;
  } catch (e) {
    await App().log(e.toString());
    return null;
  }
}
