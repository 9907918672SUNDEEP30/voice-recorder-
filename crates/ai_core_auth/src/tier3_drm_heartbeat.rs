//! # Tier 3: Local Hardware & NVR Desktop Tools (DRM License Key Only)
//!
//! Target Apps: `apps/cctv_guard`, `apps/local_rag_engine`, `apps/dsp_recorder`, `apps/video_studio`, `apps/nvr_core`
//! Policy: Machine-Bound License Key & 24-Hour DRM Heartbeat (No User Passwords Required)
//! Mandate:
//!   1. Do NOT require username/password user authentication.
//!   2. Single valid License Key bound to machine hardware fingerprint (CPU ID + Motherboard Serial).
//!   3. 24-Hour DRM Heartbeat: Background check against Cloudflare Workers to verify active subscription status.
//!   4. Fail-Closed Locking: Lock software features immediately if subscription status is revoked or invalid.

use crate::hardware;
use crate::tier2_local_auth::EMBEDDED_ROOT_PUBLIC_KEY_HEX;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use zeroize::ZeroizeOnDrop;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmLease {
    pub license_key: String,
    pub machine_fingerprint: String,
    pub app_name: String,
    pub status: String, // "active", "revoked", "expired"
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signature_hex: String,
}

#[derive(ZeroizeOnDrop)]
pub struct ActiveHardwareSession {
    pub license_key: String,
    pub machine_fingerprint: String,
    pub is_locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub license_key: String,
    pub machine_fingerprint: String,
    pub app_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub lease: DrmLease,
    pub is_active: bool,
    pub message: Option<String>,
}

#[async_trait]
pub trait Tier3DrmHeartbeat: Send + Sync {
    /// Initial hardware license verification during setup/installation
    async fn install_license(&self, license_key: &str) -> Result<DrmLease>;

    /// Validates local lease offline without network dependency
    fn verify_local_lease(&self) -> Result<()>;

    /// Sends 24-hour periodic pulse to Cloudflare edge to renew lease and verify subscription
    async fn sync_24h_heartbeat(&self) -> Result<DrmLease>;

    /// Locks features immediately upon revocation
    fn lock_appliance(&self, reason: &str);
}

pub struct DrmHeartbeatManager {
    app_name: String,
    gateway_url: String,
    lease_cache_path: PathBuf,
    public_key: VerifyingKey,
    http_client: reqwest::Client,
}

impl DrmHeartbeatManager {
    pub fn new(app_name: &str, gateway_url: &str) -> Result<Self> {
        let pubkey_bytes = hex::decode(EMBEDDED_ROOT_PUBLIC_KEY_HEX)
            .map_err(|e| anyhow!("Invalid root public key hex: {}", e))?;
        let pubkey_array: [u8; 32] = pubkey_bytes
            .try_into()
            .map_err(|_| anyhow!("Public key length mismatch"))?;
        let public_key = VerifyingKey::from_bytes(&pubkey_array)
            .map_err(|e| anyhow!("Failed to parse root verifying key: {}", e))?;

        let cache_file_name = format!(".shambhavi_lease_{}.json", app_name);
        let lease_cache_path = PathBuf::from(cache_file_name);

        Ok(Self {
            app_name: app_name.to_string(),
            gateway_url: gateway_url.trim_end_matches('/').to_string(),
            lease_cache_path,
            public_key,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        })
    }

    pub fn with_custom_lease_path(mut self, path: PathBuf) -> Self {
        self.lease_cache_path = path;
        self
    }

    /// Custom public key override (useful for unit tests and key rotation)
    pub fn with_public_key(mut self, public_key: VerifyingKey) -> Self {
        self.public_key = public_key;
        self
    }

    /// Cryptographically validates a DrmLease signature and machine binding
    pub fn verify_lease_signature(&self, lease: &DrmLease) -> Result<()> {
        let current_fingerprint = hardware::get_machine_fingerprint();
        if lease.machine_fingerprint != current_fingerprint {
            return Err(anyhow!(
                "Hardware mismatch! License bound to {}, active machine is {}",
                lease.machine_fingerprint,
                current_fingerprint
            ));
        }

        if lease.app_name != self.app_name && lease.app_name != "all" {
            return Err(anyhow!(
                "App mismatch! License issued for {}, active app is {}",
                lease.app_name,
                self.app_name
            ));
        }

        if lease.status != "active" {
            return Err(anyhow!("License status is '{}'", lease.status));
        }

        if Utc::now() > lease.expires_at {
            return Err(anyhow!("Lease expired at {}", lease.expires_at));
        }

        // Verify Ed25519 root signature
        let payload = format!(
            "{}:{}:{}:{}",
            lease.license_key,
            lease.machine_fingerprint,
            lease.app_name,
            lease.expires_at.to_rfc3339()
        );

        let sig_bytes = hex::decode(&lease.signature_hex)
            .map_err(|e| anyhow!("Invalid signature hex: {}", e))?;
        let sig_array: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| anyhow!("Signature length mismatch"))?;
        let signature = Signature::from_bytes(&sig_array);

        self.public_key
            .verify_strict(payload.as_bytes(), &signature)
            .map_err(|e| anyhow!("Lease cryptographic signature mismatch: {}", e))?;

        Ok(())
    }
}

#[async_trait]
impl Tier3DrmHeartbeat for DrmHeartbeatManager {
    async fn install_license(&self, license_key: &str) -> Result<DrmLease> {
        let fingerprint = hardware::get_machine_fingerprint();
        println!(
            "🔒 [Tier 3 DRM] Binding License '{}' to Machine Fingerprint '{}'...",
            license_key, fingerprint
        );

        let endpoint = format!("{}/v1/drm/verify-heartbeat", self.gateway_url);

        let req_body = HeartbeatRequest {
            license_key: license_key.to_string(),
            machine_fingerprint: fingerprint,
            app_name: self.app_name.clone(),
        };

        let response = self
            .http_client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Gateway rejected license installation: {}", err_text));
        }

        let hb_res: HeartbeatResponse = response.json().await?;
        if !hb_res.is_active {
            return Err(anyhow!(
                "License is marked inactive/revoked: {:?}",
                hb_res.message
            ));
        }

        self.verify_lease_signature(&hb_res.lease)?;

        // Cache 24-hour lease on disk
        let lease_json = serde_json::to_string_pretty(&hb_res.lease)?;
        fs::write(&self.lease_cache_path, lease_json)?;

        println!(
            "✅ [Tier 3 DRM] License installed successfully. 24h rotating lease active until {}.",
            hb_res.lease.expires_at
        );
        Ok(hb_res.lease)
    }

    fn verify_local_lease(&self) -> Result<()> {
        if !self.lease_cache_path.exists() {
            return Err(anyhow!(
                "No hardware license lease found at {:?}. Software locked.",
                self.lease_cache_path
            ));
        }

        let lease_json = fs::read_to_string(&self.lease_cache_path)?;
        let lease: DrmLease = serde_json::from_str(&lease_json)?;

        self.verify_lease_signature(&lease)?;
        Ok(())
    }

    async fn sync_24h_heartbeat(&self) -> Result<DrmLease> {
        if !self.lease_cache_path.exists() {
            return Err(anyhow!("Cannot sync heartbeat: no existing license on disk"));
        }

        let lease_json = fs::read_to_string(&self.lease_cache_path)?;
        let existing_lease: DrmLease = serde_json::from_str(&lease_json)?;

        let endpoint = format!("{}/v1/drm/verify-heartbeat", self.gateway_url);

        let req_body = HeartbeatRequest {
            license_key: existing_lease.license_key.clone(),
            machine_fingerprint: existing_lease.machine_fingerprint.clone(),
            app_name: self.app_name.clone(),
        };

        let response = self
            .http_client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            self.lock_appliance(&format!("Heartbeat failed: {}", err_text));
            return Err(anyhow!("Heartbeat rejected by gateway: {}", err_text));
        }

        let hb_res: HeartbeatResponse = response.json().await?;
        if !hb_res.is_active {
            self.lock_appliance("Subscription revoked or expired at server");
            return Err(anyhow!("Subscription revoked by gateway"));
        }

        self.verify_lease_signature(&hb_res.lease)?;

        // Update local lease file with refreshed 24h window
        let updated_json = serde_json::to_string_pretty(&hb_res.lease)?;
        fs::write(&self.lease_cache_path, updated_json)?;

        println!(
            "💓 [Tier 3 DRM Heartbeat] 24-hour lease refreshed. Valid until {}.",
            hb_res.lease.expires_at
        );
        Ok(hb_res.lease)
    }

    fn lock_appliance(&self, reason: &str) {
        eprintln!(
            "🚨 [FAIL-CLOSED DRM LOCK] Software features locked for '{}': {}",
            self.app_name, reason
        );
        // Remove or corrupt lease to enforce fail-closed restart
        let _ = fs::remove_file(&self.lease_cache_path);
    }
}
