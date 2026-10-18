use cctv_guard::rtsp::client::SimulatorScenario;
use cctv_guard::rtsp::frame::{CctvFrame, DOWNSCALE_HEIGHT, DOWNSCALE_WIDTH};
use cctv_guard::storage::evidence::{EvidenceVault, EvidenceVaultConfig};
use cctv_guard::tier1::frame_diff::FrameDifferencer;
use cctv_guard::tier1::gatekeeper::{GatekeeperConfig, GatekeeperDecision, Tier1Gatekeeper};
use cctv_guard::tier1::ssim::StructuralDissimilarity;
use cctv_guard::tier2::qwen_vlm::{ThreatCategory, ThreatLevel};
use cctv_guard::tier2::vlm_manager::{VlmLifecycleManager, VlmManagerConfig};
use cctv_guard::{CctvGuardConfig, CctvGuardEngine};
use std::fs::read_to_string;
use std::time::SystemTime;

#[test]
fn test_rtsp_frame_downscaling_and_tensor_conversion() {
    let w = 640;
    let h = 480;
    let frame = CctvFrame::synthetic_pattern(w, h, 120, 80, 200, 1000, 1);

    // 1. Verify dimensions and downscaling
    assert_eq!(frame.width, w);
    assert_eq!(frame.height, h);

    let downscaled = frame.downscale_to_160x120();
    assert_eq!(downscaled.width, DOWNSCALE_WIDTH);
    assert_eq!(downscaled.height, DOWNSCALE_HEIGHT);
    assert_eq!(downscaled.gray_data.len(), 160 * 120);

    // 2. Verify Rec.601 luma conversion: (299*120 + 587*80 + 114*200) / 1000 = (35880 + 46960 + 22800) / 1000 = 105
    let expected_luma = ((299 * 120 + 587 * 80 + 114 * 200 + 500) / 1000) as u8;
    assert_eq!(downscaled.gray_data[0], expected_luma);

    // 3. Verify JPEG compression
    let jpeg = frame.to_jpeg(80).expect("JPEG encoding failed");
    assert!(jpeg.len() > 100);
    // JPEG magic bytes: 0xFF, 0xD8
    assert_eq!(jpeg[0], 0xFF);
    assert_eq!(jpeg[1], 0xD8);

    // 4. Verify 4D float tensor generation [1, 3, 384, 384]
    let tensor = frame.to_onnx_tensor(384, 384).expect("Tensor conversion failed");
    assert_eq!(tensor.shape(), &[1, 3, 384, 384]);
}

#[test]
fn test_tier1_frame_differencer_and_ssim() {
    let mut diff = FrameDifferencer::new(0.05, 20);
    let mut ssim = StructuralDissimilarity::new(100);

    let f1 = CctvFrame::synthetic_pattern(320, 240, 50, 50, 50, 0, 0).downscale_to_160x120();
    let f2 = CctvFrame::synthetic_pattern(320, 240, 50, 50, 50, 33, 1).downscale_to_160x120();

    // Baseline processing
    let m1 = diff.process(&f1);
    assert_eq!(m1.motion_ratio, 0.0);

    // Stationary scene yields negligible motion & zero dissimilarity
    let m2 = diff.process(&f2);
    let ssim2 = ssim.compute(&f2);
    assert_eq!(m2.motion_ratio, 0.0);
    assert!(ssim2 < 0.01);

    // Dynamic scene modification (simulating an intruder entering the frame)
    let mut f3_raw = vec![50u8; 320 * 240 * 3];
    // Draw bright foreign object in center
    for y in 80..160 {
        for x in 100..200 {
            let idx = (y * 320 + x) * 3;
            f3_raw[idx] = 220;
            f3_raw[idx + 1] = 220;
            f3_raw[idx + 2] = 220;
        }
    }
    let f3 = CctvFrame::new(320, 240, f3_raw, 66, 2)
        .unwrap()
        .downscale_to_160x120();

    let m3 = diff.process(&f3);
    let ssim3 = ssim.compute(&f3);

    assert!(m3.motion_ratio > 0.05, "Motion ratio was {}", m3.motion_ratio);
    assert!(ssim3 > 0.02, "Structural dissimilarity was {}", ssim3);
}

#[test]
fn test_tier1_hysteresis_and_cooldown() {
    let config = GatekeeperConfig {
        trigger_threshold_high: 0.15,
        reset_threshold_low: 0.05,
        cooldown_duration_secs: 2.0,
        motion_weight: 0.5,
        ssim_weight: 0.5,
        pre_buffer_capacity: 5,
    };

    let mut gatekeeper = Tier1Gatekeeper::new(config);

    // Frame 0: Baseline static frame
    let f0 = CctvFrame::synthetic_pattern(320, 240, 30, 30, 30, 0, 0);
    let d0 = gatekeeper.evaluate_frame(f0);
    assert!(matches!(d0, GatekeeperDecision::Normal { .. }));

    // Frame 1: Huge change causing immediate trigger
    let mut f1_raw = vec![30u8; 320 * 240 * 3];
    for i in 0..(f1_raw.len() / 2) {
        f1_raw[i] = 240;
    }
    let f1 = CctvFrame::new(320, 240, f1_raw.clone(), 33, 1).unwrap();
    let d1 = gatekeeper.evaluate_frame(f1);

    assert!(
        matches!(d1, GatekeeperDecision::TriggerVlm { .. }),
        "Expected TriggerVlm, got {:?}",
        d1
    );

    // Frame 2: Immediate next frame during cooldown must be suppressed
    let f2 = CctvFrame::new(320, 240, f1_raw, 66, 2).unwrap();
    let d2 = gatekeeper.evaluate_frame(f2);
    assert!(
        matches!(d2, GatekeeperDecision::SuppressedCooldown { .. }),
        "Expected SuppressedCooldown, got {:?}",
        d2
    );
}

#[test]
fn test_tier2_vlm_manager_fail_closed_on_missing_model() {
    let mut manager = VlmLifecycleManager::new(VlmManagerConfig {
        model_path: "C:\\AI_Models\\non_existent_test_model.onnx".to_string(),
        unload_after_idle_secs: 10,
    });

    // Zero-Mock Doctrine: When model is missing, engine MUST fail closed rather than faking threat assessments!
    let frame = CctvFrame::synthetic_pattern(640, 480, 40, 40, 45, 100, 1);
    let res = manager.evaluate_event(&frame);
    assert!(res.is_err(), "VLM must strictly reject inference when model file does not exist");

    // Housekeeping runs cleanly without crashing
    manager.tick_housekeeping();
    assert_eq!(manager.total_triggers(), 0);
}

#[test]
fn test_evidence_vault_logging_and_snapshots() {
    let unique_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let temp_path = std::env::temp_dir().join(format!("cctv_test_vault_{}", unique_id));
    let config = EvidenceVaultConfig {
        base_dir: temp_path,
        save_snapshots: true,
        jpeg_quality: 80,
    };

    let vault = EvidenceVault::new(config).expect("Failed to initialize EvidenceVault");
    let frame = CctvFrame::synthetic_pattern(320, 240, 20, 20, 25, 1728641100000, 10);
    let assessment = cctv_guard::tier2::ThreatAssessment {
        threat_level: ThreatLevel::Critical,
        category: ThreatCategory::Intruder,
        confidence: 0.96,
        description: "Intruder attempting forced entry with crowbar".to_string(),
        inference_time_ms: 15,
    };

    let record = vault
        .record_incident(&frame, &assessment, 0.45, "cam_bank_main")
        .expect("Failed to record incident");

    assert_eq!(record.camera_id, "cam_bank_main");
    assert_eq!(record.threat_level, ThreatLevel::Critical);
    assert!(record.snapshot_path.is_some());

    // Verify file exists on disk
    let snapshot_file = record.snapshot_path.unwrap();
    assert!(std::path::Path::new(&snapshot_file).exists());

    // Verify JSONL log file
    let log_content = read_to_string(vault.log_path()).expect("Failed to read log");
    assert!(log_content.contains("cam_bank_main"));
    assert!(log_content.contains("Intruder"));
    assert!(log_content.contains("0.96"));
}

#[test]
fn test_end_to_end_bank_shutter_robbery_simulation() {
    let unique_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let temp_path = std::env::temp_dir().join(format!("cctv_test_e2e_{}", unique_id));
    let guard_config = CctvGuardConfig {
        camera_id: "cam_bank_night_shutter".to_string(),
        gatekeeper: GatekeeperConfig {
            trigger_threshold_high: 0.04,
            reset_threshold_low: 0.015,
            cooldown_duration_secs: 5.0,
            ..Default::default()
        },
        storage: EvidenceVaultConfig {
            base_dir: temp_path,
            save_snapshots: true,
            jpeg_quality: 80,
        },
        ..Default::default()
    };

    let mut guard = CctvGuardEngine::new(guard_config).expect("Failed to create engine");

    // Phase 1: Normal quiet nighttime shutter frames (frames 0 to 9)
    let rtsp_cfg = cctv_guard::RtspClientConfig {
        width: 640,
        height: 360,
        target_fps: 20,
        ..Default::default()
    };

    let mut triggered_record = None;

    // Simulate night robbery scenario:
    // Frames 0-9: Quiet nighttime shutter
    // Frames 10+: Masked intruder with crowbar tampering with shutter
    for f_idx in 0..25 {
        let frame = cctv_guard::RtspClient::render_scenario_frame(
            &rtsp_cfg,
            SimulatorScenario::NightIntruderShutter,
            f_idx,
            (f_idx * 50) as u64,
        );
        match guard.process_single_frame(frame) {
            Ok(Some(rec)) => {
                println!("Frame {}: Triggered with score {}", f_idx, rec.anomaly_score);
                triggered_record = Some(rec);
                break;
            }
            Ok(None) => {
                // not triggered
            }
            Err(e) => {
                println!("Error on frame {}: {}", f_idx, e);
            }
        }
    }

    assert!(
        triggered_record.is_some(),
        "Expected Tier-1 trigger and Tier-2 VLM logging during night robbery"
    );

    let rec = triggered_record.unwrap();
    assert_eq!(rec.threat_level, ThreatLevel::Critical);
    assert_eq!(rec.category, ThreatCategory::Intruder);
    assert!(rec.description.contains("roller shutter"));
    assert_eq!(guard.telemetry().critical_threats_logged, 1);
}
