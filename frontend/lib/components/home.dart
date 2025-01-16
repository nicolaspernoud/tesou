import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart' show kDebugMode, kIsWeb;
import 'package:flutter/material.dart';
import 'package:flutter_beep/flutter_beep.dart';
import 'package:flutter_map_geojson/flutter_map_geojson.dart';
import 'package:geoxml/geoxml.dart';
import 'package:tesou/components/speed_gauge.dart';
import 'package:tesou/components/users_dropdown.dart';
import 'package:tesou/models/new_position.dart';
import 'package:tesou/models/position.dart';
import 'package:tesou/models/crud.dart';
import 'package:tesou/models/user.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart';
import 'package:tesou/models/preferences.dart';
import 'package:tesou/url_parser/url_parser.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:file_picker/file_picker.dart';

import 'package:tesou/globals.dart';
import '../i18n.dart';
import 'settings.dart';
import 'package:tesou/models/position.dart' as position;

const gpsSource = "GPS";

class Home extends StatefulWidget {
  final Crud crud;
  final Crud usersCrud;

  final String title;

  final Function foregroundTaskCommand;

  final SharedPosition? sharedPosition;

  const Home(
      {super.key,
      required this.crud,
      required this.usersCrud,
      required this.title,
      required this.foregroundTaskCommand,
      this.sharedPosition});

  @override
  HomeState createState() => HomeState();
}

class HomeState extends State<Home>
    with TickerProviderStateMixin, WidgetsBindingObserver {
  late Future<List<Position>> positions;
  late Future<List<User>> users;
  int displayedUser = 1;
  final MapController mapController = MapController();
  bool _sportMode = false;
  WebSocketChannel? wsChannel;
  final stopwatch = Stopwatch();
  List<Polyline>? trace;
  bool closeToTrace = false;
  int kms = 0;
  GeoJsonParser geoJsonParser = GeoJsonParser(
      defaultPolylineColor: Colors.green, defaultPolylineStroke: 6.0);

  @override
  void initState() {
    super.initState();
    if (widget.sharedPosition != null) {
      displayedUser = widget.sharedPosition!.shareUserId;
      App().prefs.token = widget.sharedPosition!.shareToken;
    }
    if (App().hasToken) {
      getData();
    } else {
      WidgetsBinding.instance.addPostFrameCallback(openSettings);
    }
    WidgetsBinding.instance.addObserver(this);
    stopwatch.start();
  }

  @override
  void dispose() {
    wsChannel?.sink.close();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      getData();
    }
    if (state == AppLifecycleState.paused) {
      wsChannel?.sink.close();
    }
  }

  void _animatedMapMove(LatLng destLocation, double destZoom) {
    final latTween = Tween<double>(
        begin: mapController.camera.center.latitude,
        end: destLocation.latitude);
    final lngTween = Tween<double>(
        begin: mapController.camera.center.longitude,
        end: destLocation.longitude);
    final zoomTween =
        Tween<double>(begin: mapController.camera.zoom, end: destZoom);
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

  void openSettings(Duration? d) async {
    await showDialog<String>(
      context: context,
      builder: (BuildContext context) => AlertDialog(
        title: Text(tr(context, "settings")),
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
      hasTokenOrOpenSettings();
    });
  }

  void hasTokenOrOpenSettings() {
    if (App().hasToken) {
      getData();
    } else {
      openSettings(null);
    }
  }

  void getData() {
    positions = widget.crud.read("user_id=$displayedUser");
    users = widget.usersCrud.read();
    if (App().prefs.userId != displayedUser || kIsWeb) {
      connectWsChannel();
    } else {
      wsChannel?.sink.close();
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
                'assets/icon/icon_foreground_big.png',
                fit: BoxFit.contain,
                height: 30,
              ),
              Text(
                widget.title,
                style: const TextStyle(
                    fontWeight: FontWeight.bold, color: Colors.white),
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
                    hasTokenOrOpenSettings();
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
                                  initialCenter: LatLng(
                                      itms.elementAt(0).latitude,
                                      itms.elementAt(0).longitude),
                                  initialZoom: zoom(itms.elementAt(0)),
                                  minZoom: 0,
                                  maxZoom: 18,
                                  interactionOptions: const InteractionOptions(
                                    flags: InteractiveFlag.all &
                                        ~InteractiveFlag.rotate,
                                  ),
                                  onPositionChanged: (position, hasGesture) {
                                    if (hasGesture) stopwatch.reset();
                                  },
                                ),
                                children: [
                                  TileLayer(
                                    urlTemplate:
                                        'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
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
                                              child: Icon(
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
                                                  Colors.blue.withAlpha(80),
                                              useRadiusInMeter: true,
                                              radius: 1000 // 1 km
                                              ),
                                        ]),
                                  // Draw a line with the last positions coming from GPS
                                  PolylineLayer(
                                    polylines: [
                                      if (trace != null) ...trace!,
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
                                                color: Theme.of(context)
                                                    .colorScheme
                                                    .surface,
                                                borderRadius:
                                                    const BorderRadius.vertical(
                                                  top: Radius.circular(10),
                                                )),
                                            child: lastRunDistance(itms) > 0
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
                                  ),
                                  if (itms.length >= 4)
                                    AnimatedOpacity(
                                        opacity: itms.elementAt(0).sportMode
                                            ? 1.0
                                            : 0.0,
                                        duration:
                                            const Duration(milliseconds: 300),
                                        child: SpeedGauge(
                                          speed:
                                              lastRunSpeed(itms.sublist(0, 4)),
                                          maxSpeed: 20,
                                          size: 50,
                                        ))
                                ],
                              ),
                            ),
                          ],
                        ),
                      );
                    } else if (snapshot.hasError) {
                      child = Text(tr(context, "try_new_token"));
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
        bottomNavigationBar: widget.sharedPosition != null
            ? null
            : StickyBottomAppBar(
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
                                kms = 0;
                                _sportMode = !_sportMode;
                              });
                              widget.foregroundTaskCommand(_sportMode);
                            }),
                        if (!_sportMode)
                          IconButton(
                              icon: const Icon(Icons.my_location),
                              onPressed: () async {
                                try {
                                  await getPositionAndPushToServer(false);
                                  await getDataAndMoveToLastPosition();
                                } catch (_) {}
                              }),
                      ],
                      IconButton(
                          icon: const Icon(Icons.refresh),
                          onPressed: () async {
                            await getDataAndMoveToLastPosition();
                          }),
                      IconButton(
                          icon: const Icon(Icons.timeline),
                          onPressed: () async {
                            await openFilePickerAndReadTrace();
                          }),
                      if (App().hasToken)
                        UsersDropdown(
                          users: users,
                          callback: (val) async {
                            displayedUser = val;
                            await getDataAndMoveToLastPosition();
                          },
                          initialIndex: 1,
                        ),
                    ],
                  ),
                )),
              ));
  }

  Future<void> getDataAndMoveToLastPosition() async {
    try {
      setState(() {
        getData();
      });
      var itms = await positions;
      if (itms.isNotEmpty) {
        _animatedMapMove(
            LatLng(itms.last.latitude, itms.last.longitude), zoom(itms.last));
      }
    } catch (_) {}
  }

  void connectWsChannel() async {
    await (wsChannel?.sink.close());
    try {
      String websocketUrl;
      if (kIsWeb) {
        websocketUrl =
            "${Uri.base.scheme == "http" ? "ws" : "wss"}://${Uri.base.host}${Uri.base.hasPort ? ':${Uri.base.port}' : ''}";
      } else {
        websocketUrl = App().prefs.hostname.replaceFirst("http", "ws");
      }
      websocketUrl +=
          "/api/positions/ws?user_id=$displayedUser&token=${Uri.encodeComponent(App().prefs.token)}";
      wsChannel = WebSocketChannel.connect(Uri.parse(websocketUrl));
      wsChannel?.stream.listen((message) async {
        Position pos = Position.fromJson(json.decode((message)));
        var itms = await positions;
        itms.insert(0, pos);
        if (itms.isNotEmpty) {
          setState(() {
            positions = Future.value(itms);
          });
          centerView(itms);
        }
      });
    } catch (e) {
      if (kDebugMode) {
        print('Error establishing WebSocket connection: $e');
      }
    }
  }

  void centerView(List<Position> itms) {
    if (stopwatch.elapsed.inSeconds > 20) {
      _animatedMapMove(
          LatLng(itms.elementAt(0).latitude, itms.elementAt(0).longitude),
          zoom(itms.elementAt(0)));
    }
  }

  Future<void> addLocalPosition(Position pos) async {
    if (pos.userId == displayedUser) {
      var itms = await positions;
      setState(() {
        itms.insert(0, pos);
        positions = Future.value(itms);
      });
      centerView(itms);
      // If we enter the trace proximity, make a success beep, if we leave, make a failure beep
      if (trace != null) {
        bool gettingCloseToTrace = trace!.cast<LatLng?>().firstWhere(
                  (element) =>
                      position.Haversine.haversine(element!.latitude,
                          element.longitude, pos.latitude, pos.longitude) <
                      0.05,
                  orElse: () => null,
                ) !=
            null;
        if (!closeToTrace && gettingCloseToTrace) {
          closeToTrace = true;
          FlutterBeep.beep();
        } else if (closeToTrace && !gettingCloseToTrace) {
          closeToTrace = false;
          FlutterBeep.beep(false);
        }
      }
      // Beep every kilometers
      int newKms = lastRunDistance(itms).floor();
      if (newKms > kms) {
        if (newKms % 5 == 0) {
          await FlutterBeep.playSysSound(
              AndroidSoundIDs.TONE_CDMA_ALERT_INCALL_LITE);
        } else {
          FlutterBeep.playSysSound(AndroidSoundIDs.TONE_CDMA_PRESSHOLDKEY_LITE);
        }
      }
      kms = newKms;
    }
  }

  double zoom(Position position) {
    if (position.source == gpsSource) {
      return 16;
    }
    return 14;
  }

  Future<void> openFilePickerAndReadTrace() async {
    try {
      FilePickerResult? result = await FilePicker.platform
          .pickFiles(type: FileType.any, withData: true);

      if (result != null && result.files.single.extension != null) {
        String fileContent = String.fromCharCodes(result.files.single.bytes!);
        switch (result.files.single.extension!.toLowerCase()) {
          case 'gpx':
            var traceXml = await GeoXml.fromGpxString(fileContent);
            setState(() {
              trace = traceXml.trks[0].trksegs
                  .map((segment) => Polyline(
                      points: segment.trkpts
                          .map((e) => LatLng(e.lat!, e.lon!))
                          .toList(),
                      strokeWidth: 6.0,
                      color: Colors.green))
                  .toList();
            });
          case 'json':
            geoJsonParser.parseGeoJsonAsString(fileContent);
            setState(() {
              trace = geoJsonParser.polylines;
            });
          default:
            setState(() {
              trace = null;
            });
        }
      } else {
        setState(() {
          trace = null;
        });
      }
    } catch (e) {
      if (kDebugMode) {
        print(e);
      }
    }
  }
}

class StickyBottomAppBar extends StatelessWidget {
  final BottomAppBar child;
  const StickyBottomAppBar({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    return Transform.translate(
      offset: Offset(0.0, -1 * MediaQuery.of(context).viewInsets.bottom),
      child: child,
    );
  }
}
