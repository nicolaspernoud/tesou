import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart' show kDebugMode, kIsWeb;
import 'package:tesou/models/preferences.dart';
import 'models/crud.dart';
import 'models/position.dart';

class App {
  late Preferences prefs;
  bool _initialized = false;
  late PositionQueue _positions;
  App._privateConstructor();

  static final App _instance = App._privateConstructor();

  factory App() {
    return _instance;
  }

  bool get hasToken {
    return prefs.token != "";
  }

  log(String v) async {
    if (!_initialized) {
      await init();
    }
    await prefs.addToLog(v);
  }

  getLog() {
    return prefs.log;
  }

  clearLog() {
    prefs.clearLog();
  }

  Future init() async {
    prefs = Preferences();
    _positions = PositionQueue();
    if (kIsWeb || !Platform.environment.containsKey('FLUTTER_TEST')) {
      await prefs.read();
      // Reload the position queue from cache
      await _positions.read();
    }
    _initialized = true;
  }

  Future<bool> pushPosition(Position pos) async {
    if (!_initialized) {
      await init();
    }
    // Add the position to the queue
    _positions.push(pos);
    // Try to push all positions to the server
    try {
      List<Position> posToRemove = [];
      for (Position pos in _positions.queue) {
        await APICrud<Position>().create(pos);
        posToRemove.add(pos);
      }
      _positions.remove(posToRemove);
      return true;
    } on Exception catch (e) {
      await App().log(e.toString());
    }
    return false;
  }
}

class PositionQueue extends LocalFilePersister {
  PositionQueue() : super("positions");

  List<Position> queue = [];

  push(Position p) async {
    queue.add(p);
    await write();
  }

  remove(List<Position> pos) async {
    queue.removeWhere((e) => pos.contains(e));
    await write();
  }

  @override
  fromJson(String source) {
    try {
      queue = List<Position>.from(json
          .decode(source)["positions"]
          .map((data) => Position.fromJson(data)));
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
