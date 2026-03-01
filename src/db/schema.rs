diesel::table! {
    user_mappings (id) {
        id -> BigInt,
        matrix_user_id -> Text,
        phone_number -> Text,
        contact_name -> Nullable<Text>,
        contact_avatar -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    room_mappings (id) {
        id -> BigInt,
        matrix_room_id -> Text,
        phone_number -> Text,
        portal_name -> Nullable<Text>,
        portal_avatar -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    message_mappings (id) {
        id -> BigInt,
        message_id -> Text,
        matrix_room_id -> Text,
        matrix_event_id -> Text,
        direction -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    processed_events (id) {
        id -> BigInt,
        event_id -> Text,
        event_type -> Text,
        source -> Text,
        processed_at -> Timestamptz,
    }
}

diesel::table! {
    portal_configs (id) {
        id -> BigInt,
        matrix_room_id -> Text,
        phone_number -> Text,
        auto_bridge -> Bool,
        bridge_read_receipts -> Bool,
        bridge_typing -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    user_mappings,
    room_mappings,
    message_mappings,
    processed_events,
    portal_configs,
);
