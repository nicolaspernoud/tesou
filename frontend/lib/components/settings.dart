import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tesou/models/user.dart';
import 'package:tesou/models/crud.dart';
import 'package:flutter/foundation.dart' show kDebugMode, kIsWeb;
import 'package:tesou/globals.dart';
import 'package:tesou/url_parser/url_parser.dart';
import 'package:url_launcher/url_launcher_string.dart';
import '../i18n.dart';
import 'new_user.dart';

class Settings extends StatefulWidget {
  final Crud crud;
  const Settings({super.key, required this.crud});

  @override
  SettingsState createState() => SettingsState();
}

class SettingsState extends State<Settings> {
  late String _logContent = "";
  bool _logEnabled = App().prefs.logEnabled;
  late Future<List<User>> users;
  static const _url = 'https://github.com/nicolaspernoud/tesou/releases/latest';
  @override
  void initState() {
    super.initState();
    users = widget.crud.read();
    refreshLog();
  }

  Future<void> refreshLog() async {
    var lc = App().getLog().join("\n");
    setState(() {
      _logContent = lc;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(tr(context, "settings")),
        actions: [
          IconButton(
            icon: const Icon(Icons.share),
            tooltip: tr(context, "share_my_position"),
            onPressed: () async {
              var token = await getShareToken(App().prefs.userId);
              if (!context.mounted) return;
              Clipboard.setData(
                ClipboardData(
                  text:
                      "${tr(context, "go_to")}\n\n${App().prefs.hostname.isNotEmpty ? App().prefs.hostname : getOrigin()}?token=${Uri.encodeComponent(token)}&user=${App().prefs.userId}",
                ),
              );
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text(tr(context, "share_info_copied_to_clipboard")),
                ),
              );
            },
          ),
        ],
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: ListView(
            children: [
              const SettingsField(),
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 24),
                child: ElevatedButton(
                  onPressed: () async {
                    await canLaunchUrlString(_url)
                        ? await launchUrlString(_url)
                        : throw 'Could not launch $_url';
                  },
                  child: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 12),
                    child: Text(tr(context, "get_latest_release")),
                  ),
                ),
              ),
              ...[
                Center(
                  child: Padding(
                    padding: const EdgeInsets.all(20),
                    child: Text(
                      tr(context, "users"),
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                  ),
                ),
                FutureBuilder<List<User>>(
                  future: users,
                  builder: (context, snapshot) {
                    if (snapshot.hasData) {
                      return Column(
                        children: [
                          ...snapshot.data!.map(
                            (a) => Card(
                              child: InkWell(
                                splashColor: Colors.blue.withAlpha(30),
                                onTap: () {
                                  _editUser(a);
                                },
                                child: Column(
                                  mainAxisSize: MainAxisSize.min,
                                  children: <Widget>[
                                    ListTile(
                                      leading: (App().prefs.userId == a.id)
                                          ? const Icon(
                                              Icons.radio_button_checked,
                                            )
                                          : const Icon(
                                              Icons.radio_button_unchecked,
                                            ),
                                      title: Text(a.name),
                                      subtitle: Text(a.surname),
                                    ),
                                  ],
                                ),
                              ),
                            ),
                          ),
                          Padding(
                            padding: const EdgeInsets.all(16.0),
                            child: IconButton(
                              icon: const Icon(Icons.add),
                              color: Colors.blue,
                              onPressed: () {
                                _editUser(User(id: 0, name: "", surname: ""));
                              },
                            ),
                          ),
                        ],
                      );
                    } else if (snapshot.hasError) {
                      return const Center(child: Text('...'));
                    }
                    // By default, show a loading spinner.
                    return const Center(child: CircularProgressIndicator());
                  },
                ),
                Row(
                  children: [
                    Checkbox(
                      onChanged: (bool? value) {
                        if (value != null) {
                          _logEnabled = value;
                          App().prefs.logEnabled = value;
                        }
                        setState(() {});
                      },
                      value: _logEnabled,
                    ),
                    Text(tr(context, "enable_log")),
                  ],
                ),
                if (_logEnabled) ...[
                  Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      IconButton(
                        icon: const Icon(Icons.clear),
                        color: Colors.black,
                        onPressed: () {
                          App().clearLog();
                          refreshLog();
                        },
                      ),
                      IconButton(
                        icon: const Icon(Icons.refresh_outlined),
                        color: Colors.black,
                        onPressed: () {
                          refreshLog();
                        },
                      ),
                    ],
                  ),
                  TextFormField(
                    key: Key(_logContent),
                    initialValue: _logContent,
                    maxLines: null,
                  ),
                ],
              ],
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _editUser(User u) async {
    await Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (BuildContext context) {
          return NewEditUser(crud: APICrud<User>(), user: u);
        },
      ),
    );
    setState(() {
      users = widget.crud.read();
    });
  }
}

class SettingsField extends StatelessWidget {
  const SettingsField({super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        if (!kIsWeb || kDebugMode)
          TextFormField(
            initialValue: App().prefs.hostname,
            // initialValue: App().prefs.hostname != "" ? App().prefs.hostname : "http://10.0.2.2:8080-",
            decoration: InputDecoration(labelText: tr(context, "hostname")),
            onChanged: (text) {
              App().prefs.hostname = text;
            },
            key: const Key("hostnameField"),
          ),
        const SizedBox(height: 20),
        TextFormField(
          //initialValue: App().prefs.token != "" ? App().prefs.token : "token-",
          initialValue: App().prefs.token,
          decoration: InputDecoration(labelText: tr(context, "token")),
          onChanged: (text) {
            App().prefs.token = text;
          },
          key: const Key("tokenField"),
        ),
      ],
    );
  }
}
