import 'package:tesou/models/crud.dart';

class User extends Serialisable {
  String name;
  String surname;

  User({
    required super.id,
    required this.name,
    required this.surname,
  });

  @override
  Map<String, dynamic> toJson() {
    return {
      if (id > 0) 'id': id,
      'name': name,
      'surname': surname,
    };
  }

  factory User.fromJson(Map<String, dynamic> data) {
    return User(
      id: data['id'],
      name: data['name'],
      surname: data['surname'],
    );
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;
    return other is User &&
        other.id == id &&
        other.name == name &&
        other.surname == surname;
  }

  @override
  int get hashCode {
    return Object.hash(id, name, surname);
  }
}
