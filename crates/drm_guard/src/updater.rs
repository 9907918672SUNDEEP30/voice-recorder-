//! # Native Rust OTA Auto-Update & Binary Swapper Engine
//!
//! Provides cryptographic Over-The-Air (OTA) binary updates, background downloading,
//! SHA-256 integrity verification, and cross-platform atomic binary swapping.

use super::heartbeat::UpdateMetadata;
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtaStatus {
    UpToDate,
    UpdateAvailable(UpdateMetadata),
    Staged { version: String, path: PathBuf },
    Failed(String),
}

pub const MAX_DOWNLOAD_BYTES: usize = 150 * 1024 * 1024; // 150 Megabytes maximum

#[derive(Clone)]
pub struct OtaUpdater {
    current_version: String,
    http_client: Client,
    staging_dir: Option<PathBuf>,
    attempted_versions: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
}

impl OtaUpdater {
    pub fn new(current_version: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            http_client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            staging_dir: None,
            attempted_versions: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
        }
    }

    pub fn with_staging_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.staging_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    pub fn staging_dir(&self) -> Option<&Path> {
        self.staging_dir.as_deref()
    }

    pub fn has_attempted(&self, version: &str) -> bool {
        if let Ok(set) = self.attempted_versions.lock() {
            set.contains(version)
        } else {
            false
        }
    }

    pub fn mark_attempted(&self, version: &str) {
        if let Ok(mut set) = self.attempted_versions.lock() {
            set.insert(version.to_string());
        }
    }

    /// Evaluates if an available version is strictly newer than current version
    pub fn is_newer_version(&self, available: &str) -> bool {
        Self::compare_semver(available, &self.current_version) > 0
    }

    /// Performs semantic version comparison: returns 1 if v1 > v2, -1 if v1 < v2, 0 if equal
    pub fn compare_semver(v1: &str, v2: &str) -> i32 {
        let clean1 = v1.trim().trim_start_matches('v');
        let clean2 = v2.trim().trim_start_matches('v');

        let p1: Vec<u64> = clean1.split('.').filter_map(|s| s.parse().ok()).collect();
        let p2: Vec<u64> = clean2.split('.').filter_map(|s| s.parse().ok()).collect();

        let max_len = p1.len().max(p2.len());
        for i in 0..max_len {
            let num1 = p1.get(i).copied().unwrap_or(0);
            let num2 = p2.get(i).copied().unwrap_or(0);
            if num1 > num2 {
                return 1;
            } else if num1 < num2 {
                return -1;
            }
        }
        0
    }

    /// Computes SHA-256 hash of a byte slice and returns lower hex
    pub fn compute_sha256(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Verifies cryptographic SHA-256 checksum of raw binary data
    pub fn verify_integrity(bytes: &[u8], expected_sha256: &str) -> bool {
        let computed = Self::compute_sha256(bytes);
        computed.eq_ignore_ascii_case(expected_sha256.trim())
    }

    /// Downloads and stages update binary in the background, validating SHA-256 integrity and Ed25519 signature
    pub async fn download_and_verify(&self, metadata: &UpdateMetadata) -> Result<Vec<u8>> {
        self.mark_attempted(&metadata.version_available);

        if !self.is_newer_version(&metadata.version_available) {
            return Err(anyhow!(
                "Available version '{}' is not newer than current '{}'",
                metadata.version_available,
                self.current_version
            ));
        }

        let url = &metadata.download_url;
        let is_secure = url.starts_with("https://")
            || url.starts_with("http://127.0.0.1")
            || url.starts_with("http://localhost");
        if !is_secure {
            return Err(anyhow!(
                "Insecure download URL rejected. OTA updates must be downloaded over HTTPS: {}",
                url
            ));
        }

        println!(
            "[OTA UPDATER] Securely downloading update binary v{} from {}",
            metadata.version_available, metadata.download_url
        );

        let mut response = self
            .http_client
            .get(&metadata.download_url)
            .send()
            .await
            .context("Failed to send update download request")?;

        if !response.status().is_success() {
            return Err(anyhow!("Update server returned HTTP error: {}", response.status()));
        }

        if let Some(content_len) = response.content_length() {
            if content_len > MAX_DOWNLOAD_BYTES as u64 {
                return Err(anyhow!(
                    "Update payload exceeds maximum allowed limit ({} MB)",
                    MAX_DOWNLOAD_BYTES / (1024 * 1024)
                ));
            }
        }

        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.context("Error reading response chunk")? {
            if bytes.len() + chunk.len() > MAX_DOWNLOAD_BYTES {
                return Err(anyhow!(
                    "Update binary download exceeded 150 MB safety threshold. Download aborted."
                ));
            }
            bytes.extend_from_slice(&chunk);
        }

        // 1. Cryptographic SHA-256 integrity validation
        if !Self::verify_integrity(&bytes, &metadata.sha256) {
            return Err(anyhow!(
                "Cryptographic integrity mismatch! Expected SHA-256: {}, Computed: {}",
                metadata.sha256,
                Self::compute_sha256(&bytes)
            ));
        }

        // 2. Mandatory Ed25519 signature verification — NO optional path, NO feature flag.
        //    An absent or invalid signature unconditionally aborts the update and deletes
        //    the downloaded bytes. SHA-256 alone is insufficient: a supply-chain attacker
        //    can provide a matching hash for a malicious binary.
        let sig_hex = metadata.signature_hex.as_deref().ok_or_else(|| {
            anyhow!(
                "[OTA SECURITY] CRITICAL: Update v{} rejected — Ed25519 signature absent. \
                All OTA binaries MUST carry a server-signed Ed25519 signature. \
                Unsigned binary discarded.",
                metadata.version_available
            )
        })?;

        crate::guard::verify_ed25519_signature(
            crate::guard::ROOT_ED25519_PUBLIC_KEY_HEX,
            &metadata.sha256,
            sig_hex,
        )
        .context(
            "[OTA SECURITY] Ed25519 signature verification FAILED — binary rejected and discarded. \
            Possible supply-chain tampering detected.",
        )?;

        tracing::info!(
            "[OTA UPDATER] ✅ Ed25519 signature verified for v{} ({} bytes)",
            metadata.version_available,
            bytes.len()
        );

        println!(
            "[OTA UPDATER] SHA-256 integrity verified successfully for v{} ({} bytes)",
            metadata.version_available,
            bytes.len()
        );

        Ok(bytes)
    }

    /// Performs cross-platform atomic binary swapping with rollback protection
    pub fn apply_binary_swap(&self, new_binary_bytes: &[u8], target_exe: Option<&Path>) -> Result<PathBuf> {
        let current_exe = match target_exe {
            Some(p) => p.to_path_buf(),
            None => std::env::current_exe().context("Failed to resolve current running executable path")?,
        };

        let parent_dir = current_exe
            .parent()
            .ok_or_else(|| anyhow!("Cannot determine parent directory of current executable"))?;

        let staged_new = parent_dir.join(format!(".update_new_{}.tmp", std::process::id()));
        let _backup_old = parent_dir.join(format!("{}.old", current_exe.file_name().and_then(|n| n.to_str()).unwrap_or("app")));

        // 1. Write the new binary to a temporary staging file first
        std::fs::write(&staged_new, new_binary_bytes)
            .context("Failed to write new binary to staging path")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&staged_new, std::fs::Permissions::from_mode(0o755));
        }

        // 2. Cross-platform atomic swap
        #[cfg(target_os = "windows")]
        {
            // Windows running .exe lock bypass:
            // Windows locks running files against deletion/overwriting, but allows RENAMING!
            // Step A: Remove any previous .old backup if present
            if backup_old.exists() {
                let _ = std::fs::remove_file(&backup_old);
            }

            // Step B: Rename running executable to .old
            std::fs::rename(&current_exe, &backup_old)
                .context("Failed to rename currently running executable to .old")?;

            // Step C: Move new binary into place
            if let Err(e) = std::fs::rename(&staged_new, &current_exe) {
                // Rollback if placing new binary fails
                let _ = std::fs::rename(&backup_old, &current_exe);
                let _ = std::fs::remove_file(&staged_new);
                return Err(anyhow!("Failed to swap new binary into place, rolled back: {}", e));
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix, atomic rename directly replaces the inode
            if let Err(e) = std::fs::rename(&staged_new, &current_exe) {
                let _ = std::fs::remove_file(&staged_new);
                return Err(anyhow!("Failed to atomically replace executable: {}", e));
            }
        }

        println!(
            "[OTA UPDATER] Atomic binary swap successful. Updated binary in place: {:?}",
            current_exe
        );
        Ok(current_exe)
    }
}
