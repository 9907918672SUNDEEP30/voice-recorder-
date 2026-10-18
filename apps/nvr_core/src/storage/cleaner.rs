//! Automated FIFO Disk Space Manager
//! Prunes oldest continuous recordings when storage usage exceeds the configured quota (default: 90%),
//! while strictly preserving locked evidence files.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Summary of a cleanup run
#[derive(Debug, Clone, Default)]
pub struct CleanupReport {
    pub files_deleted: usize,
    pub bytes_freed: u64,
}

/// Disk Quota Cleaner Engine
pub struct StorageCleaner;

impl StorageCleaner {
    /// Prune oldest normal recordings if total directory bytes exceed quota_bytes * max_quota_pct / 100
    pub fn enforce_quota_bytes<P: AsRef<Path>>(
        storage_dir: P,
        quota_bytes: u64,
        max_quota_pct: u8,
    ) -> Result<CleanupReport> {
        let storage_dir = storage_dir.as_ref();
        if !storage_dir.exists() {
            return Ok(CleanupReport::default());
        }

        let mut report = CleanupReport::default();
        let mut recording_files = Vec::new();

        Self::collect_recordings(storage_dir, &mut recording_files)?;

        // Sort oldest first (FIFO pruning)
        recording_files.sort_by_key(|(_, modified, _)| *modified);

        // Calculate total size of recordings currently stored
        let mut total_bytes: u64 = recording_files.iter().map(|(_, _, size)| *size).sum();

        // Target threshold (e.g. 90% of allocated quota)
        let threshold_bytes = quota_bytes.saturating_mul(max_quota_pct as u64) / 100;
        let target_retention_bytes = quota_bytes.saturating_mul(80) / 100; // prune down to 80% watermark

        if total_bytes > threshold_bytes {
            eprintln!(
                "[NVR STORAGE CLEANER] Storage usage ({} MB) exceeded {}% threshold ({} MB). Pruning oldest recordings...",
                total_bytes / (1024 * 1024),
                max_quota_pct,
                threshold_bytes / (1024 * 1024)
            );

            for (path, _, size) in recording_files {
                if total_bytes <= target_retention_bytes {
                    break;
                }

                // Strictly preserve locked evidence files
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if filename.contains("evidence") || filename.contains("locked") {
                    continue;
                }

                if fs::remove_file(&path).is_ok() {
                    report.files_deleted += 1;
                    report.bytes_freed += size;
                    total_bytes = total_bytes.saturating_sub(size);
                }
            }

            eprintln!(
                "[NVR STORAGE CLEANER] Cleanup complete: freed {} MB ({} files deleted), current usage {} MB.",
                report.bytes_freed / (1024 * 1024),
                report.files_deleted,
                total_bytes / (1024 * 1024)
            );
        }

        Ok(report)
    }

    /// Prune oldest normal recordings based on percentage quota of default 50GB storage pool
    pub fn enforce_quota<P: AsRef<Path>>(storage_dir: P, max_quota_pct: u8) -> Result<CleanupReport> {
        // Default allocation: 50 GB storage quota per camera volume
        const DEFAULT_VOLUME_QUOTA_BYTES: u64 = 50 * 1024 * 1024 * 1024;
        Self::enforce_quota_bytes(storage_dir, DEFAULT_VOLUME_QUOTA_BYTES, max_quota_pct)
    }

    fn collect_recordings(dir: &Path, files: &mut Vec<(PathBuf, SystemTime, u64)>) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let _ = Self::collect_recordings(&path, files);
                } else if path.extension().and_then(|e| e.to_str()) == Some("mp4") {
                    if let Ok(meta) = entry.metadata() {
                        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let size = meta.len();
                        files.push((path, modified, size));
                    }
                }
            }
        }
        Ok(())
    }
}
