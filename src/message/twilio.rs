use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::time::Duration;
use tracing::{debug, info, warn};

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

    fn messages_api_url(&self) -> String {
        format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        )
    }
}

#[async_trait]
impl MessageGateway for TwilioGateway {
    async fn send_message(&self, to: &str, body: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct TwilioMessageResponse {
            sid: String,
        }

        let url = self.messages_api_url();
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 0..3 {
            let response = self
                .client
                .post(&url)
                .basic_auth(&self.account_sid, Some(&self.auth_token))
                .json(&serde_json::json!({
                    "To": to,
                    "From": &self.phone_number,
                    "Body": body,
                }))
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let payload: TwilioMessageResponse = resp
                        .json()
                        .await
                        .context("failed to parse twilio success response")?;
                    info!("[TWILIO] sent message to {} (sid={})", to, payload.sid);
                    return Ok(payload.sid);
                }
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    let err = anyhow!("twilio send failed: status={} body={}", status, text);
                    last_error = Some(err);
                }
                Err(err) => {
                    last_error = Some(err.into());
                }
            }

            if attempt < 2 {
                let backoff = 250_u64.saturating_mul(1_u64 << attempt);
                warn!(
                    "[TWILIO] send failed on attempt {}/3, retrying in {}ms",
                    attempt + 1,
                    backoff
                );
                tokio::time::sleep(Duration::from_millis(backoff)).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("unknown twilio send error")))
    }

    async fn get_contact_name(&self, _phone_number: &str) -> Result<Option<String>> {
        Ok(None)
    }

    async fn health_check(&self) -> Result<bool> {
        if self.account_sid.is_empty() || self.auth_token.is_empty() || self.phone_number.is_empty() {
            debug!("twilio health check failed due to missing credentials");
            return Ok(false);
        }
        Ok(true)
    }
}
