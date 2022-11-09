import 'dart:math';

import 'package:equatable/equatable.dart';

import 'crud.dart';

class Position extends Serialisable with EquatableMixin {
  int userId;
  double latitude;
  double longitude;
  String source;
  DateTime time;
  int batteryLevel;
  bool sportMode;

  Position(
      {required id,
      required this.userId,
      required this.latitude,
      required this.longitude,
      required this.source,
      required this.batteryLevel,
      required this.sportMode,
      required this.time})
      : super(id: id);

  @override
  Map<String, dynamic> toJson() {
    return {
      if (id > 0) 'id': id,
      'user_id': userId,
      'latitude': latitude,
      'longitude': longitude,
      'source': source,
      'battery_level': batteryLevel,
      'sport_mode': sportMode,
      'time': time.millisecondsSinceEpoch
    };
  }

  factory Position.fromJson(Map<String, dynamic> json) {
    return Position(
        id: json['id'] ?? 0,
        userId: json['user_id'],
        latitude: json['latitude'],
        longitude: json['longitude'],
        source: json['source'],
        batteryLevel: json['battery_level'],
        sportMode: json['sport_mode'] ?? false,
        time: json['time'] != null
            ? DateTime.fromMillisecondsSinceEpoch(json['time'])
            : DateTime.now());
  }

  @override
  List<Object> get props {
    return [
      id,
      userId,
      latitude,
      longitude,
      source,
      batteryLevel,
      sportMode,
      time
    ];
  }

  @override
  bool get stringify => true;
}

double lastRunDuration(List<Position> positions) {
  positions.sort((a, b) => b.time.compareTo(a.time));
  var pos = positions.takeWhile((value) => value.sportMode);
  try {
    return pos.first.time.difference(pos.last.time).inSeconds / 3600;
  } catch (e) {
    return 0.0;
  }
}

double lastRunDistance(List<Position> positions) {
  positions.sort((a, b) => b.time.compareTo(a.time));
  var pos = positions.takeWhile((value) => value.sportMode);
  Position? previous;
  double acc = 0.0;
  for (var p in pos) {
    previous ??= p;
    acc += Haversine.haversine(
        previous.latitude, previous.longitude, p.latitude, p.longitude);
    previous = p;
  }
  return acc;
}

double lastRunSpeed(List<Position> positions) {
  var lastRunTime = lastRunDuration(positions);
  if (lastRunTime == 0.0) return 0.0;
  return lastRunDistance(positions) / lastRunTime;
}

class Haversine {
  static const R = 6372.8; // In kilometers

  static double haversine(double lat1, lon1, lat2, lon2) {
    double dLat = _toRadians(lat2 - lat1);
    double dLon = _toRadians(lon2 - lon1);
    lat1 = _toRadians(lat1);
    lat2 = _toRadians(lat2);
    double a =
        pow(sin(dLat / 2), 2) + pow(sin(dLon / 2), 2) * cos(lat1) * cos(lat2);
    double c = 2 * asin(sqrt(a));
    return R * c;
  }

  static double _toRadians(double degree) {
    return degree * pi / 180;
  }
}
