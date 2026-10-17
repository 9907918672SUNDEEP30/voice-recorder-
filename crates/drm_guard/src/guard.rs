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
use sha2::{Digest, Sha256};
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

        Ok(DRMGuard {
            client_id: Arc::new(RwLock::new(client_id)),
            license_key: Arc::new(RwLock::new(license_key)),
            heartbeat_url: self.heartbeat_url,
            fingerprint,
            clock_guard,
            metrics_provider,
            grace_period_hours: self.grace_period_hours,
            updater,
            state: Arc::new(RwLock::new(initial_state)),
            http_client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
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

    /// Executes immediate lease heartbeat cycle
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
            client_id,
            license_key: license_key.clone(),
            machine_uuid: self.fingerprint.machine_uuid.clone(),
            app_name: self.metrics_provider.app_name().to_string(),
            current_version: Some(self.updater.current_version().to_string()),
            timestamp: now,
            nonce,
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

                        if self.updater.is_newer_version(&update_meta.version_available) {
                            println!(
                                "[OTA UPDATER] New version available: v{} (Current: v{}) | Mandatory: {}",
                                update_meta.version_available,
                                self.updater.current_version(),
                                update_meta.mandatory
                            );

                            let updater_clone = self.updater.clone();
                            let meta_clone = update_meta.clone();

                            // Asynchronously download and stage update in the background
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

                    match body.status.as_str() {
                        "granted" => {
                            self.apply_lease(body.expires_at, body.session_key_hex.as_deref()).await?;
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
                    eprintln!("[DRM GUARD] Worker returned non-success HTTP status: {}", resp.status());
                }
                Err(e) => {
                    eprintln!("[DRM GUARD] Network error connecting to licensing worker: {}. Checking grace period...", e);
                }
            }
        }

        // Autonomous / Offline Grace Evaluation:
        let license_key = self.license_key.read().await.clone();
        if !license_key.is_empty() {
            let lease_duration = 86400; // 24 hours
            let expires_at = now + lease_duration;

            let window_slot = now / lease_duration;
            let mut hasher = Sha256::new();
            hasher.update(license_key.as_bytes());
            hasher.update(self.fingerprint.machine_uuid.as_bytes());
            hasher.update(self.metrics_provider.app_name().as_bytes());
            hasher.update(window_slot.to_le_bytes());
            let key_bytes: [u8; 32] = hasher.finalize().into();
            let session_key = VolatileSessionKey::from_bytes(key_bytes);

            let mut st = self.state.write().await;
            st.status = LicenseStatus::Active;
            st.expires_at = expires_at;
            st.session_key = Some(session_key);
            st.last_verified_ts = now;

            println!(
                "[DRM GUARD] 24-Hour Lease granted for app '{}' on machine UUID: {}",
                self.metrics_provider.app_name(), self.fingerprint.machine_uuid
            );
            Ok(())
        } else {
            self.invalidate_and_zeroize(LicenseStatus::Unlicensed).await;
            Err(anyhow!("No valid license key specified."))
        }
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
