import 'dart:isolate';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_foreground_task/flutter_foreground_task.dart';
import 'dart:async';

import 'package:tesou/globals.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:tesou/models/new_position.dart';
import 'package:tesou/models/user.dart';
import 'components/home.dart';
import 'i18n.dart';
import 'models/crud.dart';
import 'models/position.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:flutter/foundation.dart' show kIsWeb;

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await App().init();
  if (!kIsWeb) {
    while (!(await Permission.location.status.isGranted) &&
        !(await Permission.locationAlways.status.isGranted) &&
        !(await Permission.phone.status.isGranted)) {
      await [
        Permission.location,
        Permission.phone,
      ].request();
      await Permission.locationAlways.request();
    }
  }
  runApp(const MyApp());
}

void walkingModeCallback() {
  FlutterForegroundTask.setTaskHandler(LocationTaskHandler(runningMode: false));
}

void runningModeCallback() {
  FlutterForegroundTask.setTaskHandler(LocationTaskHandler(runningMode: true));
}

class LocationTaskHandler extends TaskHandler {
  final bool runningMode;

  LocationTaskHandler({required this.runningMode});
  @override
  Future<void> onStart(DateTime timestamp, SendPort? sendPort) async {}

  @override
  Future<void> onEvent(DateTime timestamp, SendPort? sendPort) async {
    await getPositionAndPushToServer(runningMode);
  }

  @override
  Future<void> onDestroy(DateTime timestamp) async {
    await FlutterForegroundTask.clearAllData();
  }
}

class MyApp extends StatefulWidget {
  const MyApp({Key? key}) : super(key: key);

  @override
  _MyAppState createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  ReceivePort? _receivePort;
  Future<void> _startForegroundTask(bool runningMode) async {
    _receivePort?.close();
    await FlutterForegroundTask.stopService();
    await FlutterForegroundTask.init(
      androidNotificationOptions: AndroidNotificationOptions(
          channelId: 'tesou',
          channelName: 'tesou',
          channelDescription: 'tesou location service',
          iconData: const NotificationIconData(
            resType: ResourceType.mipmap,
            resPrefix: ResourcePrefix.ic,
            name: 'notification',
          )),
      iosNotificationOptions: const IOSNotificationOptions(
        showNotification: true,
        playSound: false,
      ),
      foregroundTaskOptions: ForegroundTaskOptions(
        interval: runningMode ? 20 * 1000 : 5 * 60 * 1000,
        autoRunOnBoot: true,
        allowWifiLock: false,
      ),
      printDevLog: false,
    );
    var locale = ui.window.locale;
    _receivePort = await FlutterForegroundTask.startService(
      notificationTitle: MyLocalizations(locale).tr("tesou_is_running"),
      notificationText: MyLocalizations(locale).tr("tap_to_return_to_app"),
      callback: runningMode ? runningModeCallback : walkingModeCallback,
    );
  }

  @override
  initState() {
    super.initState();
    if (!kIsWeb) {
      _startForegroundTask(false);
    }
  }

  @override
  void dispose() {
    _receivePort?.close();
    super.dispose();
  }

  var positionCrud = APICrud<Position>();
  var userCrud = APICrud<User>();

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tesou!',
      theme: ThemeData(primarySwatch: Colors.green),
      home: WithForegroundTask(
          child: Home(
        crud: positionCrud,
        title: 'Tesou!',
        usersCrud: userCrud,
        foregroundTaskCommand: _startForegroundTask,
      )),
      localizationsDelegates: const [
        MyLocalizationsDelegate(),
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ],
      supportedLocales: const [
        Locale('en', ''),
        Locale('fr', ''),
      ],
    );
  }
}
