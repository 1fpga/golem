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
        config_string -> Nullable<Text>,
        released_at -> Timestamp,
        last_played -> Nullable<Timestamp>,
        favorite -> Bool,
        downloaded_at -> Timestamp,
    }
}

diesel::table! {
    dat_files (id) {
        id -> Integer,
        name -> Text,
        path -> Text,
        core_id -> Integer,
        priority -> Integer,
    }
}

diesel::table! {
    games (id) {
        id -> Integer,
        name -> Text,
        core_id -> Nullable<Integer>,
        path -> Nullable<Text>,
        description -> Text,
        last_played -> Nullable<Timestamp>,
        added_at -> Timestamp,
        favorite -> Bool,
    }
}

diesel::joinable!(dat_files -> cores (core_id));
diesel::joinable!(games -> cores (core_id));

diesel::allow_tables_to_appear_in_same_query!(
    cores,
    dat_files,
    games,
);
