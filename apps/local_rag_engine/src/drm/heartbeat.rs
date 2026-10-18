//! # 24-Hour Cloudflare DRM Heartbeat & Lease Guard
//!
//! Enforces 24-hour cryptographically signed DRM leases, machine binding,
//! anti-clock-tampering protection, and instant memory zeroization upon lease expiry.

use super::machine::MachineFingerprint;
use anyhow::{anyhow, Result};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use vector_store::SessionKey;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseStatus {
    Active,
    ExpiringSoon,
    Expired,
    Revoked,
    HardwareMismatch,
    ClockTamperingDetected,
    Unlicensed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub client_id: String,
    pub license_key: String,
    pub machine_uuid: String,
    pub timestamp: i64,
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub status: String,
    pub expires_at: i64,
    pub session_key_hex: Option<String>,
    pub signature_hex: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmInfo {
    pub client_id: String,
    pub machine_uuid: String,
    pub status: LicenseStatus,
    pub expires_at: i64,
    pub seconds_remaining: i64,
    pub is_valid: bool,
}

struct InnerDrmState {
    status: LicenseStatus,
    expires_at: i64,
    session_key: Option<SessionKey>,
    last_verified_ts: i64,
}

pub struct DrmGuard {
    client_id: String,
    license_key: String,
    worker_url: Option<String>,
    fingerprint: MachineFingerprint,
    state: Arc<RwLock<InnerDrmState>>,
    last_monotonic_time: Arc<AtomicI64>,
    http_client: Client,
}

impl DrmGuard {
    pub fn new(
        client_id: String,
        license_key: String,
        worker_url: Option<String>,
    ) -> Self {
        let fingerprint = MachineFingerprint::generate();
        let now = Utc::now().timestamp();

        let initial_state = InnerDrmState {
            status: LicenseStatus::Unlicensed,
            expires_at: 0,
            session_key: None,
            last_verified_ts: now,
        };

        Self {
            client_id,
            license_key,
            worker_url,
            fingerprint,
            state: Arc::new(RwLock::new(initial_state)),
            last_monotonic_time: Arc::new(AtomicI64::new(now)),
            http_client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn machine_uuid(&self) -> &str {
        &self.fingerprint.machine_uuid
    }

    /// Checks if license is currently valid and active.
    pub async fn is_active(&self) -> bool {
        self.verify_clock_integrity().await;
        let st = self.state.read().await;
        let now = Utc::now().timestamp();
        (st.status == LicenseStatus::Active || st.status == LicenseStatus::ExpiringSoon)
            && st.expires_at > now
            && st.session_key.is_some()
    }

    /// Retrieves a clone of the ephemeral SessionKey if lease is active.
    pub async fn get_session_key(&self) -> Option<SessionKey> {
        if !self.is_active().await {
            return None;
        }
        let st = self.state.read().await;
        st.session_key.clone()
    }

    /// Retrieves detailed status snapshot for UI and diagnostic reporting.
    pub async fn get_status_info(&self) -> DrmInfo {
        let st = self.state.read().await;
        let now = Utc::now().timestamp();
        let remaining = (st.expires_at - now).max(0);
        let is_valid = (st.status == LicenseStatus::Active || st.status == LicenseStatus::ExpiringSoon)
            && st.expires_at > now
            && st.session_key.is_some();

        DrmInfo {
            client_id: self.client_id.clone(),
            machine_uuid: self.fingerprint.machine_uuid.clone(),
            status: st.status.clone(),
            expires_at: st.expires_at,
            seconds_remaining: remaining,
            is_valid,
        }
    }

    /// Validates system monotonic time to prevent local backdating clock exploits.
    async fn verify_clock_integrity(&self) -> bool {
        let now = Utc::now().timestamp();
        let last = self.last_monotonic_time.load(Ordering::SeqCst);

        // If clock was rewound backwards by more than 5 minutes (300 seconds), flag tampering
        if now < last - 300 {
            eprintln!("[DRM SECURITY ALERT] Clock tampering detected! System time moved backwards.");
            let mut st = self.state.write().await;
            st.status = LicenseStatus::ClockTamperingDetected;
            st.session_key = None; // Zeroize
            return false;
        }

        if now > last {
            self.last_monotonic_time.store(now, Ordering::SeqCst);
        }
        true
    }

    /// Triggers immediate heartbeat ping to Cloudflare Worker or local offline lease validation.
    pub async fn perform_heartbeat(&self) -> Result<()> {
        if !self.verify_clock_integrity().await {
            return Err(anyhow!("Clock tampering detected. License invalidated."));
        }

        let now = Utc::now().timestamp();
        let nonce = format!("{:x}", rand::random::<u64>());

        let req = HeartbeatRequest {
            client_id: self.client_id.clone(),
            license_key: self.license_key.clone(),
            machine_uuid: self.fingerprint.machine_uuid.clone(),
            timestamp: now,
            nonce: nonce.clone(),
        };

        if let Some(url) = &self.worker_url {
            match self.http_client.post(url).json(&req).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let body: HeartbeatResponse = resp.json().await?;
                    if body.status == "granted" {
                        self.apply_lease(body.expires_at, body.session_key_hex.as_deref()).await?;
                        return Ok(());
                    } else {
                        self.invalidate_and_zeroize(LicenseStatus::Revoked).await;
                        return Err(anyhow!("DRM Worker refused lease: {:?}", body.message));
                    }
                }
                Ok(resp) => {
                    eprintln!("[DRM WARNING] Cloudflare Worker returned status: {}", resp.status());
                }
                Err(e) => {
                    eprintln!("[DRM WARNING] Worker connection failed: {}. Evaluating offline grace period...", e);
                }
            }
        }

        // Offline / Autonomous Evaluation:
        // FAIL-CLOSED: No arbitrary string may ever generate a valid session key.
        // Offline execution is strictly permitted ONLY if an unexpired, server-signed lease
        // with valid Ed25519 root signature is present on disk for this machine.
        let lease_path = std::env::temp_dir().join(".shambhavi_lease_local_rag_engine.json");
        if let Ok(cache_data) = std::fs::read_to_string(&lease_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cache_data) {
                let expires_at = val["expires_at"].as_i64().unwrap_or(0);
                let machine_uuid = val["machine_uuid"].as_str().unwrap_or("");
                let app_name = val["app_name"].as_str().unwrap_or("");
                let sig_hex = val["signature_hex"].as_str().unwrap_or("");

                let canonical_msg = format!(
                    "{}:{}:{}:{}:{}:",
                    val["client_id"].as_str().unwrap_or(""),
                    machine_uuid,
                    app_name,
                    expires_at,
                    val["nonce"].as_str().unwrap_or("")
                );

                const ROOT_KEY: &str = "82d9651fdb75e85ab7e88c35440e4c7629b53b97c9fbfdf2335a6cb458af1400";
                let sig_valid = if let (Ok(pub_b), Ok(sig_b)) = (hex::decode(ROOT_KEY), hex::decode(sig_hex)) {
                    if pub_b.len() == 32 && sig_b.len() == 64 {
                        let mut pk = [0u8; 32];
                        let mut sk = [0u8; 64];
                        pk.copy_from_slice(&pub_b);
                        sk.copy_from_slice(&sig_b);
                        if let (Ok(vk), sig) = (ed25519_dalek::VerifyingKey::from_bytes(&pk), ed25519_dalek::Signature::from_bytes(&sk)) {
                            use ed25519_dalek::Verifier;
                            vk.verify(canonical_msg.as_bytes(), &sig).is_ok()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if sig_valid && machine_uuid == self.fingerprint.machine_uuid && app_name == "local_rag_engine" && expires_at > now {
                    self.apply_lease(expires_at, None).await?;
                    println!(
                        "[DRM GUARD] Verified server-signed offline lease active until: {} (Remaining: {}s)",
                        expires_at, expires_at - now
                    );
                    return Ok(());
                }
            }
        }

        self.invalidate_and_zeroize(LicenseStatus::Expired).await;
        Err(anyhow!(
            "[DRM SECURITY] Fail-Closed: No valid server-signed Ed25519 lease available. \
            Connection to licensing endpoint required for initial activation or lease renewal."
        ))
    }

    async fn apply_lease(&self, expires_at: i64, key_hex: Option<&str>) -> Result<()> {
        let session_key = if let Some(hex_str) = key_hex {
            let bytes = hex::decode(hex_str).map_err(|e| anyhow!("Invalid key hex: {}", e))?;
            SessionKey::from_slice(&bytes)?
        } else {
            SessionKey::generate_random()
        };

        let mut st = self.state.write().await;
        st.status = LicenseStatus::Active;
        st.expires_at = expires_at;
        st.session_key = Some(session_key);
        st.last_verified_ts = Utc::now().timestamp();
        Ok(())
    }

    /// Immediate zeroization and lease revocation.
    pub async fn invalidate_and_zeroize(&self, status: LicenseStatus) {
        let mut st = self.state.write().await;
        st.status = status;
        st.expires_at = 0;
        st.session_key = None; // Dropped and automatically zeroized by zeroize crate
        eprintln!("[DRM GUARD] Session key zeroized in memory. Local encrypted vault locked.");
    }

    /// Spawns the background Tokio heartbeat monitoring loop.
    pub fn spawn_heartbeat_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(600)); // Every 10 minutes
            loop {
                interval.tick().await;

                // 1. Check expiration
                {
                    let now = Utc::now().timestamp();
                    let mut st = self.state.write().await;
                    if st.expires_at > 0 && now >= st.expires_at {
                        eprintln!("[DRM NOTICE] 24-hour lease expired. Zeroizing active session keys.");
                        st.status = LicenseStatus::Expired;
                        st.session_key = None;
                    } else if st.expires_at > 0 && (st.expires_at - now) < 3600 {
                        st.status = LicenseStatus::ExpiringSoon;
                    }
                }

                // 2. Refresh heartbeat
                if let Err(e) = self.perform_heartbeat().await {
                    eprintln!("[DRM HEARTBEAT ERROR] Refresh cycle failed: {}", e);
                }
            }
        });
    }
}
