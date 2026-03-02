#[cfg(test)]
mod tests {
    use crate::cli::{generate_registration, Commands};
    use crate::config::Config;

    #[test]
    fn test_generate_registration() {
        let registration = generate_registration("test", "http://localhost:8008", "example.org");

        assert!(registration.contains("id: test"));
        assert!(registration.contains("url: http://localhost:8008"));
        assert!(registration.contains("sender_localpart: _message_"));
    }

    #[test]
    fn test_config_sample() {
        let config_content = r#"
bridge:
  domain: "example.com"
  homeserver_url: "http://127.0.0.1:6006"
  bind_address: "0.0.0.0"
  port: 9006
  bot_username: "message-bot"

registration:
  id: "message"
  url: "http://127.0.0.1:9006"
  as_token: "test_token"
  hs_token: "test_token"
  sender_localpart: "_message_"

message:
  gateway_type: "mock"
  mock:
    enabled: true

database:
  url: "sqlite://./test.db"

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
  max_file_size: 104857600
  matrix_event_age_limit_ms: 300000
  message_rate_limit: 100

admin_users:
  - "@admin:example.com"
"#;

        let config: Config = serde_yaml::from_str(config_content).expect("Failed to parse config");
        assert_eq!(config.bridge.domain, "example.com");
        assert_eq!(config.message.gateway_type, "mock");
    }
}
