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
use drm_guard::{CctvGuardMetrics, DRMGuard};
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
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::AllowOrigin::predicate(|origin, _| {
                    let s = origin.to_str().unwrap_or("");
                    s.starts_with("http://localhost") || s.starts_with("http://127.0.0.1")
                }))
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::DELETE,
                ])
                .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]),
        )
        .with_state(state)
}

fn redact_rtsp_url(url: &str) -> String {
    if let Some(at_idx) = url.find('@') {
        if let Some(proto_idx) = url.find("://") {
            let prefix = &url[..proto_idx + 3];
            let user_part = &url[proto_idx + 3..at_idx];
            let host_part = &url[at_idx..];
            if let Some(colon_idx) = user_part.find(':') {
                let user = &user_part[..colon_idx];
                return format!("{}{}:*****{}", prefix, user, host_part);
            }
        }
    }
    url.to_string()
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

    let sanitized_cameras: Vec<CameraConfig> = conf
        .cameras
        .iter()
        .map(|c| {
            let mut sanitized = c.clone();
            sanitized.rtsp_url = redact_rtsp_url(&sanitized.rtsp_url);
            sanitized
        })
        .collect();

    let resp = SystemStatusResponse {
        appliance_id: conf.appliance.box_id.clone(),
        tier: format!("{:?}", conf.appliance.tier),
        uptime_seconds: uptime,
        hardware: HardwareInfo {
            machine_hash: hw.machine_hash,
            primary_mac: "[PROTECTED]".to_string(),
            board_uuid: "[PROTECTED]".to_string(),
            cpu_id: "[PROTECTED]".to_string(),
        },
        drm: drm_info.clone(),
        cameras: sanitized_cameras,
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
    let sanitized_cameras: Vec<CameraConfig> = conf
        .cameras
        .iter()
        .map(|c| {
            let mut sanitized = c.clone();
            sanitized.rtsp_url = redact_rtsp_url(&sanitized.rtsp_url);
            sanitized
        })
        .collect();
    Json(sanitized_cameras)
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
    let ota = state.drm_guard.check_ota_update().await;
    if let Some(meta) = ota {
        Json(json!({
            "update_available": true,
            "update": meta
        }))
    } else {
        Json(json!({
            "update_available": false,
            "current_version": state.drm_guard.current_version(),
            "message": "System is running latest version."
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

async fn apply_ota_handler(State(state): State<Arc<WebPortalState>>) -> impl IntoResponse {
    let ota = state.drm_guard.check_ota_update().await;
    if let Some(meta) = ota {
        println!("[WEB PORTAL] Applying verified OTA update v{}...", meta.version_available);
        match state.drm_guard.updater().download_and_verify(&meta).await {
            Ok(bytes) => {
                match state.drm_guard.updater().apply_binary_swap(&bytes, None) {
                    Ok(_) => (
                        StatusCode::OK,
                        Json(json!({
                            "success": true,
                            "message": format!("OTA Update v{} verified and swapped atomically.", meta.version_available)
                        })),
                    ),
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "success": false, "message": format!("Binary swap failed: {}", e) })),
                    ),
                }
            }
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "success": false, "message": format!("Download/Verification failed: {}", e) })),
            ),
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "message": "No pending OTA update available to apply." })),
        )
    }
}
