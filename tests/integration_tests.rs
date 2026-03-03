use std::sync::Arc;
use std::time::Duration;

use matrix_bridge_messager::bridge::BridgeCore;
use matrix_bridge_messager::config::{
    BehaviorConfig, BridgeConfig, Config, DatabaseConfig, LimitsConfig, LoggingConfig, MessageConfig,
    MockConfig, RegistrationConfig,
};
use matrix_bridge_messager::db::DatabaseManager;
use matrix_bridge_messager::matrix::{MatrixAppservice, MatrixEvent};
use matrix_bridge_messager::message::create_gateway;
use matrix_bridge_messager::utils::security::sign_payload;
use matrix_bridge_messager::web::WebServer;
use serde_json::json;

fn random_local_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("port should bind")
        .local_addr()
        .expect("local addr should be available")
        .port()
}

fn test_config(port: u16, db_url: String) -> Config {
    Config {
        bridge: BridgeConfig {
            domain: "example.com".to_string(),
            homeserver_url: "http://localhost:8008".to_string(),
            bind_address: "127.0.0.1".to_string(),
            port,
            bot_username: "message-bot".to_string(),
        },
        registration: RegistrationConfig {
            id: "message".to_string(),
            url: format!("http://127.0.0.1:{port}"),
            as_token: "test_as_token".to_string(),
            hs_token: "test_hs_token".to_string(),
            sender_localpart: "_message_".to_string(),
        },
        message: MessageConfig {
            gateway_type: "mock".to_string(),
            twilio: None,
            aws_sns: None,
            mock: Some(MockConfig { enabled: true }),
        },
        database: DatabaseConfig {
            url: db_url,
            max_connections: Some(5),
            min_connections: Some(1),
        },
        logging: LoggingConfig {
            level: "debug".to_string(),
            format: "pretty".to_string(),
        },
        behavior: BehaviorConfig {
            auto_create_portals: true,
            sync_contacts: true,
            enable_read_receipts: true,
            enable_typing_notifications: true,
            max_message_age: 3600,
        },
        limits: LimitsConfig {
            max_file_size: 1024 * 1024,
            matrix_event_age_limit_ms: 300_000,
            message_rate_limit: 100,
        },
        admin_users: vec!["@admin:example.com".to_string()],
    }
}

async fn wait_for_health(base_url: &str) {
    for _ in 0..30 {
        if reqwest::get(format!("{}/health", base_url)).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("web server did not become ready in time");
}

#[tokio::test]
async fn matrix_client_room_member_and_metadata_management() {
    let config = Arc::new(test_config(
        random_local_port(),
        format!("sqlite://./test-matrix-{}.db", uuid::Uuid::new_v4()),
    ));
    let matrix = MatrixAppservice::new(config).await.expect("matrix client should init");
    let room_id = matrix
        .create_room(
            Some("Test Room"),
            Some("Initial Topic"),
            Some(&["@alice:example.com"]),
        )
        .await
        .expect("room should be created");

    let members = matrix.room_members(&room_id).await;
    assert!(members.iter().any(|m| m == "@alice:example.com"));

    matrix
        .set_room_membership(&room_id, "@alice:example.com", false)
        .await
        .expect("membership should update");
    let members = matrix.room_members(&room_id).await;
    assert!(!members.iter().any(|m| m == "@alice:example.com"));

    matrix
        .sync_room_metadata(&room_id, Some("Renamed"), Some("Updated Topic"))
        .await
        .expect("metadata sync should work");
    let (name, topic) = matrix.room_metadata(&room_id).await;
    assert_eq!(name.as_deref(), Some("Renamed"));
    assert_eq!(topic.as_deref(), Some("Updated Topic"));
}

#[tokio::test]
async fn mock_gateway_integration_send_and_health() {
    let cfg = test_config(
        random_local_port(),
        format!("sqlite://./test-gateway-{}.db", uuid::Uuid::new_v4()),
    );
    let gateway = create_gateway(&cfg.message).expect("gateway should be created");
    let message_id = gateway
        .send_message("+12345678901", "hello")
        .await
        .expect("mock send should succeed");
    assert!(message_id.starts_with("mock_"));
    assert!(gateway.health_check().await.expect("health should be checked"));
}

#[tokio::test]
async fn bridge_logic_integration_queue_and_forwarding() {
    let cfg = Arc::new(test_config(
        random_local_port(),
        format!("sqlite://./test-bridge-{}.db", uuid::Uuid::new_v4()),
    ));
    let db = Arc::new(
        DatabaseManager::new(&cfg.database)
            .await
            .expect("database manager should init"),
    );
    let matrix = Arc::new(
        MatrixAppservice::new(cfg.clone())
            .await
            .expect("matrix appservice should init"),
    );
    let gateway = create_gateway(&cfg.message).expect("gateway should be created");
    let bridge = Arc::new(BridgeCore::new(matrix.clone(), gateway, db.clone()));

    bridge
        .handle_incoming_message("+12345678901", "Hello from SMS")
        .await
        .expect("incoming message should be handled");
    let room_mapping = db
        .room_store()
        .get_by_phone_number("+12345678901")
        .await
        .expect("lookup should succeed");
    let room_mapping = room_mapping.expect("room mapping should exist");

    bridge
        .handle_matrix_message(&MatrixEvent {
            event_id: Some("$event1:example.com".to_string()),
            event_type: "m.room.message".to_string(),
            room_id: room_mapping.matrix_room_id.clone(),
            sender: "@alice:example.com".to_string(),
            state_key: None,
            content: Some(json!({
                "msgtype": "m.text",
                "body": "Hello from Matrix",
            })),
            timestamp: Some(chrono::Utc::now().timestamp_millis().to_string()),
        })
        .await
        .expect("matrix message should enqueue");

    assert_eq!(bridge.queue_depth().await, 1);

    let worker_bridge = bridge.clone();
    let worker = tokio::spawn(async move {
        let _ = worker_bridge.start().await;
    });
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(bridge.queue_depth().await, 0);
    worker.abort();
}

#[tokio::test]
async fn web_api_integration_health_metrics_and_webhook() {
    let port = random_local_port();
    let cfg = Arc::new(test_config(
        port,
        format!("sqlite://./test-web-{}.db", uuid::Uuid::new_v4()),
    ));
    let db = Arc::new(
        DatabaseManager::new(&cfg.database)
            .await
            .expect("database manager should init"),
    );
    let matrix = Arc::new(
        MatrixAppservice::new(cfg.clone())
            .await
            .expect("matrix appservice should init"),
    );
    let gateway = create_gateway(&cfg.message).expect("gateway should be created");
    let bridge = Arc::new(BridgeCore::new(matrix.clone(), gateway, db.clone()));
    let web_server = WebServer::new(cfg.clone(), matrix, db, bridge.clone());

    let web_task = tokio::spawn(async move {
        let _ = web_server.start().await;
    });

    let base_url = format!("http://127.0.0.1:{port}");
    wait_for_health(&base_url).await;

    let health: serde_json::Value = reqwest::get(format!("{base_url}/health"))
        .await
        .expect("health request should work")
        .json()
        .await
        .expect("health response should be json");
    assert_eq!(health["status"], "healthy");

    let metrics_text = reqwest::get(format!("{base_url}/metrics"))
        .await
        .expect("metrics request should work")
        .text()
        .await
        .expect("metrics body should be text");
    assert!(metrics_text.contains("bridge_http_requests_total"));

    let payload = json!({
        "from": "+12345678901",
        "body": "hello webhook"
    });
    let sign_source = format!(
        "{}:{}",
        payload["from"].as_str().expect("from should be string"),
        payload["body"].as_str().expect("body should be string")
    );
    let signature =
        sign_payload("test_hs_token", sign_source.as_bytes()).expect("signature should be created");

    let client = reqwest::Client::new();
    let webhook_resp: serde_json::Value = client
        .post(format!("{base_url}/webhook/message"))
        .header("X-Bridge-Signature", signature)
        .json(&payload)
        .send()
        .await
        .expect("webhook request should work")
        .json()
        .await
        .expect("webhook response should be json");
    assert_eq!(webhook_resp["status"], "accepted");

    web_task.abort();
}
