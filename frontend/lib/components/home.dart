import 'dart:async';

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:tesou/components/users_dropdown.dart';
import 'package:tesou/models/new_position.dart';
import 'package:tesou/models/position.dart';
import 'package:tesou/models/crud.dart';
import 'package:tesou/models/user.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart';
import 'package:tesou/models/preferences.dart';

import 'package:tesou/globals.dart';
import '../i18n.dart';
import 'settings.dart';

const gpsSource = "GPS";

class Home extends StatefulWidget {
  final Crud crud;
  final Crud usersCrud;

  final String title;

  final Function foregroundTaskCommand;

  const Home(
      {Key? key,
      required this.crud,
      required this.usersCrud,
      required this.title,
      required this.foregroundTaskCommand})
      : super(key: key);

  @override
  HomeState createState() => HomeState();
}

class HomeState extends State<Home> with TickerProviderStateMixin {
  late Future<List<Position>> positions;
  late Future<List<User>> users;
  String displayedUser = "1";
  final MapController mapController = MapController();
  bool _sportMode = false;

  @override
  void initState() {
    super.initState();
    if (App().hasToken) {
      positions = widget.crud.read("user_id=$displayedUser");
      users = widget.usersCrud.read();
    } else {
      WidgetsBinding.instance.addPostFrameCallback(openSettings);
    }
  }

  void _animatedMapMove(LatLng destLocation, double destZoom) {
    final latTween = Tween<double>(
        begin: mapController.center.latitude, end: destLocation.latitude);
    final lngTween = Tween<double>(
        begin: mapController.center.longitude, end: destLocation.longitude);
    final zoomTween = Tween<double>(begin: mapController.zoom, end: destZoom);
    var controller = AnimationController(
        duration: const Duration(milliseconds: 500), vsync: this);
    Animation<double> animation =
        CurvedAnimation(parent: controller, curve: Curves.fastOutSlowIn);

    controller.addListener(() {
      mapController.move(
          LatLng(latTween.evaluate(animation), lngTween.evaluate(animation)),
          zoomTween.evaluate(animation));
    });

    animation.addStatusListener((status) {
      if (status == AnimationStatus.completed) {
        controller.dispose();
      } else if (status == AnimationStatus.dismissed) {
        controller.dispose();
      }
    });

    controller.forward();
  }

  void openSettings(_) async {
    await showDialog<String>(
      context: context,
      builder: (BuildContext context) => AlertDialog(
        title: Text(tr(context,"settings")),
        content: const SizedBox(
          height: 150,
          child: SettingsField(),
        ),
        actions: <Widget>[
          TextButton(
            onPressed: () => Navigator.pop(context, 'OK'),
            child: const Text('OK'),
          ),
        ],
      ),
    );
    setState(() {
      hasTokenOrOpenSettings(_);
    });
  }

  void hasTokenOrOpenSettings(_) {
    if (App().hasToken) {
      positions = widget.crud.read("user_id=$displayedUser");
      users = widget.usersCrud.read();
    } else {
      openSettings(_);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(
          title: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Image.asset(
                'assets/icon/icon.png',
                fit: BoxFit.contain,
                height: 30,
              ),
              Text(
                widget.title,
                style: const TextStyle(fontWeight: FontWeight.bold),
              )
            ],
          ),
          actions: [
            IconButton(
                icon: const Icon(Icons.settings),
                onPressed: () async {
                  await Navigator.push(context,
                      MaterialPageRoute<void>(builder: (BuildContext context) {
                    return Settings(crud: APICrud<User>());
                  }));
                  setState(() {
                    hasTokenOrOpenSettings(null);
                  });
                })
          ],
        ),
        body: (App().hasToken)
            ? Center(
                child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: FutureBuilder<List<Position>>(
                  future: positions,
                  builder: (context, snapshot) {
                    Widget child;
                    if (snapshot.hasData && snapshot.data!.isNotEmpty) {
                      var itms = snapshot.data!;
                      itms.sort((a, b) => b.time.compareTo(a.time));
                      child = ClipRRect(
                        borderRadius: BorderRadius.circular(10.0),
                        child: Column(
                          children: [
                            Flexible(
                              child: FlutterMap(
                                mapController: mapController,
                                options: MapOptions(
                                    center: LatLng(itms.elementAt(0).latitude,
                                        itms.elementAt(0).longitude),
                                    zoom: zoom(itms.elementAt(0)),
                                    minZoom: 0,
                                    maxZoom: 18,
                                    enableScrollWheel: true,
                                    interactiveFlags: InteractiveFlag.all &
                                        ~InteractiveFlag.rotate),
                                nonRotatedChildren: <Widget>[
                                  Column(
                                    mainAxisAlignment: MainAxisAlignment.end,
                                    children: [
                                      Row(
                                        mainAxisAlignment:
                                            MainAxisAlignment.center,
                                        children: [
                                          Container(
                                            padding: const EdgeInsets.symmetric(
                                                vertical: 5, horizontal: 10),
                                            decoration: BoxDecoration(
                                                color: Colors.grey.shade50,
                                                borderRadius:
                                                    const BorderRadius.vertical(
                                                  top: Radius.circular(10),
                                                )),
                                            child: _sportMode
                                                ? Text(
                                                    "${formatTime(itms.elementAt(0).time)} - ${itms.elementAt(0).batteryLevel.toString()}% - ${lastRunDistance(itms).toStringAsFixed(1)} km - ${lastRunSpeed(itms).toStringAsFixed(1)} km/h")
                                                : Text(
                                                    "${formatTime(itms.elementAt(0).time)} - ${itms.elementAt(0).batteryLevel.toString()}%",
                                                    style: const TextStyle(
                                                      fontSize: 16,
                                                      color: Colors.black,
                                                    ),
                                                  ),
                                          )
                                        ],
                                      ),
                                    ],
                                  )
                                ],
                                children: [
                                  TileLayer(
                                    urlTemplate:
                                        'https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png',
                                    subdomains: const ['a', 'b', 'c'],
                                  ),
                                  // If the last element comes from GPS, display a marker
                                  itms.elementAt(0).source == gpsSource
                                      ? MarkerLayer(
                                          markers: [
                                            Marker(
                                              width: 80.0,
                                              height: 80.0,
                                              point: LatLng(
                                                  itms.elementAt(0).latitude,
                                                  itms.elementAt(0).longitude),
                                              builder: (ctx) => Icon(
                                                Icons.location_on,
                                                color:
                                                    itms.elementAt(0).sportMode
                                                        ? Colors.pink
                                                        : Colors.blue,
                                                size: 40,
                                              ),
                                            ),
                                          ],
                                        )
                                      // else display a circle
                                      : CircleLayer(circles: [
                                          CircleMarker(
                                              point: LatLng(
                                                  itms.elementAt(0).latitude,
                                                  itms.elementAt(0).longitude),
                                              color:
                                                  Colors.blue.withOpacity(0.3),
                                              useRadiusInMeter: true,
                                              radius: 1000 // 1 km
                                              ),
                                        ]),
                                  // Draw a line with the last positions coming from GPS
                                  PolylineLayer(
                                    polylines: [
                                      Polyline(
                                          points: itms
                                              .where((e) => e.time.isAfter(
                                                  DateTime.now().subtract(
                                                      const Duration(
                                                          hours: 6))))
                                              .where(
                                                  (e) => e.source == gpsSource)
                                              .map((e) => LatLng(
                                                  e.latitude, e.longitude))
                                              .toList(),
                                          strokeWidth: 4.0,
                                          color: Colors.blueAccent),
                                      Polyline(
                                          points: itms
                                              .takeWhile(
                                                  (value) => value.sportMode)
                                              .where(
                                                  (e) => e.source == gpsSource)
                                              .map((e) => LatLng(
                                                  e.latitude, e.longitude))
                                              .toList(),
                                          strokeWidth: 4.0,
                                          color: Colors.pink),
                                    ],
                                  ),
                                ],
                              ),
                            ),
                          ],
                        ),
                      );
                    } else if (snapshot.hasError) {
                      child = Text(
                          tr(context,"try_new_token"));
                    } else {
                      child = const CircularProgressIndicator();
                    }
                    return AnimatedSwitcher(
                      duration: const Duration(milliseconds: 300),
                      child: child,
                    );
                  },
                ),
              ))
            : null,
        bottomNavigationBar: StickyBottomAppBar(
          child: BottomAppBar(
              child: Padding(
            padding: const EdgeInsets.all(8),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                if (!kIsWeb) ...[
                  IconButton(
                      icon: _sportMode
                          ? const CircleAvatar(
                              radius: 40,
                              backgroundColor: Colors.pink,
                              child: Icon(Icons.directions_run,
                                  color: Colors.white),
                            )
                          : const Icon(Icons.directions_walk),
                      onPressed: () async {
                        setState(() {
                          _sportMode = !_sportMode;
                        });
                        widget.foregroundTaskCommand(_sportMode);
                      }),
                  if (!_sportMode)
                    IconButton(
                        icon: const Icon(Icons.my_location),
                        onPressed: () async {
                          try {
                            if (await getPositionAndPushToServer(false)) {
                              await panMap();
                            }
                            // ignore: empty_catches
                          } on Exception {}
                        }),
                ],
                IconButton(
                    icon: const Icon(Icons.refresh),
                    onPressed: () async {
                      panMap();
                    }),
                if (App().hasToken)
                  UsersDropdown(
                    users: users,
                    callback: (val) {
                      displayedUser = val.toString();
                      panMap();
                    },
                    initialIndex: 1,
                  ),
              ],
            ),
          )),
        ));
  }

  Future<void> panMap() async {
    setState(() {
      positions = widget.crud.read("user_id=$displayedUser");
    });
    var itms = await positions;
    itms.sort((a, b) => b.time.compareTo(a.time));
    if (itms.isNotEmpty) {
      _animatedMapMove(
          LatLng(itms.elementAt(0).latitude, itms.elementAt(0).longitude),
          zoom(itms.elementAt(0)));
    }
  }

  double zoom(Position position) {
    if (position.source == gpsSource) {
      return 16;
    }
    return 14;
  }
}

class StickyBottomAppBar extends StatelessWidget {
  final BottomAppBar child;
  const StickyBottomAppBar({Key? key, required this.child}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Transform.translate(
      offset: Offset(0.0, -1 * MediaQuery.of(context).viewInsets.bottom),
      child: child,
    );
  }
}
