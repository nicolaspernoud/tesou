import 'package:flutter_foreground_task/flutter_foreground_task.dart';
import 'package:tesou/models/new_position.dart';

class NormalTaskHandler extends TaskHandler {
  @override
  void onStart(DateTime timestamp) {}

  @override
  void onRepeatEvent(DateTime timestamp) {
    getPositionAndPushToServer(false);
  }

  @override
  void onDestroy(DateTime timestamp) {
    FlutterForegroundTask.clearAllData();
  }

  @override
  void onNotificationPressed() {
    FlutterForegroundTask.launchApp('/');
  }

}
