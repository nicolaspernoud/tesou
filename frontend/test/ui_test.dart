import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tesou/globals.dart';
import 'package:tesou/i18n.dart';
import 'package:tesou/main.dart';

Future<void> main() async {
  testWidgets('Basic app opening tests', (WidgetTester tester) async {
    // Initialize configuration
    await App().init();
    // Build our app and trigger a frame
    await tester.pumpWidget(
      const MaterialApp(
        home: MyHomePage(title: 'Tesou!'),
        localizationsDelegates: [
          MyLocalizationsDelegate(),
        ],
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
