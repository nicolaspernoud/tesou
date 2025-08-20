import 'package:tesou/url_parser/url_parser.dart';

export 'url_parser_stub.dart' if (dart.library.html) 'url_parser_web.dart';

class SharedPosition {
  final String shareToken;
  final int shareUserId;

  SharedPosition._({required this.shareToken, required this.shareUserId});

  static SharedPosition? fromUrl() {
    String? codedToken = getQueryParameter("token");
    if (codedToken == null || codedToken.isEmpty) {
      return null;
    }
    String? token = Uri.decodeComponent(codedToken);
    int? id = int.tryParse(getQueryParameter("user") ?? "");
    if (id == null) {
      return null;
    }
    return SharedPosition._(shareToken: token, shareUserId: id);
  }
}
