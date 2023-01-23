import 'package:flutter/material.dart';
import 'package:tesou/globals.dart';
import 'package:tesou/models/crud.dart';
import 'package:tesou/models/user.dart';

import '../i18n.dart';

class NewEditUser extends StatefulWidget {
  final Crud crud;
  final User user;
  const NewEditUser({Key? key, required this.crud, required this.user})
      : super(key: key);

  @override
  NewEditUserState createState() => NewEditUserState();
}

class NewEditUserState extends State<NewEditUser> {
  final _formKey = GlobalKey<FormState>();

  @override
  Widget build(BuildContext context) {
    // Build a Form widget using the _formKey created above.
    return Scaffold(
        appBar: AppBar(
          title: widget.user.id > 0
              ? Text(tr(context, "edit_user"))
              : Text(tr(context, "new_user")),
          actions: widget.user.id > 0
              ? [
                  IconButton(
                      icon: const Icon(Icons.delete_forever),
                      onPressed: () async {
                        await widget.crud.delete(widget.user.id);
                        if (!mounted) return;
                        Navigator.pop(context);
                        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                            content: Text(tr(context, "user_deleted"))));
                      })
                ]
              : null,
        ),
        body: Padding(
            padding: const EdgeInsets.all(16.0),
            child: Form(
              key: _formKey,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (widget.user.id > 0)
                    Row(
                      children: [
                        Text(tr(context, "active_user")),
                        Checkbox(
                          value: App().prefs.userId == widget.user.id,
                          onChanged: (bool? value) {
                            if (value != null && value) {
                              App().prefs.userId = widget.user.id;
                              setState(() {});
                            }
                          },
                        ),
                      ],
                    ),
                  TextFormField(
                    initialValue: widget.user.name,
                    decoration: InputDecoration(labelText: tr(context, "name")),
                    // The validator receives the text that the user has entered.
                    validator: (value) {
                      if (value == null || value.isEmpty) {
                        return tr(context, "please_enter_some_text");
                      }
                      return null;
                    },
                    onChanged: (value) {
                      widget.user.name = value;
                    },
                  ),
                  TextFormField(
                    initialValue: widget.user.surname,
                    decoration:
                        InputDecoration(labelText: tr(context, "surname")),
                    // The validator receives the text that the user has entered.
                    validator: (value) {
                      if (value == null || value.isEmpty) {
                        return tr(context, "please_enter_some_text");
                      }
                      return null;
                    },
                    onChanged: (value) {
                      widget.user.surname = value;
                    },
                  ),
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 16.0),
                    child: ElevatedButton(
                      onPressed: () async {
                        // Validate returns true if the form is valid, or false otherwise.
                        if (_formKey.currentState!.validate()) {
                          var msg = tr(context, "user_created");
                          try {
                            if (widget.user.id > 0) {
                              await widget.crud.update(widget.user);
                            } else {
                              await widget.crud.create(widget.user);
                            }
                            // Do nothing on TypeError as Create respond with a null id
                          } catch (e) {
                            msg = e.toString();
                          }
                          if (!mounted) return;
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text(msg)),
                          );
                          Navigator.pop(context);
                        }
                      },
                      child: Padding(
                        padding: const EdgeInsets.all(16.0),
                        child: Text(tr(context, "submit")),
                      ),
                    ),
                  ),
                ],
              ),
            )));
  }
}
