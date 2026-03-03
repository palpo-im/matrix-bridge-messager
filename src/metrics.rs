use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use once_cell::sync::Lazy;

#[derive(Default)]
pub struct BridgeMetrics {
    pub matrix_events_total: AtomicU64,
    pub message_to_gateway_total: AtomicU64,
    pub message_to_matrix_total: AtomicU64,
    pub bridge_errors_total: AtomicU64,
    pub http_requests: AtomicU64,
    pub queue_depth: AtomicU64,
    pub degraded_mode: AtomicBool,
}

static GLOBAL_METRICS: Lazy<BridgeMetrics> = Lazy::new(BridgeMetrics::default);

pub fn global_metrics() -> &'static BridgeMetrics {
    &GLOBAL_METRICS
}

pub fn set_queue_depth(depth: usize) {
    global_metrics()
        .queue_depth
        .store(depth as u64, Ordering::Relaxed);
}
