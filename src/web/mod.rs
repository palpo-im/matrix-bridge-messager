use std::sync::Arc;
use std::sync::atomic::Ordering;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use salvo::prelude::*;
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info, warn};

use crate::bridge::BridgeCore;
use crate::config::Config;
use crate::db::DatabaseManager;
use crate::matrix::MatrixAppservice;
use crate::metrics::global_metrics;
use crate::utils::security::verify_payload_signature;
use crate::utils::validation::validate_phone_number;

#[derive(Clone)]
struct WebState {
    config: Arc<Config>,
    bridge: Arc<BridgeCore>,
}

static WEB_STATE: Lazy<RwLock<Option<WebState>>> = Lazy::new(|| RwLock::new(None));

fn web_state() -> Option<WebState> {
    WEB_STATE.read().clone()
}

pub struct WebServer {
    config: Arc<Config>,
    _matrix_client: Arc<MatrixAppservice>,
    _db_manager: Arc<DatabaseManager>,
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
            _matrix_client: matrix_client,
            _db_manager: db_manager,
            bridge,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        *WEB_STATE.write() = Some(WebState {
            config: self.config.clone(),
            bridge: self.bridge.clone(),
        });

        let router = Router::new()
            .push(Router::with_path("/health").get(health))
            .push(Router::with_path("/ready").get(ready))
            .push(Router::with_path("/status").get(status))
            .push(Router::with_path("/metrics").get(metrics))
            .push(Router::with_path("/dashboard").get(dashboard))
            .push(Router::with_path("/webhook/message").post(message_webhook));

        let bind_addr = format!("{}:{}", self.config.bridge.bind_address, self.config.bridge.port);
        info!("Starting web server on {}", bind_addr);

        let acceptor = TcpListener::new(bind_addr).bind().await;
        Server::new(acceptor).serve(router).await;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct IncomingMessageWebhook {
    from: String,
    body: String,
}

#[handler]
async fn health(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    global_metrics().http_requests.fetch_add(1, Ordering::Relaxed);
    res.render(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })));
}

#[handler]
async fn ready(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    global_metrics().http_requests.fetch_add(1, Ordering::Relaxed);
    let degraded = global_metrics().degraded_mode.load(Ordering::Relaxed);
    res.render(Json(json!({
        "status": if degraded { "degraded" } else { "ready" }
    })));
}

#[handler]
async fn status(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    global_metrics().http_requests.fetch_add(1, Ordering::Relaxed);
    let queue_depth = if let Some(state) = web_state() {
        state.bridge.queue_depth().await
    } else {
        0
    };

    res.render(Json(json!({
        "status": "running",
        "version": env!("CARGO_PKG_VERSION"),
        "metrics": {
            "matrix_events_total": global_metrics().matrix_events_total.load(Ordering::Relaxed),
            "message_to_gateway_total": global_metrics().message_to_gateway_total.load(Ordering::Relaxed),
            "message_to_matrix_total": global_metrics().message_to_matrix_total.load(Ordering::Relaxed),
            "bridge_errors_total": global_metrics().bridge_errors_total.load(Ordering::Relaxed),
            "queue_depth": queue_depth,
            "degraded_mode": global_metrics().degraded_mode.load(Ordering::Relaxed),
        }
    })));
}

#[handler]
async fn metrics(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    global_metrics().http_requests.fetch_add(1, Ordering::Relaxed);
    let output = format!(
        r#"# HELP bridge_matrix_events_total Total Matrix events processed
# TYPE bridge_matrix_events_total counter
bridge_matrix_events_total {}

# HELP bridge_message_to_gateway_total Total messages forwarded to gateway
# TYPE bridge_message_to_gateway_total counter
bridge_message_to_gateway_total {}

# HELP bridge_message_to_matrix_total Total messages forwarded to Matrix
# TYPE bridge_message_to_matrix_total counter
bridge_message_to_matrix_total {}

# HELP bridge_errors_total Total bridge errors
# TYPE bridge_errors_total counter
bridge_errors_total {}

# HELP bridge_http_requests_total Total HTTP requests
# TYPE bridge_http_requests_total counter
bridge_http_requests_total {}

# HELP bridge_queue_depth Current queue depth
# TYPE bridge_queue_depth gauge
bridge_queue_depth {}
"#,
        global_metrics().matrix_events_total.load(Ordering::Relaxed),
        global_metrics().message_to_gateway_total.load(Ordering::Relaxed),
        global_metrics().message_to_matrix_total.load(Ordering::Relaxed),
        global_metrics().bridge_errors_total.load(Ordering::Relaxed),
        global_metrics().http_requests.load(Ordering::Relaxed),
        global_metrics().queue_depth.load(Ordering::Relaxed),
    );

    res.add_header("Content-Type", "text/plain; version=0.0.4", true)
        .ok();
    res.render(Text::Plain(output));
}

#[handler]
async fn dashboard(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    global_metrics().http_requests.fetch_add(1, Ordering::Relaxed);
    let queue_depth = if let Some(state) = web_state() {
        state.bridge.queue_depth().await
    } else {
        0
    };
    let alert_threshold = std::env::var("BRIDGE_ALERT_QUEUE_DEPTH")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100);
    let alert = queue_depth >= alert_threshold;

    res.render(Json(json!({
        "title": "Matrix Bridge Messager Dashboard",
        "queue_depth": queue_depth,
        "alert_threshold": alert_threshold,
        "alert_active": alert,
        "degraded_mode": global_metrics().degraded_mode.load(Ordering::Relaxed),
    })));
}

#[handler]
async fn message_webhook(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    global_metrics().http_requests.fetch_add(1, Ordering::Relaxed);
    let state = match web_state() {
        Some(state) => state,
        None => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({"error": "web state unavailable"})));
            return;
        }
    };
    let bridge = state.bridge;
    let config = state.config;

    let payload = match req.parse_json::<IncomingMessageWebhook>().await {
        Ok(payload) => payload,
        Err(err) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(json!({"error": format!("invalid payload: {}", err)})));
            return;
        }
    };

    if !validate_phone_number(&payload.from) {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(json!({"error": "invalid phone number"})));
        return;
    }

    let signature_header = req.header::<String>("X-Bridge-Signature");
    let secret = std::env::var("WEBHOOK_SECRET")
        .ok()
        .or_else(|| config.message.twilio.as_ref().map(|twilio| twilio.auth_token.clone()))
        .unwrap_or_else(|| config.registration.hs_token.clone());

    if let Some(signature) = signature_header {
        let sign_source = format!("{}:{}", payload.from, payload.body);
        if !verify_payload_signature(&secret, sign_source.as_bytes(), &signature) {
            warn!("webhook signature verification failed");
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(json!({"error": "signature verification failed"})));
            return;
        }
    } else {
        warn!("missing webhook signature header; rejecting request");
        res.status_code(StatusCode::UNAUTHORIZED);
        res.render(Json(json!({"error": "missing signature header"})));
        return;
    }

    match bridge.handle_incoming_message(&payload.from, &payload.body).await {
        Ok(()) => {
            res.render(Json(json!({"status": "accepted"})));
        }
        Err(err) => {
            error!("failed to process webhook: {}", err);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({"error": err.to_string()})));
        }
    }
}
