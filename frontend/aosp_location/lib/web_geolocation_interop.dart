import 'dart:js_interop';

@JS('navigator.geolocation')
external Geolocation? get geolocation;

@JS()
@staticInterop
class Geolocation {}

extension GeolocationExtension on Geolocation {
  external void getCurrentPosition(
    JSFunction successCallback, [
    JSFunction? errorCallback,
    PositionOptions? options,
  ]);
}

@JS()
@anonymous
@staticInterop
class PositionOptions {
  external factory PositionOptions();
}

extension PositionOptionsExtension on PositionOptions {
  external set enableHighAccuracy(bool value);
  external set timeout(int value);
  external set maximumAge(int value);
}

@JS()
@staticInterop
class GeolocationPosition {}

extension GeolocationPositionExtension on GeolocationPosition {
  external GeolocationCoordinates get coords;
}

@JS()
@staticInterop
class GeolocationCoordinates {}

extension GeolocationCoordinatesExtension on GeolocationCoordinates {
  external double get latitude;
  external double get longitude;
}
