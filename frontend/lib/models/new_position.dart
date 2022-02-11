import 'dart:async';
import 'dart:io';
import 'package:tesou/globals.dart';
import 'package:tesou/models/crud.dart';
import 'package:tesou/models/position.dart';
import 'package:http/http.dart' as http;
import 'package:aosp_location/aosp_location.dart';

Future<void> getPositionAndPushToServer() async {
  await App().init();
  await App().log("Getting position...");
  try {
    final String position = await AospLocation.getPositionFromGPS;
    final positions = position.split(":");
    await APICrud<Position>().create(Position(
        id: 0,
        userId: App().prefs.userId,
        latitude: double.parse(positions[0]),
        longitude: double.parse(positions[1]),
        batteryLevel: int.parse(positions[2]),
        source: "GPS",
        time: DateTime.now()));
    await App().log("Got position from GPS");
  } on HttpException catch (e) {
    await App().log(e.toString());
  } on Exception catch (e) {
    await App().log(e.toString());
    var base = App().prefs.hostname + "/api/positions/cid";
    var token = App().prefs.token;
    var client = http.Client();
    try {
      final String cellInfoJson = await AospLocation.getCellInfo;
      final response =
          await client.post(Uri.parse('$base/${App().prefs.userId}'),
              headers: <String, String>{
                'Content-Type': 'application/json; charset=UTF-8',
                'Authorization': "Bearer " + token
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
}
