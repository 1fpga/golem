// @generated automatically by Diesel CLI.

diesel::table! {
    cores (id) {
        id -> Integer,
        name -> Text,
        version -> Text,
        path -> Text,
        author -> Text,
        description -> Text,
        released_at -> Timestamp,
        downloaded_at -> Timestamp,
    }
}
