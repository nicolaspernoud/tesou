import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:aosp_location/aosp_location_android.dart';

void main() {
  const MethodChannel channel = MethodChannel('aosp_location');

  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    channel.setMockMethodCallHandler((MethodCall methodCall) async {
      return '42';
    });
  });

  tearDown(() {
    channel.setMockMethodCallHandler(null);
  });

  test('getPositionFromGPS and getCellInfo', () async {
    expect(await AospLocationAndroid().getPositionFromGPS, '42');
    expect(await AospLocationAndroid().getCellInfo, '42');
  });
}
