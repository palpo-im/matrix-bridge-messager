-- Initial migration: Create user_mappings table
CREATE TABLE IF NOT EXISTS user_mappings (
    id BIGSERIAL PRIMARY KEY,
    matrix_user_id VARCHAR(255) NOT NULL UNIQUE,
    phone_number VARCHAR(50) NOT NULL,
    contact_name VARCHAR(255),
    contact_avatar TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_mappings_matrix_user_id ON user_mappings(matrix_user_id);
CREATE INDEX idx_user_mappings_phone_number ON user_mappings(phone_number);

-- Create room_mappings table
CREATE TABLE IF NOT EXISTS room_mappings (
    id BIGSERIAL PRIMARY KEY,
    matrix_room_id VARCHAR(255) NOT NULL UNIQUE,
    phone_number VARCHAR(50) NOT NULL,
    portal_name VARCHAR(255),
    portal_avatar TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_room_mappings_matrix_room_id ON room_mappings(matrix_room_id);
CREATE INDEX idx_room_mappings_phone_number ON room_mappings(phone_number);

-- Create message_mappings table
CREATE TABLE IF NOT EXISTS message_mappings (
    id BIGSERIAL PRIMARY KEY,
    message_id VARCHAR(255) NOT NULL UNIQUE,
    matrix_room_id VARCHAR(255) NOT NULL,
    matrix_event_id VARCHAR(255) NOT NULL,
    direction VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_message_mappings_message_id ON message_mappings(message_id);
CREATE INDEX idx_message_mappings_matrix_event_id ON message_mappings(matrix_event_id);

-- Create processed_events table
CREATE TABLE IF NOT EXISTS processed_events (
    id BIGSERIAL PRIMARY KEY,
    event_id VARCHAR(255) NOT NULL UNIQUE,
    event_type VARCHAR(100) NOT NULL,
    source VARCHAR(50) NOT NULL,
    processed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_processed_events_event_id ON processed_events(event_id);

-- Create portal_configs table
CREATE TABLE IF NOT EXISTS portal_configs (
    id BIGSERIAL PRIMARY KEY,
    matrix_room_id VARCHAR(255) NOT NULL UNIQUE,
    phone_number VARCHAR(50) NOT NULL,
    auto_bridge BOOLEAN DEFAULT true,
    bridge_read_receipts BOOLEAN DEFAULT true,
    bridge_typing BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_portal_configs_matrix_room_id ON portal_configs(matrix_room_id);
CREATE INDEX idx_portal_configs_phone_number ON portal_configs(phone_number);
