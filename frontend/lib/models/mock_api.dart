import 'package:http/http.dart';
import 'package:http/testing.dart';

class MockAPI {
  late final Client client;
  MockAPI() {
    client = MockClient((request) async {
      switch (request.url.toString()) {
        case '/api/positions?user_id=1':
          return Response('''
              [{"id":1,"user_id":1,"latitude":45.74846,"longitude":4.84671,"source":"GPS","battery_level":50,"time":1642620928123},
              {"id":2,"user_id":1,"latitude":45.1911396,"longitude":5.7141747,"source":"GPS","battery_level":50,"time":1642620928123}]
              ''', 200);
        case '/api/users':
          return Response('''
              [{"id":1,"name":"John","surname":"Doe"}]
              ''', 200);
        default:
          return Response('Not Found', 404);
      }
    });
  }
}
