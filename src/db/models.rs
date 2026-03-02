use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMapping {
    pub id: i64,
    pub matrix_user_id: String,
    pub phone_number: String,
    pub contact_name: Option<String>,
    pub contact_avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMapping {
    pub id: i64,
    pub matrix_room_id: String,
    pub phone_number: String,
    pub portal_name: Option<String>,
    pub portal_avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMapping {
    pub id: i64,
    pub message_id: String,
    pub matrix_room_id: String,
    pub matrix_event_id: String,
    pub direction: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedEvent {
    pub id: i64,
    pub event_id: String,
    pub event_type: String,
    pub source: String,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalConfig {
    pub id: i64,
    pub matrix_room_id: String,
    pub phone_number: String,
    pub auto_bridge: bool,
    pub bridge_read_receipts: bool,
    pub bridge_typing: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserMapping {
    pub matrix_user_id: String,
    pub phone_number: String,
    pub contact_name: Option<String>,
    pub contact_avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRoomMapping {
    pub matrix_room_id: String,
    pub phone_number: String,
    pub portal_name: Option<String>,
    pub portal_avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMessageMapping {
    pub message_id: String,
    pub matrix_room_id: String,
    pub matrix_event_id: String,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProcessedEvent {
    pub event_id: String,
    pub event_type: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPortalConfig {
    pub matrix_room_id: String,
    pub phone_number: String,
    pub auto_bridge: bool,
    pub bridge_read_receipts: bool,
    pub bridge_typing: bool,
}

impl UserMapping {
    pub fn new(
        matrix_user_id: String,
        phone_number: String,
        contact_name: Option<String>,
        contact_avatar: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            matrix_user_id,
            phone_number,
            contact_name,
            contact_avatar,
            created_at: now,
            updated_at: now,
        }
    }
}

impl RoomMapping {
    pub fn new(
        matrix_room_id: String,
        phone_number: String,
        portal_name: Option<String>,
        portal_avatar: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            matrix_room_id,
            phone_number,
            portal_name,
            portal_avatar,
            created_at: now,
            updated_at: now,
        }
    }
}

impl MessageMapping {
    pub fn new(
        message_id: String,
        matrix_room_id: String,
        matrix_event_id: String,
        direction: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            message_id,
            matrix_room_id,
            matrix_event_id,
            direction,
            created_at: now,
        }
    }
}

impl ProcessedEvent {
    pub fn new(event_id: String, event_type: String, source: String) -> Self {
        Self {
            id: 0,
            event_id,
            event_type,
            source,
            processed_at: Utc::now(),
        }
    }
}

impl PortalConfig {
    pub fn new(
        matrix_room_id: String,
        phone_number: String,
        auto_bridge: bool,
        bridge_read_receipts: bool,
        bridge_typing: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            matrix_room_id,
            phone_number,
            auto_bridge,
            bridge_read_receipts,
            bridge_typing,
            created_at: now,
            updated_at: now,
        }
    }
}
