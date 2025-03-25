// ignore: avoid_web_libraries_in_flutter
import 'package:web/web.dart';

String? getQueryParameter(String key) {
  var uri = Uri.dataFromString(window.location.href);
  return uri.queryParameters[key];
}

String getOrigin() {
  return window.location.origin;
}
