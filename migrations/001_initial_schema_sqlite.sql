-- SQLite version of initial migration

-- Create user_mappings table
CREATE TABLE IF NOT EXISTS user_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    matrix_user_id TEXT NOT NULL UNIQUE,
    phone_number TEXT NOT NULL,
    contact_name TEXT,
    contact_avatar TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_user_mappings_matrix_user_id ON user_mappings(matrix_user_id);
CREATE INDEX IF NOT EXISTS idx_user_mappings_phone_number ON user_mappings(phone_number);

-- Create room_mappings table
CREATE TABLE IF NOT EXISTS room_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    matrix_room_id TEXT NOT NULL UNIQUE,
    phone_number TEXT NOT NULL,
    portal_name TEXT,
    portal_avatar TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_room_mappings_matrix_room_id ON room_mappings(matrix_room_id);
CREATE INDEX IF NOT EXISTS idx_room_mappings_phone_number ON room_mappings(phone_number);

-- Create message_mappings table
CREATE TABLE IF NOT EXISTS message_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id TEXT NOT NULL UNIQUE,
    matrix_room_id TEXT NOT NULL,
    matrix_event_id TEXT NOT NULL,
    direction TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_message_mappings_message_id ON message_mappings(message_id);
CREATE INDEX IF NOT EXISTS idx_message_mappings_matrix_event_id ON message_mappings(matrix_event_id);

-- Create processed_events table
CREATE TABLE IF NOT EXISTS processed_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL UNIQUE,
    event_type TEXT NOT NULL,
    source TEXT NOT NULL,
    processed_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_processed_events_event_id ON processed_events(event_id);

-- Create portal_configs table
CREATE TABLE IF NOT EXISTS portal_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    matrix_room_id TEXT NOT NULL UNIQUE,
    phone_number TEXT NOT NULL,
    auto_bridge INTEGER DEFAULT 1,
    bridge_read_receipts INTEGER DEFAULT 1,
    bridge_typing INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_portal_configs_matrix_room_id ON portal_configs(matrix_room_id);
CREATE INDEX IF NOT EXISTS idx_portal_configs_phone_number ON portal_configs(phone_number);
