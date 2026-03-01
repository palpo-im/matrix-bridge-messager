use anyhow::{anyhow, Result};

pub fn validate_config(config: &crate::config::Config) -> Result<()> {
    if config.bridge.domain.is_empty() {
        return Err(anyhow!("bridge.domain cannot be empty"));
    }

    if config.bridge.homeserver_url.is_empty() {
        return Err(anyhow!("bridge.homeserver_url cannot be empty"));
    }

    if config.database.url.is_empty() {
        return Err(anyhow!("database.url cannot be empty"));
    }

    if config.registration.as_token.is_empty()
        || config.registration.as_token == "CHANGE_ME_AS_TOKEN"
    {
        return Err(anyhow!(
            "registration.as_token must be set to a valid value"
        ));
    }

    if config.registration.hs_token.is_empty()
        || config.registration.hs_token == "CHANGE_ME_HS_TOKEN"
    {
        return Err(anyhow!(
            "registration.hs_token must be set to a valid value"
        ));
    }

    if !["twilio", "aws_sns", "mock"].contains(&config.message.gateway_type.as_str()) {
        return Err(anyhow!(
            "message.gateway_type must be one of: twilio, aws_sns, mock"
        ));
    }

    if config.message.gateway_type == "twilio" {
        if config.message.twilio.is_none() {
            return Err(anyhow!(
                "message.twilio configuration is required when gateway_type is 'twilio'"
            ));
        }
    }

    if config.message.gateway_type == "aws_sns" {
        if config.message.aws_sns.is_none() {
            return Err(anyhow!(
                "message.aws_sns configuration is required when gateway_type is 'aws_sns'"
            ));
        }
    }

    Ok(())
}
