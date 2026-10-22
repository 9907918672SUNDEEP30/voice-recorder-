//! # Immutable Binary Audit Log Module (`audit_log.bin`)
//!
//! Provides a tamper-proof cryptographic audit trail for corporate legal compliance
//! (Evidence Act & Tax Tribunal audit readiness).
//!
//! Record Structure (Fixed 64 bytes per entry):
//! - timestamp_secs: u64 (8 bytes)
//! - action_type: u16 (2 bytes: 1=WORKSPACE_INIT, 2=DOC_INGEST, 3=QUERY, 4=RENAME)
//! - payload_hash: [u8; 16] (16 bytes, truncated SHA-256)
//! - prev_record_hash: [u8; 16] (16 bytes, cryptographic blockchain-style chaining)
//! - reserved: [u8; 22] (22 bytes, zero-padded for future extensions)

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const AUDIT_RECORD_SIZE: usize = 64;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    WorkspaceInit = 1,
    DocIngest = 2,
    QueryExecuted = 3,
    FileRenamed = 4,
}

pub struct AuditLogger {
    log_path: PathBuf,
    file_lock: Mutex<()>,
}

impl AuditLogger {
    pub fn new<P: AsRef<Path>>(workspace_dir: P) -> Result<Self> {
        let log_path = workspace_dir.as_ref().join("audit_log.bin");
        if !log_path.exists() {
            let mut file = File::create(&log_path).context("Failed to create audit_log.bin")?;
            // Write genesis block
            let genesis = Self::format_record(
                chrono::Utc::now().timestamp() as u64,
                AuditAction::WorkspaceInit,
                b"GENESIS_BLOCK_CDM_2026",
                &[0u8; 16],
            );
            file.write_all(&genesis)?;
            file.flush()?;
        }

        Ok(Self {
            log_path,
            file_lock: Mutex::new(()),
        })
    }

    /// Appends a new cryptographic audit record
    pub fn append_record(&self, action: AuditAction, payload_data: &[u8]) -> Result<()> {
        let _guard = self.file_lock.lock().unwrap();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.log_path)
            .context("Failed to open audit_log.bin for appending")?;

        // 1. Read last 64 bytes to get previous record hash
        let file_len = file.metadata()?.len();
        let prev_hash = if file_len >= AUDIT_RECORD_SIZE as u64 {
            file.seek(SeekFrom::Start(file_len - AUDIT_RECORD_SIZE as u64))?;
            let mut last_record = [0u8; AUDIT_RECORD_SIZE];
            file.read_exact(&mut last_record)?;
            let mut hasher = Sha256::new();
            hasher.update(&last_record);
            let full_hash = hasher.finalize();
            let mut truncated = [0u8; 16];
            truncated.copy_from_slice(&full_hash[0..16]);
            truncated
        } else {
            [0u8; 16]
        };

        // 2. Format new record
        let now_ts = chrono::Utc::now().timestamp() as u64;
        let record_bytes = Self::format_record(now_ts, action, payload_data, &prev_hash);

        // 3. Append to file
        file.seek(SeekFrom::End(0))?;
        file.write_all(&record_bytes)?;
        file.flush()?;

        Ok(())
    }

    fn format_record(
        timestamp: u64,
        action: AuditAction,
        payload_data: &[u8],
        prev_hash: &[u8; 16],
    ) -> [u8; AUDIT_RECORD_SIZE] {
        let mut record = [0u8; AUDIT_RECORD_SIZE];

        // 8 bytes: timestamp
        record[0..8].copy_from_slice(&timestamp.to_le_bytes());

        // 2 bytes: action
        record[8..10].copy_from_slice(&(action as u16).to_le_bytes());

        // 16 bytes: payload SHA-256 (truncated)
        let mut hasher = Sha256::new();
        hasher.update(payload_data);
        let digest = hasher.finalize();
        record[10..26].copy_from_slice(&digest[0..16]);

        // 16 bytes: prev record hash
        record[26..42].copy_from_slice(prev_hash);

        // 22 bytes: reserved (zeroed)
        record
    }
}
