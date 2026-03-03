use anyhow::{Context, Result};
use config::{Config as ConfigRs, Environment, File};
use std::path::Path;

pub fn parse_config_file<P: AsRef<Path>>(path: P) -> Result<crate::config::Config> {
    let path = path.as_ref();

    let mut config_builder = ConfigRs::builder().add_source(File::from(path));

    config_builder = config_builder
        // Keep legacy prefix compatibility; new prefix takes precedence below.
        .add_source(
            Environment::with_prefix("MATRIX_BRIDGE_MESSAGE")
                .separator("__")
                .try_parsing(true),
        )
        .add_source(
            Environment::with_prefix("MATRIX_BRIDGE_MESSAGER")
                .separator("__")
                .try_parsing(true),
        );

    let config = config_builder
        .build()
        .with_context(|| format!("Failed to build configuration from {:?}", path))?;

    let config: crate::config::Config = config
        .try_deserialize()
        .with_context(|| "Failed to deserialize configuration")?;

    crate::config::validator::validate_config(&config)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::{Builder, NamedTempFile};

    use super::parse_config_file;

    fn write_config(contents: &str) -> NamedTempFile {
        let mut file = Builder::new()
            .suffix(".yaml")
            .tempfile()
            .expect("temp file should be created");
        file.write_all(contents.as_bytes())
            .expect("config should be written");
        file
    }

    #[test]
    fn parse_config_file_success() {
        let file = write_config(
            r#"
bridge:
  domain: "example.com"
  homeserver_url: "http://localhost:8008"
  bind_address: "127.0.0.1"
  port: 9006
  bot_username: "bot"
registration:
  id: "message"
  url: "http://localhost:9006"
  as_token: "valid_as_token"
  hs_token: "valid_hs_token"
  sender_localpart: "_message_"
message:
  gateway_type: "mock"
  mock:
    enabled: true
database:
  url: "sqlite://./data/test.db"
  max_connections: 10
  min_connections: 1
logging:
  level: "info"
  format: "pretty"
behavior:
  auto_create_portals: true
  sync_contacts: true
  enable_read_receipts: true
  enable_typing_notifications: true
  max_message_age: 86400
limits:
  max_file_size: 1048576
  matrix_event_age_limit_ms: 300000
  message_rate_limit: 100
admin_users:
  - "@admin:example.com"
"#,
        );

        let parsed = parse_config_file(file.path()).expect("config should parse");
        assert_eq!(parsed.bridge.domain, "example.com");
        assert_eq!(parsed.message.gateway_type, "mock");
    }

    #[test]
    fn parse_config_file_validation_error() {
        let file = write_config(
            r#"
bridge:
  domain: "example.com"
  homeserver_url: "http://localhost:8008"
  bind_address: "127.0.0.1"
  port: 9006
  bot_username: "bot"
registration:
  id: "message"
  url: "http://localhost:9006"
  as_token: "CHANGE_ME_AS_TOKEN"
  hs_token: "CHANGE_ME_HS_TOKEN"
  sender_localpart: "_message_"
message:
  gateway_type: "mock"
  mock:
    enabled: true
database:
  url: "sqlite://./data/test.db"
logging:
  level: "info"
  format: "pretty"
behavior:
  auto_create_portals: true
  sync_contacts: true
  enable_read_receipts: true
  enable_typing_notifications: true
  max_message_age: 86400
limits:
  max_file_size: 1048576
  matrix_event_age_limit_ms: 300000
  message_rate_limit: 100
admin_users:
  - "@admin:example.com"
"#,
        );

        let error = parse_config_file(file.path()).expect_err("invalid tokens should fail");
        assert!(error.to_string().contains("registration"));
    }
}

