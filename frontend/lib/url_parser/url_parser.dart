import 'package:tesou/url_parser/url_parser.dart';

export 'url_parser_stub.dart' if (dart.library.html) 'url_parser_web.dart';

class SharedPosition {
  final String shareToken;
  final int shareUserId;

  SharedPosition._({required this.shareToken, required this.shareUserId});

  static SharedPosition? fromUrl() {
    String? codedToken = getQueryParameter("token");
    String? token =
        codedToken != null ? Uri.decodeComponent(codedToken) : null;
    int? id = int.tryParse(getQueryParameter("user") ?? "");
    if (token == null || id == null) {
      return null;
    }
    return SharedPosition._(shareToken: token, shareUserId: id);
  }
}
