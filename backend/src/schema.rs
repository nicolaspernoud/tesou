table! {
    positions (id) {
        id -> Integer,
        user_id -> Integer,
        latitude -> Double,
        longitude -> Double,
        source -> Text,
        battery_level -> Integer,
        time -> BigInt,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        surname -> Text,
    }
}

joinable!(positions -> users (user_id));

allow_tables_to_appear_in_same_query!(positions, users,);
