import 'package:equatable/equatable.dart';

import 'crud.dart';

class Position extends Serialisable with EquatableMixin {
  int userId;
  double latitude;
  double longitude;
  String source;
  DateTime time;
  int batteryLevel;

  Position(
      {required id,
      required this.userId,
      required this.latitude,
      required this.longitude,
      required this.source,
      required this.batteryLevel,
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
      'time': time.millisecondsSinceEpoch
    };
  }

  factory Position.fromJson(Map<String, dynamic> json) {
    return Position(
        id: json['id'],
        userId: json['user_id'],
        latitude: json['latitude'],
        longitude: json['longitude'],
        source: json['source'],
        batteryLevel: json['battery_level'],
        time: json['time'] != null
            ? DateTime.fromMillisecondsSinceEpoch(json['time'])
            : DateTime.now());
  }

  @override
  List<Object> get props {
    return [id, userId, latitude, longitude, source, batteryLevel, time];
  }

  @override
  bool get stringify => true;
}
