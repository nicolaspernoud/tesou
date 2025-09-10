import 'dart:async';
import 'package:flutter_foreground_task/flutter_foreground_task.dart';
import 'package:geolocator/geolocator.dart';
import 'package:tesou/globals.dart';
import 'package:tesou/models/new_position.dart';

class LocationTaskHandler extends TaskHandler {
  StreamSubscription<Position>? _streamSubscription;

  @override
  Future<void> onStart(DateTime timestamp, TaskStarter starter) async {}

  @override
  void onRepeatEvent(DateTime timestamp) async {
    final position = await getPositionAndPushToServer(false);
    _applyMode(position!.sportMode);
  }

  @override
  void onNotificationDismissed() {
    FlutterForegroundTask.updateService();
  }

  @override
  void onReceiveData(Object data) {
    if (data is Map<String, dynamic>) {
      if (data['command'] == 'setSportMode') {
        final bool nextMode = data['sportMode'] as bool;
        _applyMode(nextMode);
      }
    }
  }

  void _startStream() {
    _streamSubscription =
        Geolocator.getPositionStream().where((e) => e.accuracy < 10).listen((Position? position) async {
          await App().log("Got position event from stream");
          var pos = await createPositionFromStream(position!);
          await App().log("Got position from stream : $pos");
          // Send data to the main isolate.
          FlutterForegroundTask.sendDataToMain(pos!.toJson());
          await App().log("Sent position to main isolate");
          if (!pos.sportMode) {
            _applyMode(false);
          }
        });
  }

  void _stopStream() {
    _streamSubscription?.cancel();
    _streamSubscription = null;
  }

  void _applyMode(bool sportMode) {
    FlutterForegroundTask.sendDataToMain({
      'type': 'modeChanged',
      'sportMode': sportMode,
    });

    if (sportMode) {
      _startStream();
    } else {
      _stopStream();
    }
  }

  @override
  Future<void> onDestroy(DateTime timestamp, bool isTimeout) async {
    await _streamSubscription?.cancel();
    await FlutterForegroundTask.clearAllData();
  }
}
