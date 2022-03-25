import 'aosp_location_android.dart'
    if (dart.library.js) 'aosp_location_web.dart';

abstract class AospLocation {
  static AospLocation? _instance;

  static AospLocation get instance {
    _instance ??= getAospLocation();
    return _instance as AospLocation;
  }

  Future<String> get getCellInfo {
    throw UnimplementedError();
  }

  Future<String> get getPositionFromGPS {
    throw UnimplementedError();
  }
}

AospLocation getAospLocation() {
  return getAospLocationProvider();
}
