// @generated automatically by Diesel CLI.

diesel::table! {
    cores (id) {
        id -> Integer,
        name -> Text,
        user_name -> Nullable<Text>,
        path -> Text,
        author -> Text,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        extensions -> Text,
    }
}
