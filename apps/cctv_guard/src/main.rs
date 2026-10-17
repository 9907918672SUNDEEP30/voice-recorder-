use anyhow::Result;
use cctv_guard::rtsp::client::SimulatorScenario;
use cctv_guard::{
    CctvFrame, CctvGuardConfig, CctvGuardEngine, EvidenceVaultConfig, GatekeeperConfig,
    RtspClient, RtspClientConfig, VlmManagerConfig,
};
use clap::Parser;
use drm_guard::{CctvGuardMetrics, DRMGuard};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(
    name = "cctv_guard",
    version = "0.1.0",
    about = "Pure Rust Two-Tier Proactive CCTV Guard with Edge Gatekeeper and Qwen VLM ONNX Inference"
)]
struct Args {
    /// Live RTSP stream URL from IP Camera or NVR
    #[arg(long, default_value = "rtsp://127.0.0.1:8554/live")]
    rtsp_url: String,

    /// Run built-in security scenario simulator: quiet | customer | shutter-robbery | armed-robbery | delivery | animal
    #[arg(long)]
    simulate: Option<String>,

    /// Unique camera identifier
    #[arg(long, default_value = "cam_bank_vault_01")]
    camera_id: String,

    /// Tier-1 anomaly trigger threshold [0.0 - 1.0]
    #[arg(long, default_value_t = 0.04)]
    motion_thresh: f32,

    /// Cooldown timer in seconds between consecutive Tier-2 VLM triggers
    #[arg(long, default_value_t = 15.0)]
    cooldown_secs: f64,

    /// Path to 2B Qwen VLM quantized 4-bit ONNX model
    #[arg(long, default_value = "C:\\AI_Models\\qwen2_vl_2b_int4.onnx")]
    vlm_model_path: String,

    /// Directory for saving snapshot JPEG evidence and structured JSON logs
    #[arg(long, default_value = "evidence")]
    evidence_dir: PathBuf,

    /// Duration in frames for simulated run (default 60 frames)
    #[arg(long, default_value_t = 60)]
    frames: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("=================================================================");
    println!(" 🛡️  PROACTIVE AI CCTV GUARD - TWO-TIER EDGE SENTRY");
    println!("=================================================================");
    println!(" Camera ID:        {}", args.camera_id);
    println!(" Anomaly Thresh:   {:.2}", args.motion_thresh);
    println!(" Cooldown Timer:   {:.1}s", args.cooldown_secs);
    println!(" VLM Model Path:   {}", args.vlm_model_path);
    println!(" Evidence Dir:     {:?}", args.evidence_dir);

    // Universal DRM Guard (24-Hour Rotating Lease & Background Jio TV OTA Updater)
    let drm_metrics = Arc::new(CctvGuardMetrics::new(1));
    let drm = Arc::new(
        DRMGuard::builder()
            .client_id("cctv_guard_edge")
            .license_key("PEHREDAR-SHAMBHAVI-24H-LEASE")
            .metrics_provider(drm_metrics.clone())
            .grace_period_hours(24)
            .build()?,
    );
    if let Err(e) = drm.perform_heartbeat().await {
        println!(" [DRM Guard] Initial handshake: {}. Active in offline lease mode.", e);
    } else {
        println!(" [DRM Guard] 24-Hour Lease Active. Machine: {}", &drm.machine_uuid()[..16]);
    }
    drm.clone().start_background_daemon();

    let config = CctvGuardConfig {
        camera_id: args.camera_id.clone(),
        gatekeeper: GatekeeperConfig {
            trigger_threshold_high: args.motion_thresh,
            reset_threshold_low: args.motion_thresh * 0.4,
            cooldown_duration_secs: args.cooldown_secs,
            ..Default::default()
        },
        vlm: VlmManagerConfig {
            model_path: args.vlm_model_path,
            unload_after_idle_secs: 60,
        },
        storage: EvidenceVaultConfig {
            base_dir: args.evidence_dir,
            save_snapshots: true,
            jpeg_quality: 85,
        },
    };

    let mut guard = CctvGuardEngine::new(config)?;
    let (tx, rx) = mpsc::channel::<CctvFrame>(32);
    let is_running = Arc::new(AtomicBool::new(true));

    // Handle Ctrl+C gracefully
    let running_flag = is_running.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            println!("\n[Shutdown] Received Ctrl+C. Halting CCTV Guard...");
            running_flag.store(false, Ordering::SeqCst);
        }
    });

    // Start RTSP or Simulator source
    let mut client = if let Some(sim_name) = &args.simulate {
        let scenario = match sim_name.as_str() {
            "quiet" => SimulatorScenario::NormalQuiet,
            "customer" => SimulatorScenario::NormalDayCustomer,
            "shutter-robbery" => SimulatorScenario::NightIntruderShutter,
            "armed-robbery" => SimulatorScenario::DaytimeArmedRobber,
            "delivery" => SimulatorScenario::PackageDelivery,
            "animal" => SimulatorScenario::AnimalCrossing,
            other => {
                eprintln!("Unknown scenario '{}', defaulting to 'shutter-robbery'", other);
                SimulatorScenario::NightIntruderShutter
            }
        };

        println!(" [Mode] Simulator Scenario: {:?}", scenario);
        RtspClient::new_simulator(
            RtspClientConfig {
                width: 1280,
                height: 720,
                target_fps: 20,
                ..Default::default()
            },
            scenario,
            args.frames,
        )
    } else {
        println!(" [Mode] Live Network RTSP Stream: {}", args.rtsp_url);
        RtspClient::new_network(RtspClientConfig {
            url: args.rtsp_url,
            width: 1280,
            height: 720,
            target_fps: 20,
            ..Default::default()
        })
    };

    let _stream_handle = client.start(tx);

    // Run main guard loop
    guard.run_loop(rx, is_running).await?;

    println!("\n=== Guard Run Completed ===");
    println!(" Total Frames Processed:   {}", guard.telemetry().total_frames_processed);
    println!(" Tier-1 Anomalies:         {}", guard.telemetry().tier1_anomalies_detected);
    println!(" Tier-2 VLM Invocations:   {}", guard.telemetry().tier2_vlm_triggers);
    println!(" Critical Threats Logged:  {}", guard.telemetry().critical_threats_logged);

    Ok(())
}
