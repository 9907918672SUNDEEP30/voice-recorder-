use async_trait::async_trait;
use drm_guard::{DRMGuard, LicenseStatus, UsageMetricProvider};
use serde_json::{json, Map, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct TestCctvMetrics {
    camera_count: Arc<AtomicUsize>,
}

#[async_trait]
impl UsageMetricProvider for TestCctvMetrics {
    fn app_name(&self) -> &'static str {
        "cctv_guard"
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(
            "camera_count".to_string(),
            json!(self.camera_count.load(Ordering::Relaxed)),
        );
        map
    }
}

#[tokio::test]
async fn test_drm_guard_metered_lifecycle() {
    let metrics = Arc::new(TestCctvMetrics {
        camera_count: Arc::new(AtomicUsize::new(6)),
    });

    let drm = Arc::new(
        DRMGuard::builder()
            .client_id("test_client")
            .license_key("TEST-LICENSE-KEY")
            .metrics_provider(metrics.clone())
            .grace_period_hours(24)
            .build()
            .unwrap(),
    );

    // Initial state is unlicensed
    assert!(!drm.is_active().await);

    // 1. Ensure any preexisting cache file is removed
    let cache_path = std::env::temp_dir().join(".shambhavi_lease_cctv_guard.json");
    let _ = std::fs::remove_file(&cache_path);

    // 2. Perform heartbeat ping with no server & no cache -> STRICT FAIL CLOSED
    let res = drm.perform_heartbeat().await;
    assert!(res.is_err(), "Must fail closed when no server and no signed cache exist");
    assert!(!drm.is_active().await);

    // 3. Inject a valid Ed25519 server-signed lease into the disk cache
    use ed25519_dalek::{Signer, SigningKey};
    let seed = b"SHAMBHAVI_AI_LABS_ROOT_SEED_2026";
    let signing_key = SigningKey::from_bytes(seed);
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + 86400; // 24 hours ahead
    let nonce = "test_offline_nonce_99";
    let session_key_hex = hex::encode([0x42u8; 32]);
    let canonical_msg = format!(
        "test_client:{}:cctv_guard:{}:{}:{}",
        drm.machine_uuid(),
        expires_at,
        nonce,
        session_key_hex
    );
    let signature = signing_key.sign(canonical_msg.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());

    let cached_lease = drm_guard::CachedSignedLease {
        client_id: "test_client".to_string(),
        machine_uuid: drm.machine_uuid().to_string(),
        app_name: "cctv_guard".to_string(),
        expires_at,
        nonce: nonce.to_string(),
        session_key_hex: Some(session_key_hex),
        signature_hex: sig_hex,
    };
    std::fs::write(&cache_path, serde_json::to_string(&cached_lease).unwrap()).unwrap();

    // 4. Perform heartbeat ping now -> Offline grace succeeds via Ed25519 verification
    let res = drm.perform_heartbeat().await;
    assert!(res.is_ok(), "Must grant offline grace for valid Ed25519 signed lease");

    // Verified active
    assert!(drm.is_active().await);
    assert_eq!(drm.app_name(), "cctv_guard");
    assert_eq!(drm.machine_uuid().len(), 64);

    let info = drm.get_status_info().await;
    assert_eq!(info.status, LicenseStatus::Active);
    assert!(info.seconds_remaining > 80000);

    // Retrieve volatile key
    let key = drm.get_session_key().await;
    assert!(key.is_some());

    // Invalidation and zeroization
    drm.invalidate_and_zeroize(LicenseStatus::Expired).await;
    assert!(!drm.is_active().await);
    assert!(drm.get_session_key().await.is_none());

    // Clean up
    let _ = std::fs::remove_file(&cache_path);
}
