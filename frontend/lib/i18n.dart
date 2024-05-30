import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart' show SynchronousFuture;

String tr(context, String str) {
  return MyLocalizations.of(context)!.tr(str);
}

class MyLocalizations {
  MyLocalizations(this.locale);

  final Locale locale;

  static MyLocalizations? of(BuildContext context) {
    return Localizations.of<MyLocalizations>(context, MyLocalizations);
  }

  static final Map<String, Map<String, String>> _localizedValues = {
    'en': {
      "active_user": "Active user",
      "edit_user": "Edit user",
      "enable_log": "Enable Logging",
      "get_latest_release": "Get latest release from GitHub",
      "go_to": "To access my shared position (for 2 hours), go to the website",
      "hostname": "Hostname",
      "name": "Name",
      "new_user": "New user",
      "no_users": "No users",
      "please_enter_some_text": "Please enter some text",
      "settings": "Settings",
      "share_my_position": "Share my position",
      "share_info_copied_to_clipboard": "Share info copied to clipboard...",
      "submit": "Submit",
      "surname": "Surname",
      "tap_to_return_to_app": "Tap to return to the app.",
      "tesou_is_running": "Tesou! is running...",
      "token": "Token",
      "try_new_token": "Please try a new token",
      "user": "User",
      "user_created": "User created",
      "user_deleted": "User deleted",
      "users": "Users"
    },
    'fr': {
      "active_user": "Utilisateur actif",
      "edit_user": "Éditer utilisateur",
      "enable_log": "Activer le journal",
      "get_latest_release": "Récupérer la dernière version sur GitHub",
      "go_to":
          "Pour voir ma position en temps réel (pendant 2 heures), aller sur le site",
      "hostname": "Serveur",
      "name": "Prénom",
      "new_user": "Nouvel utilisateur",
      "no_users": "Aucun utilisateur",
      "please_enter_some_text": "Veuillez entrer du texte'",
      "settings": "Paramètres",
      "share_my_position": "Partager ma position",
      "share_info_copied_to_clipboard":
          "Informations de partage copiées dans le presse papier...",
      "submit": "Valider",
      "surname": "Nom",
      "tap_to_return_to_app": "Appuyez pour revenir à l'application.",
      "tesou_is_running": "Tesou! est en cours d'exécution.",
      "token": "Jeton de sécurité",
      "try_new_token": "Veuillez mettre à jour votre jeton de sécurité",
      "user": "Utilisateur",
      "user_created": "Utilisateur créé",
      "user_deleted": "Utilisateur supprimé",
      "users": "Utilisateurs"
    },
  };

  String tr(String token) {
    return _localizedValues[locale.languageCode]![token] ?? token;
  }
}

class MyLocalizationsDelegate extends LocalizationsDelegate<MyLocalizations> {
  const MyLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) => ['en', 'fr'].contains(locale.languageCode);

  @override
  Future<MyLocalizations> load(Locale locale) {
    // Returning a SynchronousFuture here because an async 'load' operation
    // isn't needed to produce an instance of MyLocalizations.
    return SynchronousFuture<MyLocalizations>(MyLocalizations(locale));
  }

  @override
  bool shouldReload(MyLocalizationsDelegate old) => false;
}
