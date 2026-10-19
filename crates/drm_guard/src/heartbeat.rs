//! # Heartbeat Protocol & License Types Module
//!
//! Defines the request and response structures for Cloudflare Edge Worker communication,
//! including 24-hour leasing, metered metrics, and OTA binary update metadata.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseStatus {
    Active,
    ExpiringSoon,
    Expired,
    Revoked,
    HardwareMismatch,
    ClockTamperingDetected,
    LimitExceeded,
    Unlicensed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateMetadata {
    pub version_available: String,
    pub download_url: String,
    pub sha256: String,
    #[serde(default)]
    pub signature_hex: Option<String>,
    #[serde(default)]
    pub mandatory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub client_id: String,
    pub license_key: String,
    pub machine_uuid: String,
    pub app_name: String,
    pub current_version: Option<String>,
    pub timestamp: i64,
    pub nonce: String,
    pub metrics: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub status: String,
    #[serde(default)]
    pub expires_at: i64,
    #[serde(alias = "session_key")]
    pub session_key_hex: Option<String>,
    pub signature_hex: Option<String>,
    pub message: Option<String>,
    pub update: Option<UpdateMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmStatusInfo {
    pub client_id: String,
    pub app_name: String,
    pub current_version: String,
    pub machine_uuid: String,
    pub status: LicenseStatus,
    pub expires_at: i64,
    pub seconds_remaining: i64,
    pub is_valid: bool,
    pub update_available: Option<UpdateMetadata>,
}
