# Tesou!

An app to share position with trusted people.

## Backend

Backend is made with Actix-web and Rust.

## Frontend

Frontend is made with Flutter.
A native Kotlin plugin is built in for accessing Android GPS Location and Cell ID. iOS is not (yet) supported.

### Recreate frontend

```
mv frontend frontend_old
flutter create --template=app --platforms="android,web" --description="An app to share position with trusted people." --org="fr.ninico" --project-name="tesou" frontend
cd frontend
flutter create --template=plugin --platforms="android" --description="Location without Google Play Services" --org="fr.ninico" --project-name="aosp_location" aosp_location
```
