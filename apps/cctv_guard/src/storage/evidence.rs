use crate::rtsp::frame::CctvFrame;
use crate::tier2::qwen_vlm::{ThreatAssessment, ThreatCategory, ThreatLevel};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Structured log record emitted for every verified security incident.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentLogRecord {
    pub timestamp_ms: u64,
    pub camera_id: String,
    pub anomaly_score: f32,
    pub threat_level: ThreatLevel,
    pub category: ThreatCategory,
    pub confidence: f32,
    pub description: String,
    pub snapshot_path: Option<String>,
    pub inference_time_ms: u64,
}

/// Configuration options for local evidence logging and snapshot archiving.
#[derive(Debug, Clone)]
pub struct EvidenceVaultConfig {
    pub base_dir: PathBuf,
    pub save_snapshots: bool,
    pub jpeg_quality: u8,
}

impl Default for EvidenceVaultConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("evidence"),
            save_snapshots: true,
            jpeg_quality: 85,
        }
    }
}

/// Evidence archival engine saving high-res JPEG frames and appending structured JSON logs.
pub struct EvidenceVault {
    config: EvidenceVaultConfig,
    log_file_path: PathBuf,
}

impl EvidenceVault {
    /// Creates a new evidence vault and creates the target storage directory if needed.
    pub fn new(config: EvidenceVaultConfig) -> Result<Self> {
        create_dir_all(&config.base_dir)?;
        let log_file_path = config.base_dir.join("incidents.jsonl");

        Ok(Self {
            config,
            log_file_path,
        })
    }

    /// Records an incident to disk: encodes and saves JPEG snapshot and appends JSON event log.
    pub fn record_incident(
        &self,
        frame: &CctvFrame,
        assessment: &ThreatAssessment,
        anomaly_score: f32,
        camera_id: &str,
    ) -> Result<IncidentLogRecord> {
        let timestamp_ms = frame.timestamp_ms;
        let cat_str = format!("{:?}", assessment.category);

        // 1. Save JPEG Snapshot if enabled
        let snapshot_path = if self.config.save_snapshots {
            let filename = format!("incident_{}_{}.jpg", timestamp_ms, cat_str);
            let file_path = self.config.base_dir.join(&filename);

            let jpeg_bytes = frame.to_jpeg(self.config.jpeg_quality)?;
            std::fs::write(&file_path, jpeg_bytes)?;
            Some(file_path.to_string_lossy().to_string())
        } else {
            None
        };

        // 2. Build structured JSON incident record
        let record = IncidentLogRecord {
            timestamp_ms,
            camera_id: camera_id.to_string(),
            anomaly_score,
            threat_level: assessment.threat_level,
            category: assessment.category,
            confidence: assessment.confidence,
            description: assessment.description.clone(),
            snapshot_path,
            inference_time_ms: assessment.inference_time_ms,
        };

        // 3. Append to incidents.jsonl
        let json_line = serde_json::to_string(&record)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .map_err(|e| anyhow!("Failed to open incident log file {:?}: {}", self.log_file_path, e))?;

        writeln!(file, "{}", json_line)?;

        Ok(record)
    }

    /// Returns the path to the JSONL log file.
    pub fn log_path(&self) -> &Path {
        &self.log_file_path
    }
}
