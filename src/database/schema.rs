// @generated automatically by Diesel CLI.

diesel::table! {
    heat_starts (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    permanent_storage (id) {
        id -> Text,
        name_key -> Text,
        stored_at -> Timestamp,
        data -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    heat_starts,permanent_storage,);
