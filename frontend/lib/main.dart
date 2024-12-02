import 'dart:io';
import 'dart:ui';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_foreground_task/flutter_foreground_task.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:tesou/components/home.dart';
import 'package:tesou/globals.dart';
import 'package:tesou/i18n.dart';
import 'package:tesou/models/crud.dart';
import 'package:tesou/models/position.dart';
import 'package:tesou/models/user.dart';
import 'package:tesou/normal_task_handler.dart';
import 'package:tesou/sport_task_handler.dart';
import 'package:tesou/url_parser/url_parser.dart';

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
  if (!kIsWeb) FlutterForegroundTask.initCommunicationPort();
  runApp(const MyApp());
}

@pragma('vm:entry-point')
void normalModeCallback() {
  FlutterForegroundTask.setTaskHandler(NormalTaskHandler());
}

@pragma('vm:entry-point')
void sportModeCallback() {
  FlutterForegroundTask.setTaskHandler(SportTaskHandler());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});
  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  var positionCrud = APICrud<Position>();
  var userCrud = APICrud<User>();
  final GlobalKey<HomeState> _homeState = GlobalKey<HomeState>();
  SharedPosition? sharedPosition;

  @override
  void initState() {
    super.initState();
    if (kIsWeb) {
      sharedPosition = SharedPosition.fromUrl();
    } else {
      WidgetsBinding.instance.addPostFrameCallback((_) async {
        await _requestPermissions();
        _initService();
        _startService();
      });
      FlutterForegroundTask.addTaskDataCallback(_onReceiveTaskData);
    }
  }

  @override
  void dispose() {
    FlutterForegroundTask.removeTaskDataCallback(_onReceiveTaskData);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tesou!',
      theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: Colors.green,
          ),
          appBarTheme: AppBarTheme(
            backgroundColor: Colors.green,
            elevation: 4,
            shadowColor: Theme.of(context).shadowColor,
          )),
      home: Home(
          key: _homeState,
          crud: positionCrud,
          title: 'Tesou!',
          usersCrud: userCrud,
          foregroundTaskCommand: _updateService, //_startForegroundTask,
          sharedPosition: sharedPosition),
      localizationsDelegates: const [
        MyLocalizationsDelegate(),
        ...GlobalMaterialLocalizations.delegates,
        GlobalWidgetsLocalizations.delegate,
      ],
      supportedLocales: const [
        Locale('en', ''),
        Locale('fr', ''),
      ],
    );
  }

  Future<void> _onReceiveTaskData(Object position) async {
    if (position is Map<String, dynamic>) {
      await App()
          .log('Received position from stream into main isolate : $position');
      await _homeState.currentState
          ?.addLocalPosition(Position.fromJson(position));
    }
  }

  Future<void> _requestPermissions() async {
    final NotificationPermission notificationPermission =
        await FlutterForegroundTask.checkNotificationPermission();
    if (notificationPermission != NotificationPermission.granted) {
      await FlutterForegroundTask.requestNotificationPermission();
    }
    if (Platform.isAndroid) {
      if (!await FlutterForegroundTask.canDrawOverlays) {
        await FlutterForegroundTask.openSystemAlertWindowSettings();
      }
      if (!await FlutterForegroundTask.isIgnoringBatteryOptimizations) {
        await FlutterForegroundTask.requestIgnoreBatteryOptimization();
      }
    }
  }

  void _initService() {
    FlutterForegroundTask.init(
      androidNotificationOptions: AndroidNotificationOptions(
        channelId: 'tesou',
        channelName: 'tesou',
        channelDescription: 'tesou location service',
        channelImportance: NotificationChannelImportance.LOW,
        priority: NotificationPriority.LOW,
      ),
      iosNotificationOptions: const IOSNotificationOptions(
        showNotification: false,
        playSound: false,
      ),
      foregroundTaskOptions: ForegroundTaskOptions(
        eventAction: repeatDelay(false),
        autoRunOnBoot: true,
        autoRunOnMyPackageReplaced: true,
        allowWakeLock: true,
        allowWifiLock: true,
      ),
    );
  }

  ForegroundTaskEventAction repeatDelay(bool sportMode) {
    return sportMode
        ? ForegroundTaskEventAction.repeat(30 * 60 * 1000)
        : ForegroundTaskEventAction.repeat(5 * 60 * 1000);
  }

  Future<ServiceRequestResult> _startService() async {
    if (await FlutterForegroundTask.isRunningService) {
      return FlutterForegroundTask.restartService();
    } else {
      var locale = PlatformDispatcher.instance.locale;
      return FlutterForegroundTask.startService(
        serviceId: 256,
        notificationTitle: MyLocalizations(locale).tr("tesou_is_running"),
        notificationText: MyLocalizations(locale).tr("tap_to_return_to_app"),
        notificationIcon: const NotificationIcon(
          metaDataName: 'fr.ninico.tesou.NOTIFICATION_ICON',
          backgroundColor: Colors.green,
        ),
        callback: normalModeCallback,
      );
    }
  }

  void _updateService(bool sportMode) {
    FlutterForegroundTask.updateService(
      foregroundTaskOptions: ForegroundTaskOptions(
        eventAction: repeatDelay(sportMode),
      ),
      callback: sportMode ? sportModeCallback : normalModeCallback,
    );
  }
}
