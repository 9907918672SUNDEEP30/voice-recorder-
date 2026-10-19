//! # DRM Guard Coordinator & Tokio Daemon Module
//!
//! Orchestrates machine binding, anti-tampering clock verification, dynamic
//! metered telemetry extraction, 24-hour cryptographic leases, and background
//! cryptographic Over-The-Air (OTA) binary updates.

use super::clock::MonotonicClockGuard;
use super::hardware::MachineFingerprint;
use super::heartbeat::{DrmStatusInfo, HeartbeatRequest, HeartbeatResponse, LicenseStatus, UpdateMetadata};
use super::session_key::VolatileSessionKey;
use super::telemetry::UsageMetricProvider;
use super::updater::OtaUpdater;
use anyhow::{anyhow, Result};
use chrono::Utc;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

struct InnerDrmState {
    status: LicenseStatus,
    expires_at: i64,
    session_key: Option<VolatileSessionKey>,
    last_verified_ts: i64,
    update_available: Option<UpdateMetadata>,
}

pub struct DRMGuard<M: UsageMetricProvider> {
    client_id: Arc<RwLock<String>>,
    license_key: Arc<RwLock<String>>,
    heartbeat_url: Option<String>,
    fingerprint: MachineFingerprint,
    clock_guard: MonotonicClockGuard,
    metrics_provider: Arc<M>,
    grace_period_hours: u32,
    updater: Arc<OtaUpdater>,
    state: Arc<RwLock<InnerDrmState>>,
    http_client: Client,
}

pub struct DRMGuardBuilder<M: UsageMetricProvider> {
    client_id: Option<String>,
    license_key: Option<String>,
    heartbeat_url: Option<String>,
    current_version: Option<String>,
    metrics_provider: Option<Arc<M>>,
    grace_period_hours: u32,
}

impl<M: UsageMetricProvider> Default for DRMGuardBuilder<M> {
    fn default() -> Self {
        Self {
            client_id: None,
            license_key: None,
            heartbeat_url: None,
            current_version: None,
            metrics_provider: None,
            grace_period_hours: 24,
        }
    }
}

impl<M: UsageMetricProvider> DRMGuardBuilder<M> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn client_id(mut self, id: impl Into<String>) -> Self {
        self.client_id = Some(id.into());
        self
    }

    pub fn license_key(mut self, key: impl Into<String>) -> Self {
        self.license_key = Some(key.into());
        self
    }

    pub fn heartbeat_url(mut self, url: Option<String>) -> Self {
        self.heartbeat_url = url;
        self
    }

    pub fn current_version(mut self, ver: impl Into<String>) -> Self {
        self.current_version = Some(ver.into());
        self
    }

    pub fn metrics_provider(mut self, provider: Arc<M>) -> Self {
        self.metrics_provider = Some(provider);
        self
    }

    pub fn grace_period_hours(mut self, hours: u32) -> Self {
        self.grace_period_hours = hours;
        self
    }

    pub fn build(self) -> Result<DRMGuard<M>> {
        let client_id = self.client_id.ok_or_else(|| anyhow!("Missing client_id"))?;
        let license_key = self.license_key.ok_or_else(|| anyhow!("Missing license_key"))?;
        let metrics_provider = self
            .metrics_provider
            .ok_or_else(|| anyhow!("Missing metrics_provider"))?;

        let version = self.current_version.unwrap_or_else(|| "0.1.0".to_string());
        let updater = Arc::new(OtaUpdater::new(version));

        let fingerprint = MachineFingerprint::generate();
        let clock_guard = MonotonicClockGuard::default();
        let now = Utc::now().timestamp();

        let initial_state = InnerDrmState {
            status: LicenseStatus::Unlicensed,
            expires_at: 0,
            session_key: None,
            last_verified_ts: now,
            update_available: None,
        };

        let heartbeat_url = self
            .heartbeat_url
            .or_else(|| std::env::var("SHAMBHAVI_DRM_URL").ok())
            .unwrap_or_else(|| DEFAULT_HEARTBEAT_URL.to_string());

        let mut client_builder = Client::builder().timeout(Duration::from_secs(10));
        if let Ok(ip) = "104.21.47.93".parse::<std::net::IpAddr>() {
            client_builder = client_builder.resolve(
                "shambhavi-drm-guard.shambhaviailabs.workers.dev",
                std::net::SocketAddr::new(ip, 443),
            );
        }
        let http_client = client_builder.build().unwrap_or_default();

        Ok(DRMGuard {
            client_id: Arc::new(RwLock::new(client_id)),
            license_key: Arc::new(RwLock::new(license_key)),
            heartbeat_url: Some(heartbeat_url),
            fingerprint,
            clock_guard,
            metrics_provider,
            grace_period_hours: self.grace_period_hours,
            updater,
            state: Arc::new(RwLock::new(initial_state)),
            http_client,
        })
    }
}

impl<M: UsageMetricProvider> DRMGuard<M> {
    pub fn builder() -> DRMGuardBuilder<M> {
        DRMGuardBuilder::new()
    }

    pub fn machine_uuid(&self) -> &str {
        &self.fingerprint.machine_uuid
    }

    pub fn fingerprint(&self) -> &MachineFingerprint {
        &self.fingerprint
    }

    pub async fn client_id(&self) -> String {
        self.client_id.read().await.clone()
    }

    pub async fn license_key(&self) -> String {
        self.license_key.read().await.clone()
    }

    pub async fn set_credentials(&self, client_id: impl Into<String>, license_key: impl Into<String>) {
        let mut cid = self.client_id.write().await;
        *cid = client_id.into();
        let mut lk = self.license_key.write().await;
        *lk = license_key.into();
    }

    pub fn app_name(&self) -> &'static str {
        self.metrics_provider.app_name()
    }

    pub fn current_version(&self) -> &str {
        self.updater.current_version()
    }

    pub fn grace_period_hours(&self) -> u32 {
        self.grace_period_hours
    }

    pub fn updater(&self) -> &Arc<OtaUpdater> {
        &self.updater
    }

    /// Verifies if the subscription lease is valid and active.
    pub async fn is_active(&self) -> bool {
        if !self.clock_guard.check_integrity() {
            self.invalidate_and_zeroize(LicenseStatus::ClockTamperingDetected).await;
            return false;
        }

        let st = self.state.read().await;
        let now = Utc::now().timestamp();

        (st.status == LicenseStatus::Active || st.status == LicenseStatus::ExpiringSoon)
            && st.expires_at > now
            && st.session_key.is_some()
    }

    /// Retrieves an ephemeral clone of the active volatile SessionKey.
    pub async fn get_session_key(&self) -> Option<VolatileSessionKey> {
        if !self.is_active().await {
            return None;
        }
        let st = self.state.read().await;
        st.session_key.clone()
    }

    /// Returns detailed diagnostic status info
    pub async fn get_status_info(&self) -> DrmStatusInfo {
        let st = self.state.read().await;
        let now = Utc::now().timestamp();
        let remaining = (st.expires_at - now).max(0);
        let is_valid = (st.status == LicenseStatus::Active || st.status == LicenseStatus::ExpiringSoon)
            && st.expires_at > now
            && st.session_key.is_some();
        let client_id = self.client_id.read().await.clone();

        DrmStatusInfo {
            client_id,
            app_name: self.metrics_provider.app_name().to_string(),
            current_version: self.updater.current_version().to_string(),
            machine_uuid: self.fingerprint.machine_uuid.clone(),
            status: st.status.clone(),
            expires_at: st.expires_at,
            seconds_remaining: remaining,
            is_valid,
            update_available: st.update_available.clone(),
        }
    }
}

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const ROOT_ED25519_PUBLIC_KEY_HEX: &str =
    "82d9651fdb75e85ab7e88c35440e4c7629b53b97c9fbfdf2335a6cb458af1400";

pub const DEFAULT_HEARTBEAT_URL: &str =
    "https://shambhavi-drm-guard.shambhaviailabs.workers.dev/api/heartbeat";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSignedLease {
    pub client_id: String,
    pub machine_uuid: String,
    pub app_name: String,
    pub expires_at: i64,
    pub nonce: String,
    pub session_key_hex: Option<String>,
    pub signature_hex: String,
}

pub fn verify_ed25519_signature(
    pubkey_hex: &str,
    canonical_msg: &str,
    sig_hex: &str,
) -> Result<()> {
    let pubkey_bytes = hex::decode(pubkey_hex).map_err(|e| anyhow!("Invalid public key hex: {}", e))?;
    if pubkey_bytes.len() != 32 {
        return Err(anyhow!("Invalid Ed25519 public key length (must be 32 bytes)"));
    }
    let mut pubkey_arr = [0u8; 32];
    pubkey_arr.copy_from_slice(&pubkey_bytes);
    let verifying_key = VerifyingKey::from_bytes(&pubkey_arr)
        .map_err(|e| anyhow!("Invalid Ed25519 verifying key point: {}", e))?;

    let sig_bytes = hex::decode(sig_hex).map_err(|e| anyhow!("Invalid signature hex: {}", e))?;
    if sig_bytes.len() != 64 {
        return Err(anyhow!("Invalid Ed25519 signature length (must be 64 bytes)"));
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    verifying_key
        .verify(canonical_msg.as_bytes(), &signature)
        .map_err(|e| anyhow!("Ed25519 cryptographic signature mismatch: {}", e))?;
    Ok(())
}

fn lease_cache_path(app_name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(".shambhavi_lease_{}.json", app_name))
}

impl<M: UsageMetricProvider> DRMGuard<M> {
    /// Executes immediate lease heartbeat cycle with Ed25519 signature verification
    pub async fn perform_heartbeat(&self) -> Result<()> {
        if !self.clock_guard.check_integrity() {
            self.invalidate_and_zeroize(LicenseStatus::ClockTamperingDetected).await;
            return Err(anyhow!("Clock tampering detected. Lease terminated."));
        }

        let now = Utc::now().timestamp();
        let nonce = format!("{:016x}", rand::random::<u64>());
        let metrics = self.metrics_provider.collect_metrics().await;
        let client_id = self.client_id.read().await.clone();
        let license_key = self.license_key.read().await.clone();

        let req = HeartbeatRequest {
            client_id: client_id.clone(),
            license_key: license_key.clone(),
            machine_uuid: self.fingerprint.machine_uuid.clone(),
            app_name: self.metrics_provider.app_name().to_string(),
            current_version: Some(self.updater.current_version().to_string()),
            timestamp: now,
            nonce: nonce.clone(),
            metrics,
        };

        if let Some(url) = &self.heartbeat_url {
            match self.http_client.post(url).json(&req).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let body: HeartbeatResponse = resp.json().await?;

                    // Check and process OTA binary updates
                    if let Some(update_meta) = &body.update {
                        {
                            let mut st = self.state.write().await;
                            st.update_available = Some(update_meta.clone());
                        }

                        if self.updater.is_newer_version(&update_meta.version_available)
                            && !self.updater.has_attempted(&update_meta.version_available)
                        {
                            self.updater.mark_attempted(&update_meta.version_available);
                            println!(
                                "[OTA UPDATER] New version available: v{} (Current: v{}) | Mandatory: {}",
                                update_meta.version_available,
                                self.updater.current_version(),
                                update_meta.mandatory
                            );

                            let updater_clone = self.updater.clone();
                            let meta_clone = update_meta.clone();

                            tokio::spawn(async move {
                                match updater_clone.download_and_verify(&meta_clone).await {
                                    Ok(bytes) => {
                                        if let Err(e) = updater_clone.apply_binary_swap(&bytes, None) {
                                            eprintln!("[OTA UPDATER] Failed to apply binary swap: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("[OTA UPDATER] Background download failed: {}", e);
                                    }
                                }
                            });
                        }
                    }

                    let canonical_msg = format!(
                        "{}:{}:{}:{}:{}:{}",
                        client_id,
                        req.machine_uuid,
                        req.app_name,
                        body.expires_at,
                        req.nonce,
                        body.session_key_hex.as_deref().unwrap_or("")
                    );

                    match body.status.as_str() {
                        "granted" => {
                            let sig_hex = body
                                .signature_hex
                                .as_deref()
                                .ok_or_else(|| anyhow!("Server response missing required signature_hex"))?;

                            // Strict Cryptographic Ed25519 Verification against Embedded Root Key
                            verify_ed25519_signature(ROOT_ED25519_PUBLIC_KEY_HEX, &canonical_msg, sig_hex)?;

                            // Cache verified lease to disk for offline grace
                            // M-02: Never persist session key to disk. Session keys reside exclusively in RAM.
                            let cached_lease = CachedSignedLease {
                                client_id: client_id.clone(),
                                machine_uuid: req.machine_uuid.clone(),
                                app_name: req.app_name.clone(),
                                expires_at: body.expires_at,
                                nonce: req.nonce.clone(),
                                session_key_hex: None,
                                signature_hex: sig_hex.to_string(),
                            };
                            if let Ok(json_str) = serde_json::to_string(&cached_lease) {
                                let _ = std::fs::write(lease_cache_path(&req.app_name), json_str);
                            }

                            self.apply_lease(body.expires_at, body.session_key_hex.as_deref()).await?;
                            println!(
                                "[DRM GUARD] Cryptographic Ed25519 lease verified & granted for app '{}' until {}",
                                req.app_name, body.expires_at
                            );
                            return Ok(());
                        }
                        "limit_exceeded" => {
                            self.invalidate_and_zeroize(LicenseStatus::LimitExceeded).await;
                            return Err(anyhow!("Licensing limit exceeded: {:?}", body.message));
                        }
                        _ => {
                            self.invalidate_and_zeroize(LicenseStatus::Revoked).await;
                            return Err(anyhow!("License revoked or invalid: {:?}", body.message));
                        }
                    }
                }
                Ok(resp) => {
                    let status_code = resp.status();
                    if let Ok(err_body) = resp.json::<HeartbeatResponse>().await {
                        if err_body.status == "limit_exceeded" {
                            self.invalidate_and_zeroize(LicenseStatus::LimitExceeded).await;
                            return Err(anyhow!("Licensing limit exceeded: {:?}", err_body.message));
                        } else if err_body.status == "revoked" {
                            self.invalidate_and_zeroize(LicenseStatus::Revoked).await;
                            return Err(anyhow!("License revoked: {:?}", err_body.message));
                        }
                    }
                    eprintln!("[DRM GUARD] Worker returned non-success HTTP status: {}", status_code);
                }
                Err(e) => {
                    eprintln!("[DRM GUARD] Network error connecting to licensing worker: {}. Checking offline grace...", e);
                }
            }
        }

        // Offline / Network Error Evaluation:
        // FAIL CLOSED: Only allow execution if a previously cryptographically verified server-signed lease exists on disk,
        // its signature is valid, and its expires_at timestamp is still in the future!
        let cache_file = lease_cache_path(self.metrics_provider.app_name());
        if let Ok(cache_data) = std::fs::read_to_string(&cache_file) {
            if let Ok(cached) = serde_json::from_str::<CachedSignedLease>(&cache_data) {
                // M-01: Verify app_name claim matches current application
                if cached.app_name != self.metrics_provider.app_name() {
                    eprintln!(
                        "[DRM GUARD SECURITY] App name claim mismatch! Expected: {}, Found: {}",
                        self.metrics_provider.app_name(),
                        cached.app_name
                    );
                    self.invalidate_and_zeroize(LicenseStatus::Revoked).await;
                    return Err(anyhow!("Lease app name claim mismatch. Tampering detected."));
                }

                let cached_canonical = format!(
                    "{}:{}:{}:{}:{}:{}",
                    cached.client_id,
                    cached.machine_uuid,
                    cached.app_name,
                    cached.expires_at,
                    cached.nonce,
                    cached.session_key_hex.as_deref().unwrap_or("")
                );

                if verify_ed25519_signature(ROOT_ED25519_PUBLIC_KEY_HEX, &cached_canonical, &cached.signature_hex).is_ok()
                    && cached.machine_uuid == self.fingerprint.machine_uuid
                    && cached.client_id == client_id
                    && cached.app_name == self.metrics_provider.app_name()
                {
                    if now < cached.expires_at {
                        self.apply_lease(cached.expires_at, cached.session_key_hex.as_deref()).await?;
                        println!(
                            "[DRM GUARD] Offline grace active. Verified server lease valid until: {} (Remaining: {}s)",
                            cached.expires_at, cached.expires_at - now
                        );
                        return Ok(());
                    } else {
                        self.invalidate_and_zeroize(LicenseStatus::Expired).await;
                        return Err(anyhow!(
                            "Server-signed lease expired at timestamp {}. Active connection required to renew.",
                            cached.expires_at
                        ));
                    }
                }
            }
        }

        // No valid server-signed lease exists -> Fail Closed
        self.invalidate_and_zeroize(LicenseStatus::Unlicensed).await;
        Err(anyhow!(
            "No cryptographically verified license lease found. Server connection required to activate."
        ))
    }

    async fn apply_lease(&self, expires_at: i64, key_hex: Option<&str>) -> Result<()> {
        let session_key = if let Some(hex_str) = key_hex {
            let bytes = hex::decode(hex_str).map_err(|e| anyhow!("Invalid key hex: {}", e))?;
            VolatileSessionKey::from_slice(&bytes)?
        } else {
            VolatileSessionKey::generate_random()
        };

        let mut st = self.state.write().await;
        st.status = LicenseStatus::Active;
        st.expires_at = expires_at;
        st.session_key = Some(session_key);
        st.last_verified_ts = Utc::now().timestamp();
        Ok(())
    }

    /// Immediately revokes license and scrubs volatile keys to zero
    pub async fn invalidate_and_zeroize(&self, status: LicenseStatus) {
        let mut st = self.state.write().await;
        st.status = status;
        st.expires_at = 0;
        st.session_key = None; // Dropped and automatically scrubbed with 0x00 by ZeroizeOnDrop
        eprintln!("[DRM GUARD] License invalidated ({:?}). Volatile memory zeroized.", st.status);
    }

    /// Dispatches proactive 10-minute health telemetry (wire faults, camera offline, storage)
    /// to the Cloudflare Worker `/api/telemetry/heartbeat-10m` endpoint.
    pub async fn send_10m_health_telemetry(
        &self,
        wire_faults: u32,
        offline_cameras: &[String],
        storage_pct: f64,
    ) -> Result<()> {
        let heartbeat_endpoint = self
            .heartbeat_url
            .as_deref()
            .unwrap_or(DEFAULT_HEARTBEAT_URL);
        let telemetry_url = heartbeat_endpoint.replace("/api/heartbeat", "/api/telemetry/heartbeat-10m");
        let client_id = self.client_id.read().await.clone();
        let license_key = self.license_key.read().await.clone();
        let metrics = self.metrics_provider.collect_metrics().await;

        let payload = serde_json::json!({
            "client_id": client_id,
            "machine_uuid": self.fingerprint.machine_uuid,
            "app_name": self.metrics_provider.app_name(),
            "wire_faults": wire_faults,
            "offline_cameras": offline_cameras,
            "storage_pct": storage_pct,
            "metrics": metrics
        });

        let resp = self
            .http_client
            .post(&telemetry_url)
            .header("Authorization", format!("Bearer {}", license_key))
            .json(&payload)
            .send()
            .await?;

        if resp.status().is_success() {
            println!("[DRM TELEMETRY] 10-min health telemetry successfully sent to Cloudflare edge.");
            Ok(())
        } else {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            eprintln!("[DRM TELEMETRY ERROR] Server responded with {}: {}", status, err_text);
            Err(anyhow!("Telemetry error {}: {}", status, err_text))
        }
    }

    /// Spawns the background Tokio heartbeat monitoring loop.
    pub fn start_background_daemon(self: Arc<Self>) {
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
                    eprintln!("[DRM HEARTBEAT ERROR] Cycle failed: {}", e);
                }

                // 3. Proactive 10-minute health telemetry dispatch (wire faults, camera offline, storage)
                let metrics = self.metrics_provider.collect_metrics().await;
                let wire_faults = metrics
                    .get("wire_faults")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                let offline_cameras: Vec<String> = metrics
                    .get("offline_cameras")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                let storage_pct = metrics
                    .get("storage_pct")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                if let Err(e) = self
                    .send_10m_health_telemetry(wire_faults, &offline_cameras, storage_pct)
                    .await
                {
                    eprintln!("[DRM TELEMETRY NOTICE] 10-min health report: {}", e);
                }
            }
        });
    }

    /// Returns currently discovered OTA update metadata, if any.
    pub async fn check_ota_update(&self) -> Option<UpdateMetadata> {
        let st = self.state.read().await;
        st.update_available.clone()
    }

    /// Sets or clears available OTA update metadata.
    pub async fn set_update_available(&self, update: Option<UpdateMetadata>) {
        let mut st = self.state.write().await;
        st.update_available = update;
    }
}
