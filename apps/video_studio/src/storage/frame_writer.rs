use anyhow::{anyhow, Result};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::capture::VideoFrame;

/// Real-time frame recorder metrics.
#[derive(Debug, Default)]
pub struct RecordMetrics {
    pub total_frames_written: AtomicU64,
}

/// Disk frame serializer and recorder.
pub struct FrameWriter {
    output_dir: PathBuf,
    metrics: Arc<RecordMetrics>,
}

impl FrameWriter {
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Result<Self> {
        let dir = output_dir.as_ref().to_path_buf();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        Ok(Self {
            output_dir: dir,
            metrics: Arc::new(RecordMetrics::default()),
        })
    }

    /// Saves a single video frame as a lossless PNG snapshot.
    pub fn save_snapshot_png<P: AsRef<Path>>(frame: &VideoFrame, path: P) -> Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let img = frame.to_image_buffer();
        img.save(p).map_err(|e| anyhow!("Failed to save snapshot PNG: {}", e))?;
        Ok(())
    }

    /// Appends a video frame to an uncompressed raw binary/PPM frame sequence.
    pub fn save_frame_ppm(&self, frame: &VideoFrame, frame_index: u64) -> Result<PathBuf> {
        let file_name = format!("frame_{:06}.ppm", frame_index);
        let file_path = self.output_dir.join(file_name);

        let file = File::create(&file_path)?;
        let mut writer = BufWriter::with_capacity(128 * 1024, file);

        // Standard PPM P6 header: "P6\n<width> <height>\n255\n"
        writeln!(writer, "P6")?;
        writeln!(writer, "{} {}", frame.width, frame.height)?;
        writeln!(writer, "255")?;
        writer.write_all(&frame.data)?;
        writer.flush()?;

        self.metrics.total_frames_written.fetch_add(1, Ordering::Relaxed);
        Ok(file_path)
    }

    pub fn total_frames_written(&self) -> u64 {
        self.metrics.total_frames_written.load(Ordering::Relaxed)
    }
}
