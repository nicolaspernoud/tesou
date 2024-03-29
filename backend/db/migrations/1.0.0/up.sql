CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name VARCHAR NOT NULL,
    surname VARCHAR NOT NULL
);

INSERT INTO
    users (name, surname)
VALUES
    ("John", "Doe");

CREATE TABLE positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    latitude DOUBLE NOT NULL,
    longitude DOUBLE NOT NULL,
    source VARCHAR NOT NULL,
    battery_level INTEGER NOT NULL,
    sport_mode BOOLEAN NOT NULL,
    time INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);