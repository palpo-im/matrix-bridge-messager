use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use anyhow::{Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tokio::time::Duration;
use tracing::{debug, error, info, instrument, warn};

use crate::db::DatabaseManager;
use crate::matrix::{MatrixAppservice, MatrixEvent};
use crate::message::MessageGateway;
use crate::metrics::{global_metrics, set_queue_depth};
use crate::utils::security::{decrypt_text, encrypt_text};
use crate::utils::validation::{escape_html, sanitize_message_text, validate_phone_number};

const DEFAULT_QUEUE_CAPACITY: usize = 1024;
const DEFAULT_MAX_ATTEMPTS: u32 = 8;
const DEFAULT_WORKERS: usize = 4;
const MAX_MESSAGE_LEN: usize = 4096;

pub fn matrix_body_from_content(content: &serde_json::Value) -> Option<String> {
    let msgtype = content.get("msgtype").and_then(|v| v.as_str()).unwrap_or("m.text");
    if msgtype != "m.text" {
        return None;
    }
    content
        .get("body")
        .and_then(|v| v.as_str())
        .map(|value| sanitize_message_text(value, MAX_MESSAGE_LEN))
        .filter(|value| !value.is_empty())
}

pub fn format_message_for_matrix(body: &str) -> String {
    escape_html(&sanitize_message_text(body, MAX_MESSAGE_LEN))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueuedMessage {
    id: String,
    room_id: String,
    phone_number: String,
    body: String,
    event_id: Option<String>,
    attempts: u32,
    next_attempt_ms: i64,
    created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedQueuedMessage {
    id: String,
    room_id: String,
    phone_number: String,
    encrypted_body: String,
    event_id: Option<String>,
    attempts: u32,
    next_attempt_ms: i64,
    created_at_ms: i64,
}

#[derive(Default)]
struct BridgeCache {
    room_to_phone: HashMap<String, String>,
    phone_to_room: HashMap<String, String>,
    contact_name: HashMap<String, Option<String>>,
}

enum WorkerOutcome {
    Sent,
    Requeue(QueuedMessage),
    Failed(QueuedMessage, String),
}

pub struct BridgeCore {
    matrix_client: Arc<MatrixAppservice>,
    message_gateway: Arc<dyn MessageGateway>,
    db_manager: Arc<DatabaseManager>,
    queue: Arc<Mutex<VecDeque<QueuedMessage>>>,
    queue_capacity: usize,
    worker_count: usize,
    max_attempts: u32,
    queue_path: PathBuf,
    queue_secret: String,
    dead_letter_path: PathBuf,
    cache: Arc<RwLock<BridgeCache>>,
}

impl BridgeCore {
    pub fn new(
        matrix_client: Arc<MatrixAppservice>,
        message_gateway: Arc<dyn MessageGateway>,
        db_manager: Arc<DatabaseManager>,
    ) -> Self {
        let queue_path = std::env::var("BRIDGE_QUEUE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/bridge_queue.json"));
        let queue_secret = std::env::var("BRIDGE_QUEUE_KEY")
            .unwrap_or_else(|_| matrix_client.config().registration.as_token.clone());
        let dead_letter_path = queue_path.with_extension("deadletter.jsonl");

        let queue_capacity = std::env::var("BRIDGE_QUEUE_CAPACITY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_QUEUE_CAPACITY);
        let worker_count = std::env::var("BRIDGE_WORKERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_WORKERS);
        let max_attempts = std::env::var("BRIDGE_MAX_ATTEMPTS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_MAX_ATTEMPTS);

        Self {
            matrix_client,
            message_gateway,
            db_manager,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            queue_capacity,
            worker_count,
            max_attempts,
            queue_path,
            queue_secret,
            dead_letter_path,
            cache: Arc::new(RwLock::new(BridgeCache::default())),
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!(
            "bridge core starting (workers={}, max_attempts={}, queue_capacity={})",
            self.worker_count, self.max_attempts, self.queue_capacity
        );

        self.load_persisted_queue().await?;
        self.update_queue_depth_metric().await;

        let mut in_flight = tokio::task::JoinSet::new();
        loop {
            while in_flight.len() < self.worker_count {
                let Some(message) = self.pop_ready_message().await else {
                    break;
                };

                let gateway = self.message_gateway.clone();
                let max_attempts = self.max_attempts;
                in_flight.spawn(async move {
                    Self::process_queued_message(gateway, message, max_attempts).await
                });
            }

            while let Some(join_result) = in_flight.try_join_next() {
                match join_result {
                    Ok(WorkerOutcome::Sent) => {
                        global_metrics()
                            .message_to_gateway_total
                            .fetch_add(1, Ordering::Relaxed);
                        global_metrics()
                            .degraded_mode
                            .store(false, Ordering::Relaxed);
                    }
                    Ok(WorkerOutcome::Requeue(message)) => {
                        self.requeue_message(message).await?;
                        global_metrics()
                            .degraded_mode
                            .store(true, Ordering::Relaxed);
                    }
                    Ok(WorkerOutcome::Failed(message, reason)) => {
                        self.write_dead_letter(&message, &reason).await?;
                        global_metrics()
                            .bridge_errors_total
                            .fetch_add(1, Ordering::Relaxed);
                        global_metrics()
                            .degraded_mode
                            .store(true, Ordering::Relaxed);
                    }
                    Err(join_err) => {
                        error!("queue worker task failed: {}", join_err);
                        global_metrics()
                            .bridge_errors_total
                            .fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            if in_flight.is_empty() {
                tokio::time::sleep(Duration::from_millis(200)).await;
            } else {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
    }

    #[instrument(skip(self, event))]
    pub async fn handle_matrix_message(&self, event: &MatrixEvent) -> Result<()> {
        global_metrics()
            .matrix_events_total
            .fetch_add(1, Ordering::Relaxed);

        let Some(content) = &event.content else {
            return Ok(());
        };

        let Some(body) = matrix_body_from_content(content) else {
            return Ok(());
        };

        debug!(
            "handling matrix message from {} in {}: {}",
            event.sender, event.room_id, body
        );

        let Some(phone_number) = self.resolve_phone_for_room(&event.room_id).await? else {
            debug!("no room mapping found for {}", event.room_id);
            return Ok(());
        };

        let message = QueuedMessage {
            id: uuid::Uuid::new_v4().to_string(),
            room_id: event.room_id.clone(),
            phone_number,
            body,
            event_id: event.event_id.clone(),
            attempts: 0,
            next_attempt_ms: Utc::now().timestamp_millis(),
            created_at_ms: Utc::now().timestamp_millis(),
        };
        self.enqueue_message(message).await?;
        Ok(())
    }

    #[instrument(skip(self, event))]
    pub async fn handle_matrix_member(&self, event: &MatrixEvent) -> Result<()> {
        let membership = event
            .content
            .as_ref()
            .and_then(|content| content.get("membership"))
            .and_then(|v| v.as_str())
            .unwrap_or("join");

        let target_user = event
            .state_key
            .as_deref()
            .unwrap_or(event.sender.as_str());
        let joined = matches!(membership, "join" | "invite");
        self.matrix_client
            .set_room_membership(&event.room_id, target_user, joined)
            .await?;

        debug!(
            "updated room member {} in {} with membership {}",
            target_user, event.room_id, membership
        );
        Ok(())
    }

    pub async fn handle_matrix_redaction(&self, event: &MatrixEvent) -> Result<()> {
        debug!("handling matrix redaction event from {}", event.sender);
        Ok(())
    }

    pub async fn handle_matrix_reaction(&self, event: &MatrixEvent) -> Result<()> {
        debug!("handling matrix reaction event from {}", event.sender);
        Ok(())
    }

    pub async fn handle_matrix_typing(&self, event: &MatrixEvent) -> Result<()> {
        debug!("handling matrix typing event from {}", event.sender);
        Ok(())
    }

    pub async fn handle_matrix_read_receipt(&self, event: &MatrixEvent) -> Result<()> {
        debug!("handling matrix read receipt event from {}", event.sender);
        Ok(())
    }

    #[instrument(skip(self, message))]
    pub async fn handle_incoming_message(
        &self,
        phone_number: &str,
        message: &str,
    ) -> Result<()> {
        if !validate_phone_number(phone_number) {
            return Err(anyhow!("invalid phone number format: {}", phone_number));
        }

        let sanitized = sanitize_message_text(message, MAX_MESSAGE_LEN);
        if sanitized.is_empty() {
            return Ok(());
        }

        info!("handling incoming message from {}", phone_number);
        let room_store = self.db_manager.room_store();
        let room_mapping = self.resolve_room_for_phone(phone_number).await?;

        let (room_id, mut mapping) = if let Some(room) = room_mapping {
            (room.matrix_room_id.clone(), Some(room))
        } else {
            let contact_name = self.get_contact_name_cached(phone_number).await?;
            let room_name = contact_name.as_deref().unwrap_or(phone_number);
            let ghost_user = self.matrix_client.ghost_user_id_for_phone(phone_number);

            let room_id = self
                .matrix_client
                .create_room(
                    Some(room_name),
                    Some(&format!("SMS bridge with {}", phone_number)),
                    Some(&[&ghost_user]),
                )
                .await?;

            let new_mapping = room_store
                .create(crate::db::NewRoomMapping {
                    matrix_room_id: room_id.clone(),
                    phone_number: phone_number.to_string(),
                    portal_name: contact_name.clone(),
                    portal_avatar: None,
                })
                .await?;

            {
                let mut cache = self.cache.write().await;
                cache
                    .room_to_phone
                    .insert(new_mapping.matrix_room_id.clone(), new_mapping.phone_number.clone());
                cache
                    .phone_to_room
                    .insert(new_mapping.phone_number.clone(), new_mapping.matrix_room_id.clone());
            }

            (room_id, Some(new_mapping))
        };

        let ghost_user = self.matrix_client.ghost_user_id_for_phone(phone_number);
        self.matrix_client
            .ensure_room_member(&room_id, &ghost_user)
            .await?;

        if let Some(contact_name) = self.get_contact_name_cached(phone_number).await? {
            self.matrix_client
                .sync_room_metadata(
                    &room_id,
                    Some(&contact_name),
                    Some(&format!("SMS bridge with {}", phone_number)),
                )
                .await?;

            if let Some(ref mut mapping_ref) = mapping {
                if mapping_ref.portal_name.as_deref() != Some(contact_name.as_str()) {
                    mapping_ref.portal_name = Some(contact_name.clone());
                    room_store.update(mapping_ref.clone()).await?;
                }
            }
        }

        let safe_body = format_message_for_matrix(&sanitized);
        self.send_to_matrix_with_retry(&room_id, &safe_body).await?;
        global_metrics()
            .message_to_matrix_total
            .fetch_add(1, Ordering::Relaxed);
        info!("forwarded message to matrix room {}", room_id);

        Ok(())
    }

    async fn send_to_matrix_with_retry(&self, room_id: &str, body: &str) -> Result<()> {
        for attempt in 0..self.max_attempts {
            match self.matrix_client.send_message(room_id, body).await {
                Ok(_) => return Ok(()),
                Err(err) if attempt + 1 < self.max_attempts => {
                    let delay = 150_u64.saturating_mul(1_u64 << attempt.min(8));
                    warn!(
                        "matrix send failed (attempt {}/{}): {}; retrying in {}ms",
                        attempt + 1,
                        self.max_attempts,
                        err,
                        delay
                    );
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
                Err(err) => return Err(err),
            }
        }
        Err(anyhow!("failed to send matrix message after retries"))
    }

    async fn resolve_phone_for_room(&self, room_id: &str) -> Result<Option<String>> {
        if let Some(cached) = self.cache.read().await.room_to_phone.get(room_id).cloned() {
            return Ok(Some(cached));
        }

        let room_store = self.db_manager.room_store();
        let mapping = room_store.get_by_matrix_id(room_id).await?;
        if let Some(mapping) = mapping {
            let phone_number = mapping.phone_number.clone();
            let mut cache = self.cache.write().await;
            cache
                .room_to_phone
                .insert(mapping.matrix_room_id.clone(), phone_number.clone());
            cache
                .phone_to_room
                .insert(phone_number.clone(), mapping.matrix_room_id);
            Ok(Some(phone_number))
        } else {
            Ok(None)
        }
    }

    async fn resolve_room_for_phone(&self, phone_number: &str) -> Result<Option<crate::db::RoomMapping>> {
        if let Some(room_id) = self.cache.read().await.phone_to_room.get(phone_number).cloned() {
            let room_store = self.db_manager.room_store();
            if let Some(mapping) = room_store.get_by_matrix_id(&room_id).await? {
                return Ok(Some(mapping));
            }
        }

        let room_store = self.db_manager.room_store();
        let mapping = room_store.get_by_phone_number(phone_number).await?;
        if let Some(mapping) = mapping {
            let mut cache = self.cache.write().await;
            cache
                .room_to_phone
                .insert(mapping.matrix_room_id.clone(), mapping.phone_number.clone());
            cache
                .phone_to_room
                .insert(mapping.phone_number.clone(), mapping.matrix_room_id.clone());
            Ok(Some(mapping))
        } else {
            Ok(None)
        }
    }

    async fn get_contact_name_cached(&self, phone_number: &str) -> Result<Option<String>> {
        if let Some(cached) = self
            .cache
            .read()
            .await
            .contact_name
            .get(phone_number)
            .cloned()
        {
            return Ok(cached);
        }

        let contact_name = self.message_gateway.get_contact_name(phone_number).await?;
        self.cache
            .write()
            .await
            .contact_name
            .insert(phone_number.to_string(), contact_name.clone());
        Ok(contact_name)
    }

    async fn enqueue_message(&self, message: QueuedMessage) -> Result<()> {
        let mut queue = self.queue.lock().await;
        if queue.len() >= self.queue_capacity {
            return Err(anyhow!(
                "bridge queue is full (capacity={})",
                self.queue_capacity
            ));
        }
        queue.push_back(message);
        set_queue_depth(queue.len());
        drop(queue);
        self.persist_queue().await
    }

    async fn pop_ready_message(&self) -> Option<QueuedMessage> {
        let now = Utc::now().timestamp_millis();
        let mut queue = self.queue.lock().await;
        let pos = queue.iter().position(|msg| msg.next_attempt_ms <= now)?;
        let message = queue.remove(pos);
        set_queue_depth(queue.len());
        message
    }

    async fn requeue_message(&self, mut message: QueuedMessage) -> Result<()> {
        let delay = 250_i64.saturating_mul(1_i64 << message.attempts.min(8));
        message.next_attempt_ms = Utc::now().timestamp_millis() + delay;
        {
            let mut queue = self.queue.lock().await;
            queue.push_back(message);
            set_queue_depth(queue.len());
        }
        self.persist_queue().await
    }

    async fn process_queued_message(
        gateway: Arc<dyn MessageGateway>,
        mut message: QueuedMessage,
        max_attempts: u32,
    ) -> WorkerOutcome {
        match gateway.health_check().await {
            Ok(true) => {}
            Ok(false) => {
                message.attempts = message.attempts.saturating_add(1);
                warn!("gateway unhealthy; deferring message {}", message.id);
                return WorkerOutcome::Requeue(message);
            }
            Err(err) => {
                message.attempts = message.attempts.saturating_add(1);
                warn!("gateway health check failed: {}", err);
                return WorkerOutcome::Requeue(message);
            }
        }

        let mut last_error: Option<String> = None;
        for send_try in 0..3 {
            match gateway.send_message(&message.phone_number, &message.body).await {
                Ok(_) => return WorkerOutcome::Sent,
                Err(err) => {
                    last_error = Some(err.to_string());
                    if send_try < 2 {
                        let backoff = 100_u64.saturating_mul(1_u64 << send_try);
                        tokio::time::sleep(Duration::from_millis(backoff)).await;
                    }
                }
            }
        }

        message.attempts = message.attempts.saturating_add(1);
        if message.attempts < max_attempts {
            WorkerOutcome::Requeue(message)
        } else {
            WorkerOutcome::Failed(
                message,
                last_error.unwrap_or_else(|| "gateway send failed".to_string()),
            )
        }
    }

    async fn persist_queue(&self) -> Result<()> {
        let queue_snapshot = {
            let queue = self.queue.lock().await;
            queue.iter().cloned().collect::<Vec<_>>()
        };

        let persisted = queue_snapshot
            .into_iter()
            .map(|entry| PersistedQueuedMessage {
                id: entry.id,
                room_id: entry.room_id,
                phone_number: entry.phone_number,
                encrypted_body: encrypt_text(&self.queue_secret, &entry.body),
                event_id: entry.event_id,
                attempts: entry.attempts,
                next_attempt_ms: entry.next_attempt_ms,
                created_at_ms: entry.created_at_ms,
            })
            .collect::<Vec<_>>();

        self.ensure_parent_dir(&self.queue_path).await?;
        let payload = serde_json::to_vec(&persisted)?;
        tokio::fs::write(&self.queue_path, payload).await?;
        Ok(())
    }

    async fn load_persisted_queue(&self) -> Result<()> {
        if !self.queue_path.exists() {
            return Ok(());
        }

        let bytes = tokio::fs::read(&self.queue_path).await?;
        if bytes.is_empty() {
            return Ok(());
        }
        let persisted: Vec<PersistedQueuedMessage> = serde_json::from_slice(&bytes)?;

        let mut queue = self.queue.lock().await;
        queue.clear();
        for item in persisted {
            let body = decrypt_text(&self.queue_secret, &item.encrypted_body)?;
            queue.push_back(QueuedMessage {
                id: item.id,
                room_id: item.room_id,
                phone_number: item.phone_number,
                body,
                event_id: item.event_id,
                attempts: item.attempts,
                next_attempt_ms: item.next_attempt_ms,
                created_at_ms: item.created_at_ms,
            });
        }
        set_queue_depth(queue.len());
        Ok(())
    }

    async fn write_dead_letter(&self, message: &QueuedMessage, reason: &str) -> Result<()> {
        self.ensure_parent_dir(&self.dead_letter_path).await?;
        let payload = serde_json::json!({
            "id": message.id,
            "room_id": message.room_id,
            "phone_number": message.phone_number,
            "encrypted_body": encrypt_text(&self.queue_secret, &message.body),
            "event_id": message.event_id,
            "attempts": message.attempts,
            "created_at_ms": message.created_at_ms,
            "failed_at_ms": Utc::now().timestamp_millis(),
            "reason": reason,
        });
        let mut line = serde_json::to_string(&payload)?;
        line.push('\n');

        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.dead_letter_path)
            .await?;
        file.write_all(line.as_bytes()).await?;
        Ok(())
    }

    async fn ensure_parent_dir(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    async fn update_queue_depth_metric(&self) {
        let queue_len = self.queue.lock().await.len();
        set_queue_depth(queue_len);
    }

    pub async fn queue_depth(&self) -> usize {
        self.queue.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{format_message_for_matrix, matrix_body_from_content};

    #[test]
    fn matrix_body_from_content_extracts_text() {
        let content = json!({
            "msgtype": "m.text",
            "body": " hello "
        });
        assert_eq!(matrix_body_from_content(&content).as_deref(), Some("hello"));
    }

    #[test]
    fn matrix_body_from_content_rejects_non_text() {
        let content = json!({
            "msgtype": "m.image",
            "body": "ignored"
        });
        assert!(matrix_body_from_content(&content).is_none());
    }

    #[test]
    fn format_message_for_matrix_escapes_html() {
        assert_eq!(
            format_message_for_matrix("<script>alert(1)</script>"),
            "&lt;script&gt;alert(1)&lt;/script&gt;"
        );
    }
}
