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

/// Real FFmpeg RTSP stream recorder for production-grade continuous MP4 recording.
/// Uses copy codec (-c copy) to achieve zero CPU transcoding overhead.
pub struct FfmpegRtspRecorder {
    pub camera_id: String,
    pub output_path: PathBuf,
    child: Option<tokio::process::Child>,
    stdin: Option<tokio::process::ChildStdin>,
}

impl FfmpegRtspRecorder {
    /// Launches a real FFmpeg recording process for the given RTSP URL.
    pub fn start_recording(rtsp_url: &str, camera_id: &str, output_dir: &std::path::Path) -> Result<Self> {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create recording directory {:?}", output_dir))?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let output_path = output_dir.join(format!("{}_{}.mp4", camera_id, timestamp));

        let mut cmd = tokio::process::Command::new("ffmpeg");
        cmd.args([
            "-rtsp_transport",
            "tcp",
            "-i",
            rtsp_url,
            "-c",
            "copy",
            "-f",
            "mp4",
            "-movflags",
            "+faststart",
            output_path.to_str().unwrap_or("output.mp4"),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn FFmpeg recording process for camera {}", camera_id))?;

        let stdin = child.stdin.take();

        Ok(Self {
            camera_id: camera_id.to_string(),
            output_path,
            child: Some(child),
            stdin,
        })
    }

    /// Gracefully stops recording by sending 'q\n' to FFmpeg's stdin.
    pub async fn stop_gracefully(&mut self) -> Result<()> {
        if let Some(mut stdin) = self.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(b"q\n").await;
            let _ = stdin.flush().await;
        }

        if let Some(mut child) = self.child.take() {
            let wait_res = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
            if wait_res.is_err() {
                let _ = child.kill().await;
            }
        }

        Ok(())
    }

    /// Validates that the recorded MP4 file is a valid video container via ffprobe.
    pub async fn validate_container(&self) -> Result<bool> {
        if !self.output_path.exists() {
            return Ok(false);
        }
        let output = tokio::process::Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                self.output_path.to_str().unwrap_or(""),
            ])
            .output()
            .await;

        match output {
            Ok(out) => Ok(out.status.success() && !out.stdout.is_empty()),
            Err(_) => {
                // If ffprobe is not installed, verify file size > 1024 bytes (header + mdat)
                let meta = fs::metadata(&self.output_path)?;
                Ok(meta.len() > 1024)
            }
        }
    }
}
