#![forbid(unsafe_code)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_comparisons)]

use std::sync::Arc;
use anyhow::Result;
use tracing::{error, info};

mod config;
mod utils;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    utils::init();
    
    let config = Arc::new(Config::load()?);
    info!("matrix-message bridge starting up");
    
    info!("Configuration loaded successfully");
    info!("Bridge domain: {}", config.bridge.domain);
    info!("Homeserver URL: {}", config.bridge.homeserver_url);
    info!("Message gateway type: {}", config.message.gateway_type);
    
    info!("matrix-message bridge is ready (basic framework)");
    
    tokio::signal::ctrl_c().await?;
    info!("received Ctrl+C, beginning shutdown");
    
    info!("matrix-message bridge shutting down");
    Ok(())
}
