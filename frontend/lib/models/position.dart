import 'dart:math';
import 'package:latlong2/latlong.dart';

import 'crud.dart';

class Position extends Serialisable {
  int userId;
  double latitude;
  double longitude;
  String source;
  DateTime time;
  int batteryLevel;
  bool sportMode;

  Position({
    required super.id,
    required this.userId,
    required this.latitude,
    required this.longitude,
    required this.source,
    required this.batteryLevel,
    required this.sportMode,
    required this.time,
  });

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
      'time': time.millisecondsSinceEpoch,
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
          : DateTime.now(),
    );
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;
    return other is Position &&
        other.id == id &&
        other.userId == userId &&
        other.latitude == latitude &&
        other.longitude == longitude &&
        other.source == source &&
        other.batteryLevel == batteryLevel &&
        other.sportMode == sportMode &&
        other.time == time;
  }

  @override
  int get hashCode {
    return Object.hash(
      id,
      userId,
      latitude,
      longitude,
      source,
      batteryLevel,
      sportMode,
      time,
    );
  }
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
      previous.latitude,
      previous.longitude,
      p.latitude,
      p.longitude,
    );
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

// Helper function to check proximity to trace (<= 50 meters)
bool isPointCloseToTrace(LatLng point, List<LatLng> tracePoints) {
  const double thresholdMeters = 50.0;
  if (tracePoints.length < 2) return false;

  for (int i = 0; i < tracePoints.length - 1; i++) {
    LatLng a = tracePoints[i];
    LatLng b = tracePoints[i + 1];

    // Distance from point to segment ab
    double distance = _distanceToSegment(point, a, b);
    if (distance <= thresholdMeters) return true;
  }
  return false;
}

// Calculate the shortest distance from point p to segment ab
double _distanceToSegment(LatLng p, LatLng a, LatLng b) {
  // If a == b, just return distance to a
  if (a.latitude == b.latitude && a.longitude == b.longitude) {
    return Haversine.haversine(a.latitude, a.longitude, p.latitude, p.longitude) * 1000;
  }

  // Project point p onto segment ab in latitude/longitude space
  // Treat latitude as y and longitude as x for a rough planar approximation
  // Find t such that the projection falls on the segment ab:
  // t = ((p - a) . (b - a)) / |b - a|^2, clamped to [0, 1]
  double dx = b.longitude - a.longitude;
  double dy = b.latitude - a.latitude;
  double d2 = dx * dx + dy * dy;
  double t = ((p.longitude - a.longitude) * dx + (p.latitude - a.latitude) * dy) / d2;
  t = t.clamp(0.0, 1.0);

  // Compute projected point coordinates
  double projLongitude = a.longitude + t * dx;
  double projLatitude = a.latitude + t * dy;

  // Distance from p to projection using Haversine (in meters)
  return Haversine.haversine(projLatitude, projLongitude, p.latitude, p.longitude) * 1000;
}
