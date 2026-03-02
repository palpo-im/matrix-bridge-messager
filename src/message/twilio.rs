use async_trait::async_trait;
use anyhow::{Result, Context};
use tracing::{info, debug};

use super::MessageGateway;

pub struct TwilioGateway {
    account_sid: String,
    auth_token: String,
    phone_number: String,
    client: reqwest::Client,
}

impl TwilioGateway {
    pub fn new(account_sid: String, auth_token: String, phone_number: String) -> Self {
        Self {
            account_sid,
            auth_token,
            phone_number,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl MessageGateway for TwilioGateway {
    async fn send_message(&self, to: &str, body: &str) -> Result<String> {
        info!("[TWILIO] Would send message to {}: {}", to, body);
        Ok(format!("twilio_{}", uuid::Uuid::new_v4()))
    }

    async fn get_contact_name(&self, _phone_number: &str) -> Result<Option<String>> {
        Ok(None)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
