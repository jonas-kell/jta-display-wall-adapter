// @generated automatically by Diesel CLI.

diesel::table! {
    athletes (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    database_state (id) {
        id -> Integer,
        created_with_version -> Text,
        data -> Text,
    }
}

diesel::table! {
    heat_assignments (id) {
        id -> Integer,
        data -> Text,
    }
}

diesel::table! {
    heat_evaluations (id) {
        id -> Text,
        belongs_to_id -> Text,
        data -> Text,
    }
}

diesel::table! {
    heat_false_starts (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    heat_finishes (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    heat_intermediates (id) {
        id -> Text,
        belongs_to_id -> Text,
        data -> Text,
    }
}

diesel::table! {
    heat_results (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    heat_start_lists (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    heat_starts (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    heat_wind_missings (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    heat_winds (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    internal_wind_measurements (id) {
        id -> Text,
        data -> Text,
        wind_meas_time -> Nullable<Timestamp>,
        stored_at_local -> Timestamp,
    }
}

diesel::table! {
    internal_wind_readings (id) {
        id -> Text,
        data -> Text,
        wind_meas_time -> Nullable<Timestamp>,
        stored_at_local -> Timestamp,
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
    athletes,database_state,heat_assignments,heat_evaluations,heat_false_starts,heat_finishes,heat_intermediates,heat_results,heat_start_lists,heat_starts,heat_wind_missings,heat_winds,internal_wind_measurements,internal_wind_readings,permanent_storage,);
