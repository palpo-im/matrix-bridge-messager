use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::db::{MessageMapping, PortalConfig, ProcessedEvent, RoomMapping, UserMapping};

pub type SharedMemoryDatabase = Arc<RwLock<MemoryDatabase>>;

#[derive(Default)]
pub struct MemoryDatabase {
    pub next_user_id: i64,
    pub next_room_id: i64,
    pub next_message_id: i64,
    pub next_event_id: i64,
    pub next_portal_id: i64,
    pub users: HashMap<i64, UserMapping>,
    pub user_by_matrix: HashMap<String, i64>,
    pub user_by_phone: HashMap<String, i64>,
    pub rooms: HashMap<i64, RoomMapping>,
    pub room_by_matrix: HashMap<String, i64>,
    pub room_by_phone: HashMap<String, i64>,
    pub messages: HashMap<i64, MessageMapping>,
    pub message_by_id: HashMap<String, i64>,
    pub message_by_matrix_event: HashMap<String, i64>,
    pub processed_events: HashMap<i64, ProcessedEvent>,
    pub processed_event_ids: HashSet<String>,
    pub portals: HashMap<i64, PortalConfig>,
    pub portal_by_matrix_room: HashMap<String, i64>,
    pub portal_by_phone: HashMap<String, i64>,
}

impl MemoryDatabase {
    pub fn allocate_user_id(&mut self) -> i64 {
        self.next_user_id += 1;
        self.next_user_id
    }

    pub fn allocate_room_id(&mut self) -> i64 {
        self.next_room_id += 1;
        self.next_room_id
    }

    pub fn allocate_message_id(&mut self) -> i64 {
        self.next_message_id += 1;
        self.next_message_id
    }

    pub fn allocate_event_id(&mut self) -> i64 {
        self.next_event_id += 1;
        self.next_event_id
    }

    pub fn allocate_portal_id(&mut self) -> i64 {
        self.next_portal_id += 1;
        self.next_portal_id
    }
}

static SQLITE_DATABASES: Lazy<RwLock<HashMap<String, SharedMemoryDatabase>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn sqlite_database(path: &str) -> SharedMemoryDatabase {
    let mut dbs = SQLITE_DATABASES.write();
    dbs.entry(path.to_string())
        .or_insert_with(|| Arc::new(RwLock::new(MemoryDatabase::default())))
        .clone()
}
