import 'package:tesou/models/crud.dart';
import 'package:equatable/equatable.dart';

class User extends Serialisable with EquatableMixin {
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
  List<Object> get props {
    return [id, name, surname];
  }

  @override
  bool get stringify => true;
}
