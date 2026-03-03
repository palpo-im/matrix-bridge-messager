use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use tracing::debug;

use super::{MatrixAppservice, MatrixEvent};

const DEFAULT_AGE_LIMIT_MS: i64 = 900_000;

#[async_trait]
pub trait MatrixEventHandler: Send + Sync {
    async fn handle_room_message(&self, event: &MatrixEvent) -> Result<()>;
    async fn handle_room_member(&self, event: &MatrixEvent) -> Result<()>;
    async fn handle_room_redaction(&self, event: &MatrixEvent) -> Result<()>;
    async fn handle_reaction(&self, event: &MatrixEvent) -> Result<()>;
    async fn handle_presence(&self, event: &MatrixEvent) -> Result<()>;
    async fn handle_typing(&self, event: &MatrixEvent) -> Result<()>;
    async fn handle_read_receipt(&self, event: &MatrixEvent) -> Result<()>;
}

pub struct MatrixEventHandlerImpl {
    _appservice: Arc<MatrixAppservice>,
    bridge: Option<Arc<crate::bridge::BridgeCore>>,
}

impl MatrixEventHandlerImpl {
    pub fn new(appservice: Arc<MatrixAppservice>) -> Self {
        Self {
            _appservice: appservice,
            bridge: None,
        }
    }

    pub fn set_bridge(&mut self, bridge: Arc<crate::bridge::BridgeCore>) {
        self.bridge = Some(bridge);
    }
}

#[async_trait]
impl MatrixEventHandler for MatrixEventHandlerImpl {
    async fn handle_room_message(&self, event: &MatrixEvent) -> Result<()> {
        if let Some(bridge) = &self.bridge {
            bridge.handle_matrix_message(event).await?;
        } else {
            debug!("matrix message received without bridge binding");
        }
        Ok(())
    }

    async fn handle_room_member(&self, event: &MatrixEvent) -> Result<()> {
        if let Some(bridge) = &self.bridge {
            bridge.handle_matrix_member(event).await?;
        } else {
            debug!("matrix member received without bridge binding");
        }
        Ok(())
    }

    async fn handle_room_redaction(&self, event: &MatrixEvent) -> Result<()> {
        if let Some(bridge) = &self.bridge {
            bridge.handle_matrix_redaction(event).await?;
        } else {
            debug!("matrix redaction received without bridge binding");
        }
        Ok(())
    }

    async fn handle_reaction(&self, event: &MatrixEvent) -> Result<()> {
        if let Some(bridge) = &self.bridge {
            bridge.handle_matrix_reaction(event).await?;
        } else {
            debug!("matrix reaction received without bridge binding");
        }
        Ok(())
    }

    async fn handle_presence(&self, _event: &MatrixEvent) -> Result<()> {
        Ok(())
    }

    async fn handle_typing(&self, event: &MatrixEvent) -> Result<()> {
        if let Some(bridge) = &self.bridge {
            bridge.handle_matrix_typing(event).await?;
        } else {
            debug!("matrix typing received without bridge binding");
        }
        Ok(())
    }

    async fn handle_read_receipt(&self, event: &MatrixEvent) -> Result<()> {
        if let Some(bridge) = &self.bridge {
            bridge.handle_matrix_read_receipt(event).await?;
        } else {
            debug!("matrix read receipt received without bridge binding");
        }
        Ok(())
    }
}

pub struct MatrixEventProcessor {
    handler: Arc<dyn MatrixEventHandler>,
    age_limit_ms: i64,
}

impl MatrixEventProcessor {
    pub fn new(handler: Arc<dyn MatrixEventHandler>) -> Self {
        Self {
            handler,
            age_limit_ms: DEFAULT_AGE_LIMIT_MS,
        }
    }

    pub fn with_age_limit(handler: Arc<dyn MatrixEventHandler>, age_limit_ms: i64) -> Self {
        Self {
            handler,
            age_limit_ms: if age_limit_ms > 0 {
                age_limit_ms
            } else {
                DEFAULT_AGE_LIMIT_MS
            },
        }
    }

    pub async fn process_event(&self, event: MatrixEvent) -> Result<()> {
        if self.is_event_too_old(&event) {
            debug!(
                "ignoring old event {} (type: {})",
                event.event_id.as_deref().unwrap_or("unknown"),
                event.event_type
            );
            return Ok(());
        }

        if self.is_namespaced_user(&event.sender) {
            debug!(
                "ignoring event from namespaced user {}",
                event.sender
            );
            return Ok(());
        }

        match event.event_type.as_str() {
            "m.room.message" => {
                self.handler.handle_room_message(&event).await?;
            }
            "m.room.member" => {
                self.handler.handle_room_member(&event).await?;
            }
            "m.room.redaction" => {
                self.handler.handle_room_redaction(&event).await?;
            }
            "m.reaction" => {
                self.handler.handle_reaction(&event).await?;
            }
            "m.presence" => {
                self.handler.handle_presence(&event).await?;
            }
            "m.typing" => {
                self.handler.handle_typing(&event).await?;
            }
            "m.receipt" => {
                self.handler.handle_read_receipt(&event).await?;
            }
            _ => {
                debug!(
                    "unhandled event type: {} in room {}",
                    event.event_type, event.room_id
                );
            }
        }

        Ok(())
    }

    fn is_event_too_old(&self, event: &MatrixEvent) -> bool {
        if let Some(timestamp_str) = &event.timestamp {
            if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                let now = chrono::Utc::now().timestamp_millis();
                let age = now - timestamp;
                return age > self.age_limit_ms;
            }
        }
        false
    }

    fn is_namespaced_user(&self, user_id: &str) -> bool {
        user_id.starts_with("@_message_")
    }
}

