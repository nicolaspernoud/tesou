// ignore: avoid_web_libraries_in_flutter
import 'dart:html' as html;

String? getQueryParameter(String key) {
  var uri = Uri.dataFromString(html.window.location.href);
  return uri.queryParameters[key];
}

String getOrigin() {
  return html.window.location.origin;
}
