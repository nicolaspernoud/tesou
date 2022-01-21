// Import the test package and Counter class
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:tesou/models/user.dart';
import 'package:tesou/models/position.dart';

void main() {
  group('Serialization', () {
    test(
        'Converting an User to json an retrieving it should give the same User',
        () async {
      final User c1 = User(id: 1, name: "test name", surname: "test surname");
      final c1Json = jsonEncode(c1.toJson());
      final c2 = User.fromJson(json.decode(c1Json));
      expect(c1, c2);
    });

    test(
        'Converting a Position to json an retrieving it should give the same Position',
        () async {
      final Position i1 = Position(
        id: 1,
        userId: 1,
        latitude: 45.74846,
        longitude: 4.84671,
        source: "GPS",
        time: DateTime.fromMillisecondsSinceEpoch(
            DateTime.now().millisecondsSinceEpoch),
      );
      final a1Json = jsonEncode(i1.toJson());
      final i2 = Position.fromJson(json.decode(a1Json));
      expect(i1, i2);
    });
  });
}
