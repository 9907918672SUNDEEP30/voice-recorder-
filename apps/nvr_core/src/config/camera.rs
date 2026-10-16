//! Camera Configuration Model

use serde::{Deserialize, Serialize};

/// Individual camera stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    /// Unique camera ID (e.g. "cam_main_gate", "cam_vault_01")
    pub id: String,
    /// Friendly human label
    pub name: String,
    /// Primary High-Res RTSP Stream (for recording and Tier-2 AI)
    pub rtsp_url: String,
    /// Low-Res Sub-stream (for 16-channel HDMI Grid display & Tier-1 Sentinel)
    pub sub_stream_url: Option<String>,
    /// Expected framerate (typically 15-25 FPS)
    pub fps: u32,
    /// Whether this camera is active
    pub enabled: bool,
    /// Tier-1 Sentinel sensitivity threshold (0.01 - 0.10, default 0.04)
    pub motion_threshold: f32,
    /// Cooldown seconds after alert before re-triggering (default 15.0)
    pub cooldown_secs: f32,
}

impl CameraConfig {
    pub fn new(id: impl Into<String>, name: impl Into<String>, rtsp_url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            rtsp_url: rtsp_url.into(),
            sub_stream_url: None,
            fps: 25,
            enabled: true,
            motion_threshold: 0.04,
            cooldown_secs: 15.0,
        }
    }
}
