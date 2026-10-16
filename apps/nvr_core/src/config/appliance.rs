//! Shambhavi AI Labs — Pehredar Box Appliance Configuration
//! Linux Native, Headless & Cross-Platform Configuration Model

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Hardware capacity tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HardwareTier {
    /// Up to 8 Cameras (Intel N100/N200, 8GB RAM)
    Lite,
    /// Up to 16 Cameras (Intel Core i3/i5, 16GB RAM)
    Pro,
    /// Up to 32 Cameras (Intel Core i7, 32GB RAM)
    Enterprise,
}

impl Default for HardwareTier {
    fn default() -> Self {
        Self::Lite
    }
}

fn default_model_path() -> PathBuf {
    if PathBuf::from("./models/yolov8m.onnx").exists() {
        PathBuf::from("./models/yolov8m.onnx")
    } else if PathBuf::from("C:/AI_Models/yolov8m.onnx").exists() {
        PathBuf::from("C:/AI_Models/yolov8m.onnx")
    } else if PathBuf::from("/opt/shambhavi/models/yolov8m.onnx").exists() {
        PathBuf::from("/opt/shambhavi/models/yolov8m.onnx")
    } else {
        PathBuf::from("./models/yolov8m.onnx")
    }
}

fn default_true() -> bool {
    true
}

fn default_quota() -> u8 {
    90
}

fn default_chunk_duration() -> u64 {
    900 // 15 minutes
}

fn default_lease_endpoint() -> String {
    "https://licensing.shambhavi-ai-labs.workers.dev/api/v1/lease".to_string()
}

fn default_lease_interval() -> u64 {
    3_600_000 // 1 hour
}

/// Global appliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplianceConfig {
    /// Unique appliance identifier (e.g. "pehredar_box_001")
    pub box_id: String,
    /// Hardware tier
    pub tier: HardwareTier,
    /// Storage path for continuous 15-min MP4 recordings
    pub storage_dir: PathBuf,
    /// Storage path for locked high-priority evidence
    pub evidence_dir: PathBuf,
    /// Path to YOLO ONNX vision model
    #[serde(default = "default_model_path")]
    pub model_path: PathBuf,
    /// Webhook URL for WhatsApp / Telegram / Cloud push alerts
    #[serde(default)]
    pub alert_webhook_url: Option<String>,
    /// Maximum disk usage percentage before FIFO pruning begins (default: 90%)
    #[serde(default = "default_quota")]
    pub disk_quota_percentage: u8,
    /// Chunk duration for continuous video segments in seconds (default: 900 = 15 mins)
    #[serde(default = "default_chunk_duration")]
    pub chunk_duration_secs: u64,
    /// Cloudflare Worker lease verification endpoint
    #[serde(default = "default_lease_endpoint")]
    pub cloud_lease_endpoint: String,
    /// Milliseconds between heartbeat checks (default: 3600000 = 1 hour)
    #[serde(default = "default_lease_interval")]
    pub lease_check_interval_ms: u64,
    /// Whether AI intelligence is enabled (default: true)
    #[serde(default = "default_true")]
    pub enable_ai: bool,
}

impl Default for ApplianceConfig {
    fn default() -> Self {
        Self {
            box_id: "shambhavi_pehredar_box_001".to_string(),
            tier: HardwareTier::Lite,
            storage_dir: PathBuf::from("./recordings"),
            evidence_dir: PathBuf::from("./evidence"),
            model_path: default_model_path(),
            alert_webhook_url: None,
            disk_quota_percentage: 90,
            chunk_duration_secs: 900,
            cloud_lease_endpoint: default_lease_endpoint(),
            lease_check_interval_ms: default_lease_interval(),
            enable_ai: true,
        }
    }
}
