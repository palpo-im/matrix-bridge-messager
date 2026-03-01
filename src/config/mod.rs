pub mod parser;
pub mod validator;

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub bridge: BridgeConfig,
    pub registration: RegistrationConfig,
    pub message: MessageConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub behavior: BehaviorConfig,
    pub limits: LimitsConfig,
    pub admin_users: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgeConfig {
    pub domain: String,
    pub homeserver_url: String,
    pub bind_address: String,
    pub port: u16,
    pub bot_username: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistrationConfig {
    pub id: String,
    pub url: String,
    pub as_token: String,
    pub hs_token: String,
    pub sender_localpart: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageConfig {
    pub gateway_type: String,
    pub twilio: Option<TwilioConfig>,
    pub aws_sns: Option<AwsSnsConfig>,
    pub mock: Option<MockConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub phone_number: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AwsSnsConfig {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MockConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BehaviorConfig {
    pub auto_create_portals: bool,
    pub sync_contacts: bool,
    pub enable_read_receipts: bool,
    pub enable_typing_notifications: bool,
    pub max_message_age: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    pub max_file_size: u64,
    pub matrix_event_age_limit_ms: u64,
    pub message_rate_limit: u32,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = std::env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.yaml".to_string());
        
        Self::load_from_path(&config_path)
    }
    
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        parser::parse_config_file(path)
    }
}
