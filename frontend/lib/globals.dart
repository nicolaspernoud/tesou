import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart' show kDebugMode, kIsWeb;
import 'package:tesou/models/preferences.dart';
import 'models/crud.dart';
import 'models/position.dart';

class App {
  Preferences prefs = Preferences();
  bool _initialized = false;
  final PositionQueue _positions = PositionQueue();
  App._privateConstructor();

  static final App _instance = App._privateConstructor();

  factory App() {
    return _instance;
  }

  bool get hasToken {
    return prefs.token != "";
  }

  bool get sportMode {
    return prefs.sportsMode;
  }

  set sportMode(bool v) {
    prefs.sportMode = v;
  }

  Future<void> log(String v) async {
    if (!_initialized) {
      await init();
    }
    await prefs.addToLog(v);
  }

  List<String> getLog() {
    return prefs.log;
  }

  void clearLog() {
    prefs.clearLog();
  }

  Future init() async {
    if (kIsWeb || !Platform.environment.containsKey('FLUTTER_TEST')) {
      await prefs.read();
      // Reload the position queue from cache
      await _positions.read();
    }
    _initialized = true;
  }

  Future<Position> pushPosition(Position pos) async {
    if (!_initialized) {
      await init();
    }
    // Add the position at the start of the queue
    _positions.push(pos);
    // Filter the queue to discard positions that are too old
    _positions.removeWhere(
      (element) => element.time.isBefore(
        DateTime.now().subtract(const Duration(hours: 1)),
      ),
    );
    // Try to push all positions to the server
    try {
      var result = await APICrud<Position>().createMany(_positions.queue);
      _positions.clear();
      // We return the last position pushed
      return result;
    } on Exception catch (e) {
      await App().log(e.toString());
      rethrow;
    }
  }
}

class PositionQueue extends LocalFilePersister {
  PositionQueue() : super("positions");

  List<Position> queue = [];

  Future<void> push(Position p) async {
    queue.add(p);
    await write();
  }

  Future<void> clear() async {
    queue = [];
    await write();
  }

  Future<void> removeWhere(bool Function(Position) predicate) async {
    queue.removeWhere(predicate);
    await write();
  }

  @override
  void fromJson(String source) {
    try {
      queue = List<Position>.from(
        json.decode(source)["positions"].map((data) => Position.fromJson(data)),
      );
    } catch (e) {
      if (kDebugMode) {
        print("e");
      }
    }
  }

  @override
  String toJson() {
    Map<String, dynamic> posMap = {'positions': queue};
    return jsonEncode(posMap);
  }
}
