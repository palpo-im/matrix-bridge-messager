use std::sync::Arc;
use anyhow::Result;
use tracing::{debug, error, info};

use crate::db::DatabaseManager;
use crate::matrix::{MatrixAppservice, MatrixEvent};
use crate::message::MessageGateway;

pub struct BridgeCore {
    matrix_client: Arc<MatrixAppservice>,
    message_gateway: Arc<dyn MessageGateway>,
    db_manager: Arc<DatabaseManager>,
}

impl BridgeCore {
    pub fn new(
        matrix_client: Arc<MatrixAppservice>,
        message_gateway: Arc<dyn MessageGateway>,
        db_manager: Arc<DatabaseManager>,
    ) -> Self {
        Self {
            matrix_client,
            message_gateway,
            db_manager,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("bridge core starting");
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    pub async fn handle_matrix_message(&self, event: &MatrixEvent) -> Result<()> {
        let Some(content) = &event.content else {
            return Ok(());
        };

        let msgtype = content
            .get("msgtype")
            .and_then(|v| v.as_str())
            .unwrap_or("m.text");

        let body = content
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        debug!(
            "handling matrix message from {} in {}: {}",
            event.sender, event.room_id, body
        );

        let room_store = self.db_manager.room_store();
        let room_mapping = room_store.get_by_matrix_id(&event.room_id).await?;

        if let Some(room) = room_mapping {
            self.message_gateway.send_message(&room.phone_number, body).await?;
            info!("forwarded message to {}", room.phone_number);
        } else {
            debug!("no room mapping found for {}", event.room_id);
        }

        Ok(())
    }

    pub async fn handle_matrix_member(&self, event: &MatrixEvent) -> Result<()> {
        debug!("handling matrix member event from {}", event.sender);
        Ok(())
    }

    pub async fn handle_matrix_redaction(&self, event: &MatrixEvent) -> Result<()> {
        debug!("handling matrix redaction event from {}", event.sender);
        Ok(())
    }

    pub async fn handle_matrix_reaction(&self, event: &MatrixEvent) -> Result<()> {
        debug!("handling matrix reaction event from {}", event.sender);
        Ok(())
    }

    pub async fn handle_matrix_typing(&self, event: &MatrixEvent) -> Result<()> {
        debug!("handling matrix typing event from {}", event.sender);
        Ok(())
    }

    pub async fn handle_matrix_read_receipt(&self, event: &MatrixEvent) -> Result<()> {
        debug!("handling matrix read receipt event from {}", event.sender);
        Ok(())
    }

    pub async fn handle_incoming_message(
        &self,
        phone_number: &str,
        message: &str,
    ) -> Result<()> {
        info!("handling incoming message from {}: {}", phone_number, message);

        let room_store = self.db_manager.room_store();
        let room_mapping = room_store.get_by_phone_number(phone_number).await?;

        let room_id = if let Some(room) = room_mapping {
            room.matrix_room_id.clone()
        } else {
            let contact_name = self.message_gateway.get_contact_name(phone_number).await?;
            let room_name = contact_name.as_deref().unwrap_or(phone_number);
            
            let ghost_user = self.matrix_client.ghost_user_id_for_phone(phone_number);
            
            let room_id = self.matrix_client.create_room(
                Some(room_name),
                Some(&format!("SMS bridge with {}", phone_number)),
                Some(&[&ghost_user]),
            ).await?;

            room_store.create(crate::db::NewRoomMapping {
                matrix_room_id: room_id.clone(),
                phone_number: phone_number.to_string(),
                portal_name: contact_name.clone(),
                portal_avatar: None,
            }).await?;

            room_id
        };

        self.matrix_client.send_message(&room_id, message).await?;
        info!("forwarded message to matrix room {}", room_id);

        Ok(())
    }
}
