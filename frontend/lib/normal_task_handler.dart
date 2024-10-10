import 'package:flutter_foreground_task/flutter_foreground_task.dart';
import 'package:tesou/models/new_position.dart';

class NormalTaskHandler extends TaskHandler {
  @override
  Future<void> onStart(DateTime timestamp, TaskStarter starter) async {}

  @override
  void onRepeatEvent(DateTime timestamp) {
    getPositionAndPushToServer(false);
  }

  @override
  Future<void> onDestroy(DateTime timestamp) async {
    FlutterForegroundTask.clearAllData();
  }

  @override
  void onNotificationPressed() {
    FlutterForegroundTask.launchApp('/');
  }
}
