import 'dart:async';
import 'dart:js_interop';

import 'aosp_location.dart';
import 'web_geolocation_interop.dart';

AospLocation getAospLocationProvider() => AospLocationWeb();

class AospLocationWeb extends AospLocation {
  @override
  Future<String> get getPositionFromGPS async {
    final geo = geolocation;
    if (geo == null) throw Exception('Geolocation not supported');

    final completer = Completer<GeolocationPosition>();

    geo.getCurrentPosition(
      (JSAny pos) {
        completer.complete(pos as GeolocationPosition);
      }.toJS,
      (JSAny err) {
        completer.completeError(Exception('Failed to get position'));
      }.toJS,
      _buildOptions(),
    );

    final pos = await completer.future;
    final coords = pos.coords;
    return '${coords.latitude}:${coords.longitude}:-1';
  }

  PositionOptions _buildOptions() {
    final options = PositionOptions();
    options.enableHighAccuracy = true;
    options.timeout = 30000;
    options.maximumAge = 0;
    return options;
  }

  @override
  Future<String> get getCellInfo async => throw UnimplementedError();

  @override
  Stream<String> get getPositionStream async* {
    throw UnimplementedError();
  }
}
