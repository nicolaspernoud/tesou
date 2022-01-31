import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import 'package:http/http.dart';
import 'package:tesou/models/position.dart';
import 'package:tesou/globals.dart';
import 'user.dart';
import 'mock_api.dart';

dynamic fromJSONbyType(Type t, Map<String, dynamic> map) {
  switch (t) {
    case Position:
      return Position.fromJson(map);
    case User:
      return User.fromJson(map);
  }
}

String routeByType(Type t) {
  switch (t) {
    case Position:
      return "positions";
    case User:
      return "users";
    default:
      return "";
  }
}

abstract class Serialisable {
  Serialisable({
    required this.id,
  });

  fromJson(Map<String, dynamic> json) {}
  int id = 0;
  Map<String, dynamic> toJson();
}

abstract class Crud<T extends Serialisable> {
  create(T val) {}

  readOne(int id) {}

  read([String? queryFilter]) {}

  update(T val) {}

  delete(int id) {}
}

class APICrud<T extends Serialisable> extends Crud<T> {
  late final Client client;

  final String route = routeByType(T);

  String get base => App().prefs.hostname + "/api";
  String get token => App().prefs.token;

  APICrud() {
    if (!kIsWeb && Platform.environment.containsKey('FLUTTER_TEST')) {
      client = MockAPI().client;
    } else {
      client = http.Client();
    }
  }

  @override
  Future<T?> create(T val) async {
    try {
      final response = await client.post(
        Uri.parse('$base/$route'),
        headers: <String, String>{
          'Content-Type': 'application/json; charset=UTF-8',
          'Authorization': "Bearer " + token
        },
        body: jsonEncode(val),
      );
      if (response.statusCode == 201) {
        return fromJSONbyType(T, json.decode(utf8.decode(response.bodyBytes)));
      }
    } on Exception {
      rethrow;
    }
  }

  @override
  Future<T> readOne(int id) async {
    try {
      final response = await client.get(
        Uri.parse('$base/$route/${id.toString()}'),
        headers: <String, String>{
          'Authorization': "Bearer " + token,
          'Content-Type': 'application/json'
        },
      );
      if (response.statusCode == 200) {
        return fromJSONbyType(T, json.decode(utf8.decode(response.bodyBytes)));
      } else {
        throw Exception('Failed to load object');
      }
    } on Exception {
      rethrow;
    }
  }

  @override
  Future<List<T>> read([String? queryFilter]) async {
    try {
      final response = await client.get(
        queryFilter == null
            ? Uri.parse('$base/$route')
            : Uri.parse('$base/$route?$queryFilter'),
        headers: <String, String>{'Authorization': "Bearer " + token},
      );
      if (response.statusCode == 200) {
        final List t = json.decode(utf8.decode(response.bodyBytes));
        final List<T> list = t.map((e) => fromJSONbyType(T, e) as T).toList();
        return list;
      } else {
        throw Exception('Failed to load objects');
      }
    } on Exception {
      rethrow;
    }
  }

  @override
  update(T val) async {
    try {
      final response = await client.put(
        Uri.parse('$base/$route/${val.id}'),
        headers: <String, String>{
          'Content-Type': 'application/json; charset=UTF-8',
          'Authorization': "Bearer " + token
        },
        body: jsonEncode(val),
      );
      if (response.statusCode != 200) {
        throw Exception(response.body.toString());
      }
    } on Exception {
      rethrow;
    }
  }

  @override
  delete(int id) async {
    try {
      final response = await client.delete(
        Uri.parse('$base/$route/$id'),
        headers: <String, String>{'Authorization': "Bearer " + token},
      );
      if (response.statusCode != 200) {
        throw Exception(response.body.toString());
      }
    } on Exception {
      rethrow;
    }
  }
}
