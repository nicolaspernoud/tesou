import 'dart:async';
import 'package:tesou/globals.dart';
import 'package:tesou/models/position.dart';
import 'package:http/http.dart' as http;
import 'package:aosp_location/aosp_location.dart';

Future<bool> getPositionAndPushToServer(bool sportMode) async {
  await App().init();

  await App().log("Getting position...");
  try {
    final String position = await AospLocation.instance.getPositionFromGPS;
    // Fast mock for Debugging
    //final String position ='${45.50 + Random().nextDouble() / 50}:${4.835659 + Random().nextDouble() / 50}:50';
    final positions = position.split(":");
    // Push a position to the position queue
    var pos = Position(
        id: 0,
        userId: App().prefs.userId,
        latitude: double.parse(positions[0]),
        longitude: double.parse(positions[1]),
        batteryLevel: int.parse(positions[2]),
        source: "GPS",
        time: DateTime.now(),
        sportMode: sportMode);
    await App().log("Got position from GPS");
    return await App().pushPosition(pos);
  } on Exception catch (e) {
    await App().log(e.toString());
    var base = "${App().prefs.hostname}/api/positions/cid";
    var token = App().prefs.token;
    var client = http.Client();
    try {
      final String cellInfoJson = await AospLocation.instance.getCellInfo;
      final response =
          await client.post(Uri.parse('$base/${App().prefs.userId}'),
              headers: <String, String>{
                'Content-Type': 'application/json; charset=UTF-8',
                'Authorization': "Bearer $token"
              },
              body: cellInfoJson);
      if (response.statusCode != 201) {
        throw Exception(response.body.toString());
      }
      await App().log("Got position from Cell Id");
    } on Exception catch (e) {
      await App().log(e.toString());
    }
  }
  return false;
}

Future<Position?> createPositionFromStream(String event) async {
  await App().init();
  try {
    final positions = event.split(":");
    var pos = Position(
        id: 0,
        userId: App().prefs.userId,
        latitude: double.parse(positions[0]),
        longitude: double.parse(positions[1]),
        batteryLevel: int.parse(positions[2]),
        source: "GPS",
        time: DateTime.now(),
        sportMode: true);
    App().pushPosition(pos);
    return pos;
  } catch (e) {
    await App().log(e.toString());
    return null;
  }
}
