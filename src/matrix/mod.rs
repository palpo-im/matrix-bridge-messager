use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use anyhow::Result;
use matrix_bot_sdk::appservice::{Appservice, AppserviceHandler};
use matrix_bot_sdk::client::{MatrixAuth, MatrixClient};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use url::Url;

use crate::config::Config;

pub mod event_handler;
pub mod command_handler;

pub use self::event_handler::{MatrixEventHandler, MatrixEventHandlerImpl, MatrixEventProcessor};
pub use self::command_handler::{MatrixCommandHandler, MatrixCommandOutcome};

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
    room_state: Arc<RwLock<HashMap<String, RoomState>>>,
}

#[derive(Debug, Clone, Default)]
struct RoomState {
    name: Option<String>,
    topic: Option<String>,
    members: HashSet<String>,
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
            room_state: Arc::new(RwLock::new(HashMap::new())),
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
        let room_id = format!("!mock_room_{}:{}", uuid::Uuid::new_v4(), self.config.bridge.domain);

        let mut members = HashSet::new();
        members.insert(self.bot_user_id());
        if let Some(invites) = invite {
            for user in invites {
                members.insert((*user).to_string());
            }
        }

        let state = RoomState {
            name: name.map(ToOwned::to_owned),
            topic: topic.map(ToOwned::to_owned),
            members,
        };
        self.room_state.write().await.insert(room_id.clone(), state);

        debug!("Created mock room {} name={:?} topic={:?}", room_id, name, topic);
        Ok(room_id)
    }

    pub async fn invite_user(&self, room_id: &str, user_id: &str) -> Result<()> {
        let mut rooms = self.room_state.write().await;
        let room = rooms
            .entry(room_id.to_string())
            .or_insert_with(RoomState::default);
        room.members.insert(user_id.to_string());
        debug!("Invited {} to {}", user_id, room_id);
        Ok(())
    }

    pub async fn set_room_membership(
        &self,
        room_id: &str,
        user_id: &str,
        joined: bool,
    ) -> Result<()> {
        let mut rooms = self.room_state.write().await;
        let room = rooms
            .entry(room_id.to_string())
            .or_insert_with(RoomState::default);
        if joined {
            room.members.insert(user_id.to_string());
        } else {
            room.members.remove(user_id);
        }
        debug!(
            "Updated membership in {} for {} -> {}",
            room_id, user_id, joined
        );
        Ok(())
    }

    pub async fn sync_room_metadata(
        &self,
        room_id: &str,
        name: Option<&str>,
        topic: Option<&str>,
    ) -> Result<()> {
        let mut rooms = self.room_state.write().await;
        let room = rooms
            .entry(room_id.to_string())
            .or_insert_with(RoomState::default);
        if let Some(name) = name {
            room.name = Some(name.to_string());
        }
        if let Some(topic) = topic {
            room.topic = Some(topic.to_string());
        }
        debug!(
            "Synced room metadata for {} name={:?} topic={:?}",
            room_id, room.name, room.topic
        );
        Ok(())
    }

    pub async fn room_members(&self, room_id: &str) -> Vec<String> {
        let rooms = self.room_state.read().await;
        rooms
            .get(room_id)
            .map(|room| room.members.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub async fn room_metadata(&self, room_id: &str) -> (Option<String>, Option<String>) {
        let rooms = self.room_state.read().await;
        if let Some(room) = rooms.get(room_id) {
            (room.name.clone(), room.topic.clone())
        } else {
            (None, None)
        }
    }

    pub async fn ensure_room_member(&self, room_id: &str, user_id: &str) -> Result<()> {
        let members = self.room_members(room_id).await;
        if members.iter().any(|member| member == user_id) {
            return Ok(());
        }
        self.invite_user(room_id, user_id).await?;
        Ok(())
    }
}

