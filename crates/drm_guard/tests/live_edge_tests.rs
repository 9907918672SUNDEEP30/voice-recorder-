//! # Live Cloudflare D1 & Worker Integration Verification Suite
//!
//! Validates end-to-end 24-hour Ed25519 cryptographic lease issuance,
//! 10-minute health telemetry dispatch with wire fault reporting,
//! and quota limit enforcement against the live Cloudflare Edge.

use async_trait::async_trait;
use drm_guard::{
    telemetry::CctvGuardMetrics,
    DRMGuard, LicenseStatus, UsageMetricProvider,
};
use serde_json::{json, Map, Value};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

struct GenericAppMetrics {
    app: &'static str,
    counter: Arc<AtomicU32>,
}

#[async_trait]
impl UsageMetricProvider for GenericAppMetrics {
    fn app_name(&self) -> &'static str {
        self.app
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(
            "active_units".to_string(),
            json!(self.counter.load(Ordering::Relaxed)),
        );
        map
    }
}

#[tokio::test]
async fn test_live_cloudflare_lease_all_apps() {
    let apps = [
        "cctv_guard",
        "nvr_core",
        "voice_agent",
        "dsp_recorder",
        "video_studio",
        "financial_engine",
        "manifestation_app",
        "content_engine",
    ];

    for app in apps {
        let metrics = Arc::new(GenericAppMetrics {
            app,
            counter: Arc::new(AtomicU32::new(1)),
        });

        let drm = DRMGuard::builder()
            .client_id("sme_client_001")
            .license_key("KEY-ENTERPRISE-2026")
            .metrics_provider(metrics.clone())
            .grace_period_hours(24)
            .build()
            .expect("Builder must succeed");

        println!("Testing live lease for app: {}", app);
        let res = drm.perform_heartbeat().await;
        assert!(
            res.is_ok(),
            "Live Cloudflare heartbeat failed for app '{}': {:?}",
            app,
            res.err()
        );

        assert!(
            drm.is_active().await,
            "App '{}' must have an active license status after heartbeat",
            app
        );

        let status = drm.get_status_info().await;
        assert_eq!(status.status, LicenseStatus::Active);
        assert!(status.seconds_remaining > 80000, "Lease must have ~24h remaining");
    }
}

#[tokio::test]
async fn test_live_cloudflare_10m_telemetry_with_wire_faults() {
    let metrics = Arc::new(CctvGuardMetrics::new(4));
    metrics
        .report_camera_status(vec!["CAM_02_LOBBY".to_string()], true)
        .await;

    let drm = DRMGuard::builder()
        .client_id("sme_client_001")
        .license_key("KEY-ENTERPRISE-2026")
        .metrics_provider(metrics.clone())
        .grace_period_hours(24)
        .build()
        .expect("Builder must succeed");

    // Send 10-minute proactive health telemetry
    let res = drm
        .send_10m_health_telemetry(
            1, // 1 wire fault detected
            &["CAM_02_LOBBY".to_string()],
            88.5, // 88.5% storage full
        )
        .await;

    assert!(
        res.is_ok(),
        "10-min health telemetry dispatch to Cloudflare edge failed: {:?}",
        res.err()
    );
}

#[tokio::test]
async fn test_live_cloudflare_quota_limit_enforcement() {
    // Client demo has a limit of max 4 cameras on cctv_guard
    let metrics = Arc::new(CctvGuardMetrics::new(10)); // Request 10 cameras (> 4 limit)

    let drm = DRMGuard::builder()
        .client_id("client_demo")
        .license_key("DEMO-KEY-2026")
        .metrics_provider(metrics.clone())
        .grace_period_hours(24)
        .build()
        .expect("Builder must succeed");

    // Heartbeat should reject with limit_exceeded
    let res = drm.perform_heartbeat().await;
    assert!(
        res.is_err(),
        "Must reject with limit_exceeded when exceeding subscription quota"
    );

    let status = drm.get_status_info().await;
    assert_eq!(
        status.status,
        LicenseStatus::LimitExceeded,
        "Status must transition to LimitExceeded and zeroize"
    );
}
