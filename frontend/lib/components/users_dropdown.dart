import 'package:flutter/material.dart';
import 'package:tesou/i18n.dart';
import 'package:tesou/models/user.dart';

typedef IntCallback = void Function(int val);

class UsersDropdown extends StatefulWidget {
  final IntCallback callback;
  final Future<List<User>> users;
  final int initialIndex;
  const UsersDropdown({
    super.key,
    required this.users,
    required this.callback,
    required this.initialIndex,
  });

  @override
  UsersDropdownState createState() => UsersDropdownState();
}

class UsersDropdownState extends State<UsersDropdown> {
  late int _index;

  @override
  void initState() {
    super.initState();
    _index = widget.initialIndex;
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<User>>(
        future: widget.users,
        builder: (context, snapshot) {
          Widget child;
          if (snapshot.hasData && snapshot.data!.isNotEmpty) {
            // Check that index exists
            var minID = snapshot.data!.first.id;
            var indexExists = false;
            for (final e in snapshot.data!) {
              if (e.id < minID) minID = e.id;
              if (_index == e.id) {
                indexExists = true;
                break;
              }
            }
            // If index does not exists, switch to the smallest that does
            if (!indexExists) {
              _index = minID;
              // Delay to allow for building interface state
              Future.delayed(Duration.zero, () {
                widget.callback(_index);
              });
            }
            child = Row(
              children: [
                Text(tr(context, "user")),
                const SizedBox(
                  width: 8,
                ),
                DropdownButton<int>(
                  value: _index,
                  items: snapshot.data!.map((a) {
                    return DropdownMenuItem<int>(
                      value: a.id,
                      child: Text(
                        a.name,
                        overflow: TextOverflow.ellipsis,
                      ),
                    );
                  }).toList(),
                  onChanged: (value) {
                    setState(() {
                      _index = value!;
                    });
                    widget.callback(value!);
                  },
                ),
              ],
            );
          } else {
            child = Padding(
              padding: const EdgeInsets.all(16.0),
              child: snapshot.hasError
                  ? const Text("...")
                  : Text(tr(context, "no_users")),
            );
          }
          return AnimatedSwitcher(
            duration: const Duration(milliseconds: 300),
            child: child,
          );
        });
  }
}
