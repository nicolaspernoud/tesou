import 'dart:isolate';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_foreground_task/flutter_foreground_task.dart';
import 'dart:async';

import 'package:tesou/globals.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:tesou/models/new_position.dart';
import 'package:tesou/models/user.dart';
import 'components/positions.dart';
import 'i18n.dart';
import 'models/crud.dart';
import 'models/position.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:flutter/foundation.dart' show kIsWeb;

const updateRateMinutes = 10;

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

void startCallback() {
  FlutterForegroundTask.setTaskHandler(LocationTaskHandler());
}

class LocationTaskHandler extends TaskHandler {
  @override
  Future<void> onStart(DateTime timestamp, SendPort? sendPort) async {}

  @override
  Future<void> onEvent(DateTime timestamp, SendPort? sendPort) async {
    await getPositionAndPushToServer();
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

  Future<void> _startForegroundTask() async {
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
      foregroundTaskOptions: const ForegroundTaskOptions(
        interval: updateRateMinutes * 60 * 1000,
        autoRunOnBoot: true,
        allowWifiLock: false,
      ),
      printDevLog: false,
    );
    var locale = ui.window.locale;
    if (await FlutterForegroundTask.isRunningService) {
      _receivePort = await FlutterForegroundTask.restartService();
    } else {
      _receivePort = await FlutterForegroundTask.startService(
        notificationTitle: MyLocalizations(locale).tr("tesou_is_running"),
        notificationText: MyLocalizations(locale).tr("tap_to_return_to_app"),
        callback: startCallback,
      );
    }
  }

  @override
  initState() {
    super.initState();
    if (!kIsWeb) {
      _startForegroundTask();
    }
  }

  @override
  void dispose() {
    _receivePort?.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tesou!',
      theme: ThemeData(primarySwatch: Colors.green),
      home: const WithForegroundTask(child: MyHomePage(title: 'Tesou!')),
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

class MyHomePage extends StatefulWidget {
  const MyHomePage({Key? key, required this.title}) : super(key: key);
  final String title;
  @override
  _MyHomePageState createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  var positionCrud = APICrud<Position>();
  var userCrud = APICrud<User>();
  @override
  Widget build(BuildContext context) {
    return Positions(
      crud: positionCrud,
      title: widget.title,
      usersCrud: userCrud,
    );
  }
}
