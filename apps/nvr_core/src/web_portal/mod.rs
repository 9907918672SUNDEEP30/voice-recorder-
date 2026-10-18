//! # Embedded TP-Link Style Web Portal Module
//!
//! Provides an embedded HTTP web server (port 8080) for headless NVR hardware appliances,
//! allowing installers and customers to configure cameras, 24-hour cryptographic leases,
//! wire fault monitoring, and background Jio TV style OTA auto-updating.

pub mod html;
pub mod routes;

use crate::config::NvrConfig;
use anyhow::Result;
use drm_guard::{CctvGuardMetrics, DRMGuard};
use routes::{build_router, WebPortalState};
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Spawns the embedded TP-Link style web administration portal in the background
pub async fn start_web_portal(
    config: Arc<RwLock<NvrConfig>>,
    drm_guard: Arc<DRMGuard<CctvGuardMetrics>>,
    metrics: Arc<CctvGuardMetrics>,
    port: u16,
) -> Result<()> {
    let admin_session_token = WebPortalState::generate_admin_token();
    let state = Arc::new(WebPortalState {
        config,
        drm_guard,
        metrics,
        auto_update_enabled: Arc::new(AtomicBool::new(true)),
        last_ota_check: Arc::new(RwLock::new(None)),
        start_time: std::time::Instant::now(),
        admin_session_token,
    });

    let app = build_router(state.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("🌐 [TP-LINK WEB PORTAL] Starting Embedded Setup Server on http://0.0.0.0:{}", port);
    println!("   Customers can open http://localhost:{} or http://<device-ip>:{} to configure cameras & 24h lease.", port, port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("[TP-LINK WEB PORTAL] Server error: {}", e);
        }
    });

    Ok(())
}
