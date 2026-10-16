//! Rolling 15-Minute MP4 Segment Manager
//! Organizes zero-copy continuous recording into partitioned directory structures.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

/// Metadata for an active or completed recording segment
#[derive(Debug, Clone)]
pub struct SegmentMetadata {
    pub camera_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub file_path: PathBuf,
    pub total_bytes_written: u64,
    pub is_evidence_locked: bool,
}

/// Rolling segment writer for continuous camera streams
#[derive(Debug)]
pub struct RollingSegmentWriter {
    pub camera_id: String,
    pub base_dir: PathBuf,
    pub segment_duration: Duration,
    current_segment: Option<SegmentMetadata>,
    active_file: Option<File>,
}

impl RollingSegmentWriter {
    pub fn new(camera_id: impl Into<String>, base_dir: PathBuf, segment_duration: Duration) -> Self {
        Self {
            camera_id: camera_id.into(),
            base_dir,
            segment_duration,
            current_segment: None,
            active_file: None,
        }
    }

    /// Write raw packet payload to current segment, rolling over if time exceeded
    pub fn write_packet(&mut self, payload: &[u8], is_keyframe: bool) -> Result<()> {
        let now = Utc::now();
        let should_rotate = match &self.current_segment {
            Some(seg) => {
                let elapsed = now.signed_duration_since(seg.start_time);
                // Rotate on keyframe once duration reached
                elapsed.to_std().unwrap_or(Duration::ZERO) >= self.segment_duration && is_keyframe
            }
            None => true,
        };

        if should_rotate {
            self.rotate_segment(now)?;
        }

        if let Some(file) = &mut self.active_file {
            file.write_all(payload).context("Failed to write packet to segment file")?;
            if let Some(seg) = &mut self.current_segment {
                seg.total_bytes_written += payload.len() as u64;
            }
        }

        Ok(())
    }

    /// Rotate and open a new segment file
    fn rotate_segment(&mut self, now: DateTime<Utc>) -> Result<()> {
        // Close previous segment
        if let Some(mut prev) = self.current_segment.take() {
            prev.end_time = Some(now);
            if let Some(mut file) = self.active_file.take() {
                let _ = file.flush();
            }
        }

        // Prepare new segment directory: <base_dir>/<cam_id>/YYYY-MM-DD/
        let date_folder = now.format("%Y-%m-%d").to_string();
        let target_dir = self.base_dir.join(&self.camera_id).join(date_folder);
        fs::create_dir_all(&target_dir)
            .with_context(|| format!("Failed to create recording directory {:?}", target_dir))?;

        let filename = format!("segment_{}.mp4", now.format("%H-%M-%S"));
        let file_path = target_dir.join(filename);

        let file = File::create(&file_path)
            .with_context(|| format!("Failed to create segment file {:?}", file_path))?;

        self.current_segment = Some(SegmentMetadata {
            camera_id: self.camera_id.clone(),
            start_time: now,
            end_time: None,
            file_path,
            total_bytes_written: 0,
            is_evidence_locked: false,
        });
        self.active_file = Some(file);

        Ok(())
    }

    /// Finish and flush the active segment
    pub fn finish(&mut self) {
        if let Some(mut file) = self.active_file.take() {
            let _ = file.flush();
        }
        if let Some(seg) = &mut self.current_segment {
            seg.end_time = Some(Utc::now());
        }
    }
}
