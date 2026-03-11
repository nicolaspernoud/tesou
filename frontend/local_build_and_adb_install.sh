#!/bin/bash

source "$HOME"/.profile &&\
adb shell am force-stop fr.ninico.tesou &&\
flutter build apk --split-per-abi &&\
adb install build/app/outputs/flutter-apk/app-arm64-v8a-release.apk &&\
read -p "Launch Tesou! and press any key to continue..." &&\
adb reboot