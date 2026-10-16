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

/// Strongly-typed metrics for the Cloud Voice AI Calling Agent (`voice_agent`)
#[derive(Clone, Default)]
pub struct VoiceAgentMetrics {
    pub calls_completed: Arc<std::sync::atomic::AtomicU64>,
    pub total_call_seconds: Arc<std::sync::atomic::AtomicU64>,
}

impl VoiceAgentMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_call(&self, duration_seconds: u64) {
        self.calls_completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_call_seconds.fetch_add(duration_seconds, std::sync::atomic::Ordering::Relaxed);
    }
}

#[async_trait]
impl UsageMetricProvider for VoiceAgentMetrics {
    fn app_name(&self) -> &'static str {
        "voice_agent"
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(
            "calls_completed".to_string(),
            Value::from(self.calls_completed.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map.insert(
            "total_call_seconds".to_string(),
            Value::from(self.total_call_seconds.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map.insert(
            "total_call_minutes".to_string(),
            Value::from(
                (self.total_call_seconds.load(std::sync::atomic::Ordering::Relaxed) as f64 / 60.0).round(),
            ),
        );
        map
    }
}

/// Strongly-typed metrics for the CCTV Guard / Pehredar Black-Box (`cctv_guard`)
#[derive(Clone)]
pub struct CctvGuardMetrics {
    pub camera_count: Arc<std::sync::atomic::AtomicU32>,
    pub wire_faults: Arc<std::sync::atomic::AtomicU32>,
    pub offline_cameras: Arc<RwLock<Vec<String>>>,
}

impl CctvGuardMetrics {
    pub fn new(camera_count: u32) -> Self {
        Self {
            camera_count: Arc::new(std::sync::atomic::AtomicU32::new(camera_count)),
            wire_faults: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            offline_cameras: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn report_camera_status(&self, offline: Vec<String>, wire_loose: bool) {
        if wire_loose {
            self.wire_faults.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        let mut list = self.offline_cameras.write().await;
        *list = offline;
    }
}

#[async_trait]
impl UsageMetricProvider for CctvGuardMetrics {
    fn app_name(&self) -> &'static str {
        "cctv_guard"
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(
            "active_camera_count".to_string(),
            Value::from(self.camera_count.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map.insert(
            "wire_faults".to_string(),
            Value::from(self.wire_faults.load(std::sync::atomic::Ordering::Relaxed)),
        );
        let offline = self.offline_cameras.read().await;
        map.insert("offline_cameras".to_string(), serde_json::to_value(&*offline).unwrap_or(Value::Array(Vec::new())));
        map
    }
}

/// Strongly-typed metrics for the Financial Super Engine (`financial_engine`)
#[derive(Clone, Default)]
pub struct FinancialMetrics {
    pub special_searches_count: Arc<std::sync::atomic::AtomicU64>,
    pub deep_scans_count: Arc<std::sync::atomic::AtomicU64>,
}

impl FinancialMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_special_search(&self) {
        self.special_searches_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_deep_scan(&self) {
        self.deep_scans_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[async_trait]
impl UsageMetricProvider for FinancialMetrics {
    fn app_name(&self) -> &'static str {
        "financial_engine"
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(
            "special_searches_count".to_string(),
            Value::from(self.special_searches_count.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map.insert(
            "deep_scans_count".to_string(),
            Value::from(self.deep_scans_count.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map
    }
}

/// Strongly-typed metrics for the Local RAG CRM Vault (`local_rag_engine`)
#[derive(Clone, Default)]
pub struct RagEngineMetrics {
    pub query_count: Arc<std::sync::atomic::AtomicU64>,
    pub chunk_count: Arc<std::sync::atomic::AtomicU64>,
}

impl RagEngineMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_query(&self) {
        self.query_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_chunk_count(&self, count: u64) {
        self.chunk_count.store(count, std::sync::atomic::Ordering::Relaxed);
    }
}

#[async_trait]
impl UsageMetricProvider for RagEngineMetrics {
    fn app_name(&self) -> &'static str {
        "local_rag_engine"
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(
            "query_count".to_string(),
            Value::from(self.query_count.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map.insert(
            "chunk_count".to_string(),
            Value::from(self.chunk_count.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map
    }
}

/// Strongly-typed metrics for Video Studio (`video_studio`)
#[derive(Clone, Default)]
pub struct VideoStudioMetrics {
    pub exports_count: Arc<std::sync::atomic::AtomicU32>,
    pub render_seconds: Arc<std::sync::atomic::AtomicU64>,
}

impl VideoStudioMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_export(&self, duration_seconds: u64) {
        self.exports_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.render_seconds.fetch_add(duration_seconds, std::sync::atomic::Ordering::Relaxed);
    }
}

#[async_trait]
impl UsageMetricProvider for VideoStudioMetrics {
    fn app_name(&self) -> &'static str {
        "video_studio"
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(
            "exports_count".to_string(),
            Value::from(self.exports_count.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map.insert(
            "render_seconds".to_string(),
            Value::from(self.render_seconds.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map.insert(
            "render_minutes".to_string(),
            Value::from(
                (self.render_seconds.load(std::sync::atomic::Ordering::Relaxed) as f64 / 60.0).round(),
            ),
        );
        map
    }
}

/// Strongly-typed metrics for Audio DSP Recorder (`dsp_recorder`)
#[derive(Clone, Default)]
pub struct DspRecorderMetrics {
    pub session_count: Arc<std::sync::atomic::AtomicU32>,
    pub recording_seconds: Arc<std::sync::atomic::AtomicU64>,
}

impl DspRecorderMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_session(&self, duration_seconds: u64) {
        self.session_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.recording_seconds.fetch_add(duration_seconds, std::sync::atomic::Ordering::Relaxed);
    }
}

#[async_trait]
impl UsageMetricProvider for DspRecorderMetrics {
    fn app_name(&self) -> &'static str {
        "dsp_recorder"
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(
            "session_count".to_string(),
            Value::from(self.session_count.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map.insert(
            "recording_seconds".to_string(),
            Value::from(self.recording_seconds.load(std::sync::atomic::Ordering::Relaxed)),
        );
        map
    }
}

