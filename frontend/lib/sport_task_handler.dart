import 'dart:async';
import 'package:flutter_foreground_task/flutter_foreground_task.dart';
import 'package:tesou/globals.dart';
import 'package:tesou/models/new_position.dart';
import 'package:geolocator/geolocator.dart';

final LocationSettings locationSettings = LocationSettings(
  accuracy: LocationAccuracy.high,
  distanceFilter: 2,
);

class SportTaskHandler extends TaskHandler {
  StreamSubscription<Position>? _streamSubscription;

  @override
  Future<void> onStart(DateTime timestamp, TaskStarter starter) async {
    await App().log("Starting positions stream...");
    _streamSubscription =
        Geolocator.getPositionStream(locationSettings: locationSettings).listen(
          (Position? position) async {
            if (position != null) {
              await App().log("Got position event from stream");
              var pos = await createPositionFromStream(position);
              await App().log("Got position from stream : $pos");
              // Send data to the main isolate.
              FlutterForegroundTask.sendDataToMain(pos!.toJson());
              await App().log("Sent position to main isolate");
            }
          },
        );
  }

  @override
  void onRepeatEvent(DateTime timestamp) {
    // do not use
  }

  @override
  Future<void> onDestroy(DateTime timestamp, bool isTimeout) async {
    await _streamSubscription?.cancel();
    await FlutterForegroundTask.clearAllData();
  }
}
