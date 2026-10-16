//! Configuration Module for Shambhavi AI Labs Pehredar Box

pub mod appliance;
pub mod camera;

pub use appliance::{ApplianceConfig, HardwareTier};
pub use camera::CameraConfig;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Complete NVR Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NvrConfig {
    pub appliance: ApplianceConfig,
    pub cameras: Vec<CameraConfig>,
}

impl Default for NvrConfig {
    fn default() -> Self {
        Self {
            appliance: ApplianceConfig::default(),
            cameras: vec![
                CameraConfig::new("cam_01", "Main Entrance", "rtsp://127.0.0.1:8554/cam_01"),
                CameraConfig::new("cam_02", "Cash Counter / Vault", "rtsp://127.0.0.1:8554/cam_02"),
                CameraConfig::new("cam_03", "Back Shutter", "rtsp://127.0.0.1:8554/cam_03"),
                CameraConfig::new("cam_04", "Perimeter Wall", "rtsp://127.0.0.1:8554/cam_04"),
            ],
        }
    }
}

impl NvrConfig {
    /// Load configuration from a JSON file, or create default if not present
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            let data = fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file at {:?}", path))?;
            let config: Self = serde_json::from_str(&data)
                .with_context(|| "Failed to parse NVR JSON configuration")?;
            Ok(config)
        } else {
            let config = Self::default();
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let data = serde_json::to_string_pretty(&config)?;
            let _ = fs::write(path, data);
            Ok(config)
        }
    }
}
