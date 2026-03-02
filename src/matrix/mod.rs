use std::sync::Arc;
use anyhow::{Context, Result};
use matrix_bot_sdk::appservice::{Appservice, AppserviceHandler};
use matrix_bot_sdk::client::{MatrixAuth, MatrixClient};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use url::Url;

use crate::config::Config;

pub mod event_handler;

pub use self::event_handler::{MatrixEventHandler, MatrixEventHandlerImpl, MatrixEventProcessor};

pub struct BridgeAppserviceHandler {
    processor: Option<Arc<MatrixEventProcessor>>,
}

#[async_trait::async_trait]
impl AppserviceHandler for BridgeAppserviceHandler {
    async fn on_transaction(&self, _txn_id: &str, body: &Value) -> Result<()> {
        let Some(processor) = &self.processor else {
            return Ok(());
        };

        if let Some(events) = body.get("events").and_then(|v| v.as_array()) {
            for event in events {
                let Some(room_id) = event.get("room_id").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Some(sender) = event.get("sender").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Some(event_type) = event.get("type").and_then(|v| v.as_str()) else {
                    continue;
                };

                let matrix_event = MatrixEvent {
                    event_id: event
                        .get("event_id")
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned),
                    event_type: event_type.to_owned(),
                    room_id: room_id.to_owned(),
                    sender: sender.to_owned(),
                    state_key: event
                        .get("state_key")
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned),
                    content: event.get("content").cloned(),
                    timestamp: event.get("origin_server_ts").map(|v| v.to_string()),
                };

                if let Err(e) = processor.process_event(matrix_event).await {
                    error!("error processing event: {}", e);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct MatrixAppservice {
    config: Arc<Config>,
    pub appservice: Appservice,
    handler: Arc<RwLock<BridgeAppserviceHandler>>,
}

#[derive(Debug, Clone)]
pub struct MatrixEvent {
    pub event_id: Option<String>,
    pub event_type: String,
    pub room_id: String,
    pub sender: String,
    pub state_key: Option<String>,
    pub content: Option<Value>,
    pub timestamp: Option<String>,
}

fn ghost_user_id(phone_number: &str, domain: &str) -> String {
    let sanitized = phone_number
        .replace("+", "")
        .replace("-", "")
        .replace(" ", "");
    format!("@_message_{}:{}", sanitized, domain)
}

fn is_namespaced_user(user_id: &str) -> bool {
    user_id.starts_with("@_message_")
}

impl MatrixAppservice {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!(
            "initializing matrix appservice for {}",
            config.bridge.domain
        );

        let homeserver_url = Url::parse(&config.bridge.homeserver_url)?;
        let auth = MatrixAuth::new(&config.registration.as_token);
        let client = MatrixClient::new(homeserver_url, auth);

        let handler = Arc::new(RwLock::new(BridgeAppserviceHandler { processor: None }));

        struct HandlerWrapper(Arc<RwLock<BridgeAppserviceHandler>>);
        #[async_trait::async_trait]
        impl AppserviceHandler for HandlerWrapper {
            async fn on_transaction(&self, txn_id: &str, body: &Value) -> Result<()> {
                self.0.read().await.on_transaction(txn_id, body).await
            }
        }

        let appservice = Appservice::new(
            &config.registration.hs_token,
            &config.registration.as_token,
            client,
        )
        .with_appservice_id(&config.registration.id)
        .with_handler(Arc::new(HandlerWrapper(handler.clone())));

        Ok(Self {
            config,
            appservice,
            handler,
        })
    }

    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    pub fn bot_user_id(&self) -> String {
        format!(
            "@{}:{}",
            self.config.registration.sender_localpart, self.config.bridge.domain
        )
    }

    pub fn is_namespaced_user(&self, user_id: &str) -> bool {
        is_namespaced_user(user_id)
    }

    pub fn ghost_user_id_for_phone(&self, phone_number: &str) -> String {
        ghost_user_id(phone_number, &self.config.bridge.domain)
    }

    pub async fn set_processor(&self, processor: Arc<MatrixEventProcessor>) {
        let mut handler = self.handler.write().await;
        handler.processor = Some(processor);
    }

    pub async fn send_message(
        &self,
        room_id: &str,
        message: &str,
    ) -> Result<String> {
        debug!("Would send message to {}: {}", room_id, message);
        Ok(format!("mock_event_{}", uuid::Uuid::new_v4()))
    }

    pub async fn create_room(
        &self,
        name: Option<&str>,
        topic: Option<&str>,
        invite: Option<&[&str]>,
    ) -> Result<String> {
        debug!("Would create room: {:?} {:?}", name, topic);
        Ok(format!("!mock_room_{}:{}", uuid::Uuid::new_v4(), self.config.bridge.domain))
    }

    pub async fn invite_user(&self, room_id: &str, user_id: &str) -> Result<()> {
        debug!("Would invite {} to {}", user_id, room_id);
        Ok(())
    }
}
