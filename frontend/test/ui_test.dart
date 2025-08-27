import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tesou/components/home.dart';
import 'package:tesou/globals.dart';
import 'package:tesou/i18n.dart';
import 'package:tesou/models/crud.dart';
import 'package:tesou/models/position.dart';
import 'package:tesou/models/user.dart';

Future<void> _startForegroundTask(int updateRateMinutes) async {}

Future<void> main() async {
  testWidgets('Basic app opening tests', (WidgetTester tester) async {
    // Initialize configuration
    await App().init();
    // Build our app and trigger a frame

    var positionCrud = APICrud<Position>();
    var userCrud = APICrud<User>();
    await tester.pumpWidget(
      MaterialApp(
        home: Home(
          crud: positionCrud,
          title: 'Tesou!',
          usersCrud: userCrud,
          foregroundTaskCommand: _startForegroundTask,
          audioHandler: null,
        ),
        localizationsDelegates: const [MyLocalizationsDelegate()],
      ),
    );

    // Check that the app title is displayed
    expect(find.text('Tesou!'), findsOneWidget);
    await tester.pump();
    // Enter a user token
    await tester.enterText(find.byKey(const Key("tokenField")), 'a_token');
    await tester.tap(find.text("OK"));
    await tester.pumpAndSettle();
    // To print the widget tree :
    //debugDumpApp();
    // Check that we display the user picker
    expect(find.text("John"), findsOneWidget);
    // Check that we diplay the last position info
    expect(find.textContaining("2022-01-19"), findsOneWidget);
  });
}
