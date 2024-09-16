import 'dart:async';

import 'package:aosp_location/aosp_location.dart';
import 'package:flutter_foreground_task/flutter_foreground_task.dart';
import 'package:tesou/globals.dart';
import 'package:tesou/models/new_position.dart';

class SportTaskHandler extends TaskHandler {
  StreamSubscription<String>? _streamSubscription;

  @override
  void onStart(DateTime timestamp) async {
    await App().log("Starting positions stream...");
    final positionStream = AospLocation.instance.getPositionStream;
    _streamSubscription = positionStream.listen((event) async {
      await App().log("Got position event from stream");
      var pos = await createPositionFromStream(event);
      await App().log("Got position from stream : $pos");
      // Send data to the main isolate.
      FlutterForegroundTask.sendDataToMain(pos!.toJson());
      await App().log("Sent position to main isolate");
    });
  }

  @override
  void onRepeatEvent(DateTime timestamp) {
    // do not use
  }

  @override
  Future<void> onDestroy(DateTime timestamp) async {
    await _streamSubscription?.cancel();
    await FlutterForegroundTask.clearAllData();
  }
}
