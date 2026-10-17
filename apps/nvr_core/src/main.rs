//! Shambhavi AI Labs — Pehredar Box (`nvr_core`)
//! Turnkey Commercial AI CCTV NVR Hardware Appliance Engine
//! Designed specifically for minimal Headless Linux / Ubuntu embedded hardware appliances.
//! Zero-Copy Storage • Direct HDMI Framebuffer • Pluggable Edge AI • 24h Cryptographic Lease.

pub mod actuator;
pub mod ai;
pub mod config;
pub mod display;
pub mod ingestion;
pub mod security;
pub mod sentinel;
pub mod storage;
pub mod web_portal;

use actuator::{AlertNotifier, EvidencePackager, HardwareRelayController};
use ai::PluggableAiEngine;
use anyhow::Result;
use clap::Parser;
use config::NvrConfig;
use display::{DisplayGridCompositor, HdmiDisplayWindow, HudRenderer};
use drm_guard::{CctvGuardMetrics, DRMGuard};
use ingestion::{SimulatedCameraFeed, SimulationScenario};
use security::{HardwareFingerprint, LeaseGuardian};
use sentinel::{GatekeeperDecision, SentinelGatekeeper};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use storage::{CameraRingBuffer, RollingSegmentWriter, StorageCleaner};
use tokio::sync::RwLock;

#[derive(Parser, Debug)]
#[command(author, version, about = "Shambhavi AI Labs Pehredar Box NVR Engine", long_about = None)]
struct CliArgs {
    /// Path to NVR configuration file (default: nvr_config.json)
    #[arg(short, long, default_value = "nvr_config.json")]
    config: PathBuf,

    /// Run simulation scenario ("shutter-break-in", "quiet-room", "delivery")
    #[arg(long)]
    simulate: Option<String>,

    /// Override number of frames to run (default: 80, 0 = infinite continuous 24/7)
    #[arg(long, default_value_t = 80)]
    frames: usize,

    /// Disable the AI intelligence layer (runs in pure recording & display mode)
    #[arg(long)]
    disable_ai: bool,

    /// Optional override for vision model path (defaults to path in nvr_config.json)
    #[arg(long)]
    model_path: Option<PathBuf>,

    /// Force headless display mode (suppresses GUI window; uses headless framebuffer)
    #[arg(long)]
    headless: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    println!("============================================================");
    println!("🛡️  SHAMBHAVI AI LABS — PEHREDAR BOX ENGINE (nvr_core)    🛡️");
    println!("============================================================");

    // 0. Anti-Tamper & Anti-Reversing Integrity Check (Dead-man trigger)
    security::AntiTamperGuardian::enforce_integrity();

    // 1. Hardware Fingerprint & Machine Identity Lock (Linux /proc/cpuinfo, /sys/class/dmi/id)
    let fingerprint = HardwareFingerprint::probe();
    println!("🔑 Machine Hardware Hash: {}", &fingerprint.machine_hash[..16]);
    println!("📟 Board UUID:           {}", fingerprint.board_uuid);
    println!("💻 CPU ID:               {}", fingerprint.cpu_id);
    println!("🌐 Primary MAC:          {}", fingerprint.primary_mac);

    // 2. 24-Hour Rotating Cloud Lease Validation (Cloudflare Worker + Supabase)
    let mut lease_guardian = LeaseGuardian::new(fingerprint.machine_hash.clone());
    lease_guardian.tick();
    if !lease_guardian.is_authorized() {
        eprintln!("❌ [SECURITY LOCKOUT] Appliance lease expired or unlicensed!");
        eprintln!("   Please scan QR code on HDMI screen or contact Shambhavi AI Labs billing.");
        return Ok(());
    }
    let entitled_channels = lease_guardian.max_allowed_cameras();
    println!("✅ 24-Hour Lease Active: Authorized for {} cameras (₹250/cam/mo tier)", entitled_channels);

    // 3. Load or Initialize Appliance Configuration
    let config = NvrConfig::load_or_default(&args.config)?;
    println!("📦 Box Identity:         {}", config.appliance.box_id);
    println!("⚙️  Hardware Tier:        {:?}", config.appliance.tier);
    println!("💾 Storage Directory:    {:?}", config.appliance.storage_dir);
    println!("📁 Evidence Directory:   {:?}", config.appliance.evidence_dir);

    // Channel Entitlement Locking: Enforce that customer cannot attach unauthorized cameras
    let mut active_channels = Vec::new();
    for (idx, cam) in config.cameras.iter().enumerate() {
        if (idx as u32) < entitled_channels {
            active_channels.push(cam.id.clone());
        } else {
            eprintln!("🔒 [CHANNEL LOCKED] Camera '{}' exceeds paid entitlement ({}/{}). Payment required to unlock.",
                cam.id, idx + 1, entitled_channels
            );
        }
    }
    println!("📹 Active Authorized Channels: {} / {}", active_channels.len(), config.cameras.len());

    // Resolve model path: CLI argument > nvr_config.json > auto-detected path
    let model_path = args.model_path.unwrap_or_else(|| config.appliance.model_path.clone());
    let model_path_str = model_path.to_string_lossy().to_string();
    println!("🧠 Vision Model Path:    {}", model_path_str);

    // 3.5. Universal DRM Guard & Metered Licensing Integration
    let metrics = Arc::new(CctvGuardMetrics::new(config.cameras.len() as u32));
    let client_id = config.appliance.client_id.clone().unwrap_or_else(|| "retail_store_patna_01".to_string());
    let license_key = config.appliance.license_key.clone().unwrap_or_else(|| "PEHREDAR-SHAMBHAVI-24H-LEASE".to_string());

    let drm = Arc::new(
        DRMGuard::builder()
            .client_id(client_id)
            .license_key(license_key)
            .heartbeat_url(Some(config.appliance.cloud_lease_endpoint.clone()))
            .metrics_provider(metrics.clone())
            .grace_period_hours(24)
            .build()?,
    );

    // Trigger initial lease validation
    if let Err(e) = drm.perform_heartbeat().await {
        println!(" [DRM NOTICE] Initial Cloudflare handshake: {}. Active under offline lease.", e);
    } else {
        println!(" [DRM SUCCESS] 24-Hour Cryptographic Lease Verified & Active.");
    }

    // Launch background 10-minute proactive health monitoring daemon
    drm.clone().start_background_daemon();

    // 3.6. Launch TP-Link Style Embedded Web Admin & Setup Portal
    let web_port = config.appliance.web_port;
    let config_lock = Arc::new(RwLock::new(config.clone()));
    web_portal::start_web_portal(config_lock, drm.clone(), metrics.clone(), web_port).await?;

    println!("============================================================\n");

    // 4. Initialize Core Subsystems
    let scenario = match args.simulate.as_deref() {
        Some("shutter-break-in") | Some("break-in") => SimulationScenario::ShutterBreakIn,
        Some("delivery") => SimulationScenario::PackageDelivery,
        Some("quiet") | Some("quiet-room") => SimulationScenario::QuietRoom,
        _ => SimulationScenario::ShutterBreakIn,
    };

    println!("🎬 Initializing Video Feed: {:?}", scenario);

    let camera_id = "cam_bank_vault_01";
    let mut feed = SimulatedCameraFeed::new(camera_id, scenario);
    let mut gatekeeper = SentinelGatekeeper::new(camera_id, 0.04, 15.0);
    let mut relay_controller = HardwareRelayController::new(60);
    let notifier = AlertNotifier::new(config.appliance.alert_webhook_url.clone());
    let mut ring_buffer = CameraRingBuffer::new(camera_id, Duration::from_secs(45));
    let mut segment_writer = RollingSegmentWriter::new(
        camera_id,
        config.appliance.storage_dir.clone(),
        Duration::from_secs(config.appliance.chunk_duration_secs),
    );

    // 5. Initialize Direct HDMI Display Engine
    let display_compositor = DisplayGridCompositor::new(1920, 1080);
    let mut hdmi_window = HdmiDisplayWindow::new(
        "Shambhavi AI Labs — Pehredar Box (Direct HDMI)",
        1920,
        1080,
        args.headless,
    );

    // 6. Initialize Pluggable AI Engine (Decoupled & Hot-Pluggable)
    let ai_enabled = !args.disable_ai && config.appliance.enable_ai;
    let ai_plugin = PluggableAiEngine::new(ai_enabled, &model_path_str);

    let mut total_alerts = 0usize;
    let target_frames = args.frames;
    let mut frame_idx = 0usize;

    println!("🚀 Starting Real-Time Ingestion, Zero-Copy Storage & HDMI Display ({} frames)...\n",
        if target_frames == 0 { "Infinite Continuous".to_string() } else { target_frames.to_string() }
    );

    loop {
        frame_idx += 1;
        if target_frames > 0 && frame_idx > target_frames {
            break;
        }

        let frame = feed.next_frame();

        // [CORE CCTV 1] Zero-Copy Rolling Storage (Runs 24/7 Regardless of AI)
        let mock_packet_bytes = vec![0u8; 128];
        let is_keyframe = (frame_idx % 25) == 1;
        segment_writer.write_packet(&mock_packet_bytes, is_keyframe)?;

        // [CORE CCTV 2] Ring Buffer Retention
        ring_buffer.push(storage::BufferedPacket {
            is_keyframe,
            timestamp_ms: frame.timestamp_ms,
            received_at: std::time::Instant::now(),
            payload: bytes::Bytes::copy_from_slice(&mock_packet_bytes),
        });

        // [CORE CCTV 3] Direct HDMI Display Compositing
        let viewports = display_compositor.compute_layout(&active_channels, None);
        hdmi_window.draw_camera_tile(&viewports[0], &frame.image);
        let _ = hdmi_window.update();

        if frame_idx % 20 == 0 {
            let ai_tag = if ai_plugin.is_active() { "AI:ACTIVE" } else { "AI:OFF" };
            let osd = format!("{} | {}", HudRenderer::format_tile_header(&viewports[0], 25, true), ai_tag);
            println!("🟢 Frame {:03} | [CCTV NORMAL] 15-min MP4 Recording | {}", frame_idx, osd);
        }

        // [OPTIONAL AI LAYER] Tier-1 Gatekeeper & Pluggable Vision Model
        if ai_plugin.is_active() {
            let decision = gatekeeper.process_frame(frame.image.as_raw(), frame.width, frame.height);

            if let GatekeeperDecision::CandidateAnomaly { dssim_score, changed_pixels_ratio } = decision {
                println!("\n⚡ [TIER-1 TRIGGER] Frame {:03} | DSSIM: {:.4} | Changed: {:.2}%",
                    frame_idx, dssim_score, changed_pixels_ratio * 100.0
                );

                // Submit to AI worker thread (NON-BLOCKING: Drops gracefully if AI is busy/crashed)
                ai_plugin.submit_candidate_frame(frame.clone(), dssim_score);
            }

            // Continuously poll for any available threat verdicts from async AI worker
            if let Some(verdict) = ai_plugin.poll_verdict() {
                println!("🧠 [TIER-2 YOLOv8m] Level: {} | Threat: {} | Conf: {:.2} | Latency: {}ms",
                    verdict.threat_level, verdict.threat_class, verdict.confidence, verdict.inference_latency_ms
                );
                println!("   Summary: {}", verdict.summary);

                if verdict.threat_level == "Critical" || verdict.threat_level == "Suspicious" {
                    total_alerts += 1;

                    // Actuator 1: Evidence Locking
                    let incident = EvidencePackager::save_incident(
                        &config.appliance.evidence_dir,
                        camera_id,
                        &verdict.threat_level,
                        &verdict.threat_class,
                        verdict.confidence,
                        0.04,
                        &frame.image,
                    )?;

                    // Actuator 2: 110dB Siren & Strobe
                    relay_controller.trigger_alarm(camera_id, &verdict.threat_class);

                    // Actuator 3: Async Push Notification / WhatsApp
                    notifier.dispatch_alert(&incident).await?;

                    // Actuator 4: On-Screen Flashing Alert HUD in HDMI Display
                    let alert_viewports = display_compositor.compute_layout(&active_channels, Some(camera_id));
                    hdmi_window.draw_camera_tile(&alert_viewports[0], &frame.image);
                    let _ = hdmi_window.update();

                    let osd = HudRenderer::format_tile_header(&alert_viewports[0], 25, true);
                    println!("📺 [HDMI DISPLAY] Viewport updated with Emergency Red Border | OSD: {}\n", osd);
                }
            }
        }

        // Periodic maintenance: relay auto-silence
        relay_controller.tick();
    }

    // Flush active recording segment
    segment_writer.finish();

    // Check disk quota cleaner
    let cleanup = StorageCleaner::enforce_quota(&config.appliance.storage_dir, config.appliance.disk_quota_percentage)?;

    println!("\n============================================================");
    println!("🏁 EXECUTION COMPLETE — PIPELINE BENCHMARK SUMMARY");
    println!("============================================================");
    println!("📊 Total Frames Recorded:    {}", frame_idx);
    println!("🚨 Total Critical Incidents: {}", total_alerts);
    println!("💾 Ring Buffer Packets Kept: {}", ring_buffer.len());
    println!("🧹 FIFO Storage Cleaner:     {} files pruned, {} bytes freed", cleanup.files_deleted, cleanup.bytes_freed);
    println!("🛡️  Pehredar Box Engine status: Operational & Ready for deployment");
    println!("============================================================");

    Ok(())
}
