// @generated automatically by Diesel CLI.

diesel::table! {
    core_files (id) {
        id -> Integer,
        name -> Nullable<Text>,
        core_id -> Integer,
        core_index -> Integer,
        game_id -> Integer,
        path -> Text,
        created_at -> Timestamp,
        last_loaded -> Nullable<Timestamp>,
    }
}

diesel::table! {
    cores (id) {
        id -> Integer,
        name -> Text,
        slug -> Text,
        system_slug -> Text,
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

diesel::table! {
    savestates (id) {
        id -> Integer,
        name -> Nullable<Text>,
        core_id -> Integer,
        game_id -> Integer,
        path -> Text,
        screenshot_path -> Nullable<Text>,
        favorite -> Bool,
        created_at -> Timestamp,
        last_played -> Nullable<Timestamp>,
    }
}

diesel::table! {
    storage (key) {
        key -> Nullable<Text>,
        value -> Text,
        username -> Nullable<Text>,
    }
}

diesel::joinable!(core_files -> cores (core_id));
diesel::joinable!(core_files -> games (game_id));
diesel::joinable!(dat_files -> cores (core_id));
diesel::joinable!(games -> cores (core_id));
diesel::joinable!(savestates -> cores (core_id));
diesel::joinable!(savestates -> games (game_id));

diesel::allow_tables_to_appear_in_same_query!(
    core_files,
    cores,
    dat_files,
    games,
    savestates,
    storage,
);
