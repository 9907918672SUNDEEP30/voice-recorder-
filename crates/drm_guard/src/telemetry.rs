//! # Usage Metric Telemetry Module
//!
//! Provides the `UsageMetricProvider` trait allowing any application in the workspace
//! to report application-specific metered usage metrics into the heartbeat payload.

use async_trait::async_trait;
use serde_json::{Map, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Core trait that applications implement to inject metered usage data into heartbeat pings.
#[async_trait]
pub trait UsageMetricProvider: Send + Sync + 'static {
    /// Canonical application identifier (e.g., "cctv_guard", "voice_agent", "local_rag_engine")
    fn app_name(&self) -> &'static str;

    /// Collects application-specific metrics (e.g., active cameras, call minutes, vector counts)
    async fn collect_metrics(&self) -> Map<String, Value>;
}

/// No-op metric provider for basic seat-only licensing
pub struct NoOpMetrics {
    app_identifier: &'static str,
}

impl NoOpMetrics {
    pub fn new(app_identifier: &'static str) -> Self {
        Self { app_identifier }
    }
}

#[async_trait]
impl UsageMetricProvider for NoOpMetrics {
    fn app_name(&self) -> &'static str {
        self.app_identifier
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        Map::new()
    }
}

/// Dynamic key-value metric provider for applications that want simple runtime metric updating
#[derive(Clone)]
pub struct DynamicMetrics {
    app_identifier: &'static str,
    metrics_map: Arc<RwLock<Map<String, Value>>>,
}

impl DynamicMetrics {
    pub fn new(app_identifier: &'static str) -> Self {
        Self {
            app_identifier,
            metrics_map: Arc::new(RwLock::new(Map::new())),
        }
    }

    pub async fn set_metric(&self, key: &str, value: Value) {
        let mut map = self.metrics_map.write().await;
        map.insert(key.to_string(), value);
    }

    pub async fn increment_counter(&self, key: &str, delta: i64) {
        let mut map = self.metrics_map.write().await;
        let current = map.get(key).and_then(|v| v.as_i64()).unwrap_or(0);
        map.insert(key.to_string(), Value::from(current + delta));
    }
}

#[async_trait]
impl UsageMetricProvider for DynamicMetrics {
    fn app_name(&self) -> &'static str {
        self.app_identifier
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let map = self.metrics_map.read().await;
        map.clone()
    }
}
