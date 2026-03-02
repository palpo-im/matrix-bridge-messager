#![forbid(unsafe_code)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_comparisons)]

use std::sync::Arc;
use anyhow::Result;
use tracing::{error, info};

mod cli;
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
use cli::{parse_args, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    utils::init();
    
    let args = parse_args();
    
    if let Some(command) = args.command {
        handle_command(command).await?;
        return Ok(());
    }
    
    run_bridge().await
}

async fn handle_command(command: cli::Commands) -> Result<()> {
    match command {
        Commands::GenerateRegistration { output, id, homeserver_url, domain } => {
            let registration = cli::generate_registration(&id, &homeserver_url, &domain);
            std::fs::write(&output, &registration)?;
            info!("Generated registration file at {:?}", output);
            println!("Registration file written to: {:?}", output);
            println!("\nPlease copy this file to your Matrix homeserver's appservice directory.");
        }
        Commands::ValidateConfig => {
            let config = Config::load()?;
            info!("Configuration is valid");
            println!("✓ Configuration file is valid");
            println!("  Bridge domain: {}", config.bridge.domain);
            println!("  Homeserver URL: {}", config.bridge.homeserver_url);
            println!("  Gateway type: {}", config.message.gateway_type);
        }
        Commands::ListPortals { limit } => {
            info!("Listing portals (limit: {})", limit);
            println!("Portal listing not yet implemented");
        }
        Commands::Unbridge { room, leave } => {
            info!("Unbridging room {} (leave: {})", room, leave);
            println!("Unbridge command not yet implemented");
        }
        Commands::AdminMe { user, room, power_level } => {
            info!("Granting admin privileges to {} (room: {:?}, power_level: {})", 
                  user, room, power_level);
            println!("Admin command not yet implemented");
        }
        Commands::Status => {
            println!("Bridge Status:");
            println!("  Status: Not implemented");
            println!("  Use 'curl http://localhost:9006/status' when bridge is running");
        }
        Commands::TestGateway { to, message } => {
            let config = Config::load()?;
            let gateway = create_gateway(&config.message)?;
            let message = message.unwrap_or_else(|| "Test message from Matrix bridge".to_string());
            
            match gateway.send_message(&to, &message).await {
                Ok(msg_id) => {
                    println!("✓ Test message sent successfully");
                    println!("  To: {}", to);
                    println!("  Message ID: {}", msg_id);
                }
                Err(e) => {
                    println!("✗ Failed to send test message: {}", e);
                }
            }
        }
    }
    Ok(())
}

async fn run_bridge() -> Result<()> {
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
