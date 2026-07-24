import 'dart:async';
import 'package:tesou/globals.dart';
import 'package:tesou/models/position.dart';
import 'package:http/http.dart' as http;
import 'package:aosp_location/aosp_location.dart';

Future<Position?> getPositionAndPushToServer(bool sportMode) async {
  await App().init();

  await App().log("Getting position...");
  Position? pos;
  try {
    final String position = await AospLocation.instance.getPositionFromGPS;
    // Fast mock for Debugging
    ///final String position = '${45.50 + Random().nextDouble() / 50}:${4.835659 + Random().nextDouble() / 50}:50';
    final positions = position.split(":");
    pos = Position(
      id: 0,
      userId: App().prefs.userId,
      latitude: double.parse(positions[0]),
      longitude: double.parse(positions[1]),
      batteryLevel: int.parse(positions[2]),
      source: "GPS",
      time: DateTime.now(),
      sportMode: sportMode,
    );
    await App().log("Got position from GPS");
  } on Exception catch (gpsError) {
    await App().log("Failed to get GPS location: $gpsError");
    // Fallback to Cell ID
    var base = "${App().prefs.hostname}/api/positions/cid";
    var token = App().prefs.token;
    var client = http.Client();
    try {
      final String cellInfoJson = await AospLocation.instance.getCellInfo;
      final response = await client.post(
        Uri.parse('$base/${App().prefs.userId}'),
        headers: <String, String>{
          'Content-Type': 'application/json; charset=UTF-8',
          'Authorization': "Bearer $token",
        },
        body: cellInfoJson,
      );
      if (response.statusCode != 201) {
        throw Exception(response.body.toString());
      }
      await App().log("Got position from Cell Id");
    } on Exception catch (cellError) {
      await App().log("Failed to get location from Cell Id: $cellError");
    }
  }

  if (pos != null) {
    try {
      return await App().pushPosition(pos);
    } catch (pushError) {
      await App().log("Failed to push GPS position to server: $pushError");
      return pos; // Return local position even if push failed
    }
  }

  return null;
}

Future<Position> createPositionFromStream(String event) async {
  await App().init();
  final positions = event.split(":");
  var pos = Position(
    id: 0,
    userId: App().prefs.userId,
    latitude: double.parse(positions[0]),
    longitude: double.parse(positions[1]),
    batteryLevel: int.parse(positions[2]),
    source: "GPS",
    time: DateTime.now(),
    sportMode: true,
  );
  try {
    return await App().pushPosition(pos);
  } catch (e) {
    await App().log("createPositionFromStream: pushPosition failed: $e");
    // Return the local position even if push to server failed, so it can be shown on the map
    return pos;
  }
}
