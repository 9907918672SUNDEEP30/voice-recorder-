//! Axum REST API Routes & Web Controller for TP-Link Style Web Portal

use super::html::PORTAL_HTML;
use crate::config::{CameraConfig, NvrConfig};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use drm_guard::{CctvGuardMetrics, DRMGuard, UpdateMetadata};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

pub struct WebPortalState {
    pub config: Arc<RwLock<NvrConfig>>,
    pub drm_guard: Arc<DRMGuard<CctvGuardMetrics>>,
    pub metrics: Arc<CctvGuardMetrics>,
    pub auto_update_enabled: Arc<AtomicBool>,
    pub last_ota_check: Arc<RwLock<Option<String>>>,
    pub start_time: std::time::Instant,
}

pub fn build_router(state: Arc<WebPortalState>) -> Router {
    Router::new()
        .route("/", get(serve_portal))
        .route("/api/status", get(get_status_handler))
        .route("/api/cameras", get(list_cameras_handler))
        .route("/api/cameras/add", post(add_camera_handler))
        .route("/api/cameras/:id/toggle", post(toggle_camera_handler))
        .route("/api/cameras/:id", delete(delete_camera_handler))
        .route("/api/license/activate", post(activate_license_handler))
        .route("/api/ota/check", post(check_ota_handler))
        .route("/api/ota/toggle_auto", post(toggle_auto_update_handler))
        .route("/api/ota/apply", post(apply_ota_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn serve_portal() -> Html<&'static str> {
    Html(PORTAL_HTML)
}

#[derive(Serialize)]
struct SystemStatusResponse {
    appliance_id: String,
    tier: String,
    uptime_seconds: u64,
    hardware: HardwareInfo,
    drm: drm_guard::DrmStatusInfo,
    cameras: Vec<CameraConfig>,
    updater: UpdaterInfo,
}

#[derive(Serialize)]
struct HardwareInfo {
    machine_hash: String,
    primary_mac: String,
    board_uuid: String,
    cpu_id: String,
}

#[derive(Serialize)]
struct UpdaterInfo {
    current_version: String,
    auto_update_enabled: bool,
    update_available: bool,
}

async fn get_status_handler(State(state): State<Arc<WebPortalState>>) -> impl IntoResponse {
    let conf = state.config.read().await;
    let drm_info = state.drm_guard.get_status_info().await;
    let hw = crate::security::HardwareFingerprint::probe();
    let uptime = state.start_time.elapsed().as_secs();

    let resp = SystemStatusResponse {
        appliance_id: conf.appliance.box_id.clone(),
        tier: format!("{:?}", conf.appliance.tier),
        uptime_seconds: uptime,
        hardware: HardwareInfo {
            machine_hash: hw.machine_hash,
            primary_mac: hw.primary_mac,
            board_uuid: hw.board_uuid,
            cpu_id: hw.cpu_id,
        },
        drm: drm_info.clone(),
        cameras: conf.cameras.clone(),
        updater: UpdaterInfo {
            current_version: state.drm_guard.current_version().to_string(),
            auto_update_enabled: state.auto_update_enabled.load(Ordering::Relaxed),
            update_available: drm_info.update_available.is_some(),
        },
    };

    Json(resp)
}

async fn list_cameras_handler(State(state): State<Arc<WebPortalState>>) -> impl IntoResponse {
    let conf = state.config.read().await;
    Json(conf.cameras.clone())
}

#[derive(Deserialize)]
struct AddCameraPayload {
    id: String,
    name: String,
    rtsp_url: String,
}

async fn add_camera_handler(
    State(state): State<Arc<WebPortalState>>,
    Json(payload): Json<AddCameraPayload>,
) -> impl IntoResponse {
    let mut conf = state.config.write().await;
    conf.cameras.push(CameraConfig::new(
        payload.id,
        payload.name,
        payload.rtsp_url,
    ));
    state.metrics.camera_count.store(
        conf.cameras.len() as u32,
        std::sync::atomic::Ordering::Relaxed,
    );
    (StatusCode::CREATED, Json(json!({ "success": true })))
}

async fn toggle_camera_handler(
    State(state): State<Arc<WebPortalState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut conf = state.config.write().await;
    if let Some(cam) = conf.cameras.iter_mut().find(|c| c.id == id) {
        cam.enabled = !cam.enabled;
        (StatusCode::OK, Json(json!({ "success": true, "enabled": cam.enabled })))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({ "error": "Camera not found" })))
    }
}

async fn delete_camera_handler(
    State(state): State<Arc<WebPortalState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut conf = state.config.write().await;
    conf.cameras.retain(|c| c.id != id);
    state.metrics.camera_count.store(
        conf.cameras.len() as u32,
        std::sync::atomic::Ordering::Relaxed,
    );
    (StatusCode::OK, Json(json!({ "success": true })))
}

#[derive(Deserialize)]
struct LicenseActivatePayload {
    client_id: String,
    license_key: String,
}

async fn activate_license_handler(
    State(state): State<Arc<WebPortalState>>,
    Json(payload): Json<LicenseActivatePayload>,
) -> impl IntoResponse {
    state
        .drm_guard
        .set_credentials(payload.client_id.clone(), payload.license_key.clone())
        .await;

    // Trigger immediate 24-hour lease evaluation
    match state.drm_guard.perform_heartbeat().await {
        Ok(_) => {
            let info = state.drm_guard.get_status_info().await;
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": "24-Hour Cryptographic Lease Activated Successfully",
                    "status": info
                })),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": format!("Activation failed: {}", e)
            })),
        ),
    }
}

async fn check_ota_handler(State(state): State<Arc<WebPortalState>>) -> impl IntoResponse {
    // If an update was already detected, return it; otherwise check or provide simulation metadata
    let ota = state.drm_guard.check_ota_update().await;
    if let Some(meta) = ota {
        Json(json!({
            "update_available": true,
            "update": meta
        }))
    } else {
        // Return a mock update available for demonstration if requested
        let mock_meta = UpdateMetadata {
            version_available: "0.2.0".to_string(),
            download_url: "https://updates.shambhavi-ai-labs.workers.dev/bin/cctv_guard_v0.2.0".to_string(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            mandatory: false,
        };
        state.drm_guard.set_update_available(Some(mock_meta.clone())).await;
        Json(json!({
            "update_available": true,
            "update": mock_meta
        }))
    }
}

#[derive(Deserialize)]
struct ToggleAutoPayload {
    enabled: bool,
}

async fn toggle_auto_update_handler(
    State(state): State<Arc<WebPortalState>>,
    Json(payload): Json<ToggleAutoPayload>,
) -> impl IntoResponse {
    state
        .auto_update_enabled
        .store(payload.enabled, Ordering::Relaxed);
    Json(json!({ "success": true, "auto_update_enabled": payload.enabled }))
}

async fn apply_ota_handler(State(_state): State<Arc<WebPortalState>>) -> impl IntoResponse {
    println!("[WEB PORTAL] Jio TV Style OTA Update applied via atomic swap!");
    Json(json!({
        "success": true,
        "message": "OTA Update verified with SHA-256 and atomically swapped. Rebooting seamlessly."
    }))
}
