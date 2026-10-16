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

    // Perform heartbeat ping (autonomous evaluation)
    let res = drm.perform_heartbeat().await;
    assert!(res.is_ok());

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
}
