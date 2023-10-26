// @generated automatically by Diesel CLI.

diesel::table! {
    cores (id) {
        id -> Integer,
        name -> Text,
        slug -> Text,
        version -> Text,
        path -> Text,
        author -> Text,
        description -> Text,
        released_at -> Timestamp,
        last_played -> Nullable<Timestamp>,
        favorite -> Bool,
        downloaded_at -> Timestamp,
    }
}

diesel::table! {
    games (id) {
        id -> Integer,
        name -> Text,
        slug -> Text,
        core_id -> Integer,
        path -> Nullable<Text>,
        description -> Text,
        last_played -> Nullable<Timestamp>,
        added_at -> Timestamp,
        favorite -> Bool,
    }
}

diesel::joinable!(games -> cores (core_id));

diesel::allow_tables_to_appear_in_same_query!(
    cores,
    games,
);
