use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;

pub mod mock;
pub mod twilio;

pub use mock::MockGateway;
pub use twilio::TwilioGateway;

#[async_trait]
pub trait MessageGateway: Send + Sync {
    async fn send_message(&self, to: &str, body: &str) -> Result<String>;
    async fn get_contact_name(&self, phone_number: &str) -> Result<Option<String>>;
    async fn health_check(&self) -> Result<bool>;
}

pub fn create_gateway(config: &crate::config::MessageConfig) -> Result<Arc<dyn MessageGateway>> {
    match config.gateway_type.as_str() {
        "mock" => {
            let enabled = config.mock.as_ref().map(|m| m.enabled).unwrap_or(true);
            Ok(Arc::new(MockGateway::new(enabled)))
        }
        "twilio" => {
            let twilio_config = config.twilio.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Twilio configuration missing"))?;
            Ok(Arc::new(TwilioGateway::new(
                twilio_config.account_sid.clone(),
                twilio_config.auth_token.clone(),
                twilio_config.phone_number.clone(),
            )))
        }
        _ => Err(anyhow::anyhow!("Unknown gateway type: {}", config.gateway_type)),
    }
}
