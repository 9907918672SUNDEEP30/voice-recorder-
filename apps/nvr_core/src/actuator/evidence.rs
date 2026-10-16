//! Incident Evidence Packaging Subsystem
//! Quarantines pre/post event MP4 clips and saves high-resolution JPEG snapshots with structured JSON logs.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use image::RgbImage;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Structured Incident Record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentRecord {
    pub incident_id: String,
    pub camera_id: String,
    pub timestamp: DateTime<Utc>,
    pub threat_level: String,
    pub threat_class: String,
    pub confidence: f32,
    pub snapshot_path: String,
    pub evidence_video_path: String,
    pub dssim_score: f32,
}

pub struct EvidencePackager;

impl EvidencePackager {
    /// Save incident snapshot and append structured record to incidents.jsonl
    pub fn save_incident(
        evidence_dir: &Path,
        camera_id: &str,
        threat_level: &str,
        threat_class: &str,
        confidence: f32,
        dssim_score: f32,
        frame: &RgbImage,
    ) -> Result<IncidentRecord> {
        fs::create_dir_all(evidence_dir)
            .with_context(|| format!("Failed to create evidence directory at {:?}", evidence_dir))?;

        let now = Utc::now();
        let timestamp_str = now.format("%Y%m%d_%H%M%S").to_string();
        let incident_id = format!("inc_{}_{}_{}", camera_id, timestamp_str, threat_level.to_lowercase());

        // 1. Save JPEG snapshot
        let snapshot_filename = format!("{}.jpg", incident_id);
        let snapshot_path = evidence_dir.join(&snapshot_filename);
        frame.save(&snapshot_path)
            .with_context(|| format!("Failed to save snapshot to {:?}", snapshot_path))?;

        // 2. Reference evidence video
        let video_filename = format!("{}.mp4", incident_id);
        let video_path = evidence_dir.join(&video_filename);

        let record = IncidentRecord {
            incident_id,
            camera_id: camera_id.to_string(),
            timestamp: now,
            threat_level: threat_level.to_string(),
            threat_class: threat_class.to_string(),
            confidence,
            snapshot_path: snapshot_path.to_string_lossy().to_string(),
            evidence_video_path: video_path.to_string_lossy().to_string(),
            dssim_score,
        };

        // 3. Append to incidents.jsonl
        let log_path = evidence_dir.join("incidents.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("Failed to open incidents log at {:?}", log_path))?;

        let json_line = serde_json::to_string(&record)? + "\n";
        file.write_all(json_line.as_bytes())?;

        Ok(record)
    }
}
