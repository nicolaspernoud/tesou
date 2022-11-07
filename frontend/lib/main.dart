import 'dart:isolate';
import 'dart:ui' as ui;

import 'package:aosp_location/aosp_location.dart';
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

void normalModeCallback() {
  FlutterForegroundTask.setTaskHandler(NormalTaskHandler());
}

void sportModeCallback() {
  FlutterForegroundTask.setTaskHandler(SportTaskHandler());
}

class NormalTaskHandler extends TaskHandler {
  NormalTaskHandler();
  @override
  Future<void> onStart(DateTime timestamp, SendPort? sendPort) async {}

  @override
  Future<void> onEvent(DateTime timestamp, SendPort? sendPort) async {
    await getPositionAndPushToServer(false);
  }

  @override
  Future<void> onDestroy(DateTime timestamp, SendPort? sendPort) async {
    await FlutterForegroundTask.clearAllData();
  }
}

class SportTaskHandler extends TaskHandler {
  SportTaskHandler();
  StreamSubscription<String>? _streamSubscription;

  @override
  Future<void> onStart(DateTime timestamp, SendPort? sendPort) async {
    await App().log("Starting positions stream...");
    final positionStream = AospLocation.instance.getPositionStream;
    _streamSubscription = positionStream.listen((event) async {
      await App().log("Got position event from stream");
      var pos = await createPositionFromStream(event);
      await App().log("Got position from stream : $pos");
      // Send data to the main isolate.
      sendPort?.send(pos);
      await App().log("Sent position to main isolate");
    });
  }

  @override
  Future<void> onEvent(DateTime timestamp, SendPort? sendPort) async {}

  @override
  Future<void> onDestroy(DateTime timestamp, SendPort? sendPort) async {
    await _streamSubscription?.cancel();
    await FlutterForegroundTask.clearAllData();
  }
}

class MyApp extends StatefulWidget {
  const MyApp({Key? key}) : super(key: key);

  @override
  MyAppState createState() => MyAppState();
}

class MyAppState extends State<MyApp> {
  ReceivePort? _receivePort;
  Future<bool> _startForegroundTask(bool sportMode) async {
    _closeReceivePort();
    await FlutterForegroundTask.stopService();
    FlutterForegroundTask.init(
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
          interval: sportMode ? 30 * 60 * 1000 : 5 * 60 * 1000,
          autoRunOnBoot: true,
          allowWifiLock: false,
        ));
    var locale = ui.window.locale;
    bool reqResult = await FlutterForegroundTask.startService(
      notificationTitle: MyLocalizations(locale).tr("tesou_is_running"),
      notificationText: MyLocalizations(locale).tr("tap_to_return_to_app"),
      callback: sportMode ? sportModeCallback : normalModeCallback,
    );

    ReceivePort? receivePort;
    if (reqResult) {
      receivePort = await FlutterForegroundTask.receivePort;
    }

    return _registerReceivePort(receivePort);
  }

  bool _registerReceivePort(ReceivePort? receivePort) {
    _closeReceivePort();
    if (receivePort != null) {
      _receivePort = receivePort;
      _receivePort?.listen((position) async {
        await App()
            .log('Received position from stream into main isolate : $position');
        if (App().prefs.userId.toString() ==
            _homeState.currentState!.displayedUser) {
          await _homeState.currentState?.panMap();
        }
      });
      return true;
    }
    return false;
  }

  void _closeReceivePort() {
    _receivePort?.close();
    _receivePort = null;
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

  final GlobalKey<HomeState> _homeState = GlobalKey<HomeState>();

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tesou!',
      theme: ThemeData(primarySwatch: Colors.green),
      home: WithForegroundTask(
          child: Home(
        key: _homeState,
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
