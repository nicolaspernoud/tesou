import 'package:audio_service/audio_service.dart';
import 'package:just_audio/just_audio.dart';

class TesouAudioHandler extends BaseAudioHandler {
  final _player = AudioPlayer();

  Future<void> play1km() =>
      _player.setAsset("assets/sounds/1km.mp3").then((value) => _player.play());
  Future<void> play5km() =>
      _player.setAsset("assets/sounds/5km.mp3").then((value) => _player.play());
  Future<void> playEnteredTrack() => _player
      .setAsset("assets/sounds/entered_track.mp3")
      .then((value) => _player.play());
  Future<void> playExitedTrack() => _player
      .setAsset("assets/sounds/exited_track.mp3")
      .then((value) => _player.play());
}
