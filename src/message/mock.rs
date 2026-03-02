use async_trait::async_trait;
use anyhow::Result;
use tracing::info;

use super::MessageGateway;

pub struct MockGateway {
    enabled: bool,
}

impl MockGateway {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait]
impl MessageGateway for MockGateway {
    async fn send_message(&self, to: &str, body: &str) -> Result<String> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Mock gateway is disabled"));
        }

        let message_id = format!("mock_{}", uuid::Uuid::new_v4());
        info!("[MOCK] Sending message to {}: {} (id: {})", to, body, message_id);
        Ok(message_id)
    }

    async fn get_contact_name(&self, phone_number: &str) -> Result<Option<String>> {
        if !self.enabled {
            return Ok(None);
        }

        Ok(Some(format!("Contact {}", phone_number)))
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.enabled)
    }
}
