use std::sync::Arc;
use salvo::prelude::*;
use serde_json::json;
use tracing::info;

use crate::config::Config;
use crate::db::DatabaseManager;
use crate::matrix::MatrixAppservice;
use crate::bridge::BridgeCore;

pub struct WebServer {
    config: Arc<Config>,
    matrix_client: Arc<MatrixAppservice>,
    db_manager: Arc<DatabaseManager>,
    bridge: Arc<BridgeCore>,
}

impl WebServer {
    pub fn new(
        config: Arc<Config>,
        matrix_client: Arc<MatrixAppservice>,
        db_manager: Arc<DatabaseManager>,
        bridge: Arc<BridgeCore>,
    ) -> Self {
        Self {
            config,
            matrix_client,
            db_manager,
            bridge,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let router = Router::new()
            .push(Router::with_path("/health").get(health))
            .push(Router::with_path("/ready").get(ready))
            .push(Router::with_path("/status").get(status));

        let bind_addr = format!("{}:{}", self.config.bridge.bind_address, self.config.bridge.port);
        info!("Starting web server on {}", bind_addr);

        let acceptor = TcpListener::new(bind_addr).bind().await;
        Server::new(acceptor).serve(router).await;

        Ok(())
    }
}

#[handler]
async fn health(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(json!({
        "status": "healthy"
    })));
}

#[handler]
async fn ready(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(json!({
        "status": "ready"
    })));
}

#[handler]
async fn status(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(json!({
        "status": "running",
        "version": env!("CARGO_PKG_VERSION"),
        "bridge": {
            "domain": "configured",
            "gateway": "configured",
        }
    })));
}
