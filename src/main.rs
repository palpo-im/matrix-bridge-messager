#![forbid(unsafe_code)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_comparisons)]

use std::sync::Arc;
use anyhow::Result;
use tracing::{error, info};

mod config;
mod db;
mod matrix;
mod message;
mod bridge;
mod web;
mod utils;

use config::Config;
use db::DatabaseManager;
use matrix::MatrixAppservice;
use message::create_gateway;
use web::WebServer;

#[tokio::main]
async fn main() -> Result<()> {
    utils::init();
    
    let config = Arc::new(Config::load()?);
    info!("matrix-message bridge starting up");
    
    let db_manager = Arc::new(DatabaseManager::new(&config.database).await?);
    db_manager.migrate().await?;
    info!("Database initialized");
    
    let matrix_client = Arc::new(MatrixAppservice::new(config.clone()).await?);
    info!("Matrix appservice initialized");
    
    let message_gateway = create_gateway(&config.message)?;
    info!("Message gateway initialized: {}", config.message.gateway_type);
    
    let mut event_handler = matrix::MatrixEventHandlerImpl::new(matrix_client.clone());
    
    let bridge = Arc::new(bridge::BridgeCore::new(
        matrix_client.clone(),
        message_gateway.clone(),
        db_manager.clone(),
    ));
    
    event_handler.set_bridge(bridge.clone());
    let processor = Arc::new(matrix::MatrixEventProcessor::with_age_limit(
        Arc::new(event_handler),
        config.limits.matrix_event_age_limit_ms as i64,
    ));
    matrix_client.set_processor(processor).await;
    
    let web_server = WebServer::new(
        config.clone(),
        matrix_client.clone(),
        db_manager.clone(),
        bridge.clone(),
    );
    
    let web_handle = tokio::spawn(async move {
        if let Err(e) = web_server.start().await {
            error!("web server error: {}", e);
        }
    });
    
    let bridge_handle = tokio::spawn(async move {
        if let Err(e) = bridge.start().await {
            error!("bridge error: {}", e);
        }
    });
    
    tokio::pin!(web_handle);
    tokio::pin!(bridge_handle);
    
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("received Ctrl+C, beginning shutdown");
        },
        _ = &mut web_handle => {
            info!("web server task exited, beginning shutdown");
        },
        _ = &mut bridge_handle => {
            info!("bridge task exited, beginning shutdown");
        },
    }
    
    web_handle.abort();
    bridge_handle.abort();
    
    info!("matrix-message bridge shutting down");
    Ok(())
}
