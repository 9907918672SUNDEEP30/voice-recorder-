use anyhow::Result;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use video_studio::capture::{CameraSource, NokhwaCamera, VirtualCamera};
use video_studio::grading::LutPreset;
use video_studio::pipeline::{StudioPipelineConfig, StudioVideoPipeline};
use video_studio::storage::FrameWriter;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=================================================================");
    println!("🎥 AI Video Studio: Virtual Cameraman & Gaze Correction Engine");
    println!("   - 🎯 Auto-Framing & Smart Face Tracking (EMA Motion Damped PTZ)");
    println!("   - 👁️ AI Eye Contact Correction (Teleprompter Iris Warping)");
    println!("   - ✨ Edge-Preserving Skin Filter & Studio Relighting");
    println!("   - 🎨 Real-Time 3D LUT Trilinear Color Grading");
    println!("=================================================================\n");

    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return Ok(());
    }

    // 1. Devices List Command
    if args.iter().any(|a| a == "--devices") {
        println!("📷 Scanning for system video camera devices...");
        match NokhwaCamera::query_devices() {
            Ok(devices) => {
                if devices.is_empty() {
                    println!("⚠️ No physical video camera devices detected.");
                } else {
                    for dev in &devices {
                        println!("   {}", dev);
                    }
                }
            }
            Err(e) => eprintln!("❌ Failed to query camera devices: {}", e),
        }
        return Ok(());
    }

    // 2. Parse Options
    let mut use_virtual_camera = true;
    let mut camera_index = 0u32;
    let mut lut_preset = LutPreset::TealAndOrange;
    let mut auto_framing = true;
    let mut gaze_correction = true;
    let mut gaze_strength = 0.85f32;
    let mut snapshot_path: Option<PathBuf> = None;
    let mut record_frames_count: Option<u64> = None;
    let mut record_dir = PathBuf::from("video_records");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--camera" | "-c" => {
                if let Some(val) = args.get(i + 1) {
                    camera_index = val.parse().unwrap_or(0);
                    use_virtual_camera = false;
                    i += 1;
                }
            }
            "--virtual" | "-v" => {
                use_virtual_camera = true;
            }
            "--lut" => {
                if let Some(val) = args.get(i + 1) {
                    lut_preset = match val.to_lowercase().as_str() {
                        "teal-orange" | "teal" => LutPreset::TealAndOrange,
                        "warm" => LutPreset::WarmStudio,
                        "minimal" => LutPreset::CleanMinimal,
                        "film" => LutPreset::FilmPunch,
                        _ => LutPreset::None,
                    };
                    i += 1;
                }
            }
            "--no-framing" => {
                auto_framing = false;
            }
            "--no-gaze" => {
                gaze_correction = false;
            }
            "--gaze" => {
                if let Some(val) = args.get(i + 1) {
                    gaze_strength = val.parse().unwrap_or(0.85);
                    i += 1;
                }
            }
            "--snapshot" => {
                if let Some(val) = args.get(i + 1) {
                    snapshot_path = Some(PathBuf::from(val));
                    i += 1;
                }
            }
            "--record-frames" => {
                if let Some(val) = args.get(i + 1) {
                    record_frames_count = val.parse().ok();
                    i += 1;
                }
            }
            "--out" | "-o" => {
                if let Some(val) = args.get(i + 1) {
                    record_dir = PathBuf::from(val);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // 3. Initialize Camera Source
    let camera: Box<dyn CameraSource> = if use_virtual_camera {
        println!("💡 Mode: Virtual Camera Simulator (Deterministic 1080p Studio Subject)");
        Box::new(VirtualCamera::new(1280, 720))
    } else {
        println!("📷 Mode: Physical Hardware Camera [Index: {}]", camera_index);
        match NokhwaCamera::new(camera_index, 1280, 720) {
            Ok(cam) => Box::new(cam),
            Err(e) => {
                eprintln!("⚠️ Failed to open hardware camera {}: {}. Falling back to Virtual Camera.", camera_index, e);
                Box::new(VirtualCamera::new(1280, 720))
            }
        }
    };

    let config = StudioPipelineConfig {
        output_width: 1280,
        output_height: 720,
        auto_framing_enabled: auto_framing,
        gaze_correction_enabled: gaze_correction,
        gaze_strength,
        skin_smoothing_enabled: true,
        skin_strength: 0.60,
        relighting_enabled: true,
        lut_preset,
    };

    println!("⚙️ Studio Configuration:");
    println!("   ├─ Canvas Output:     1280x720 (720p 60fps capable)");
    println!("   ├─ Auto-Framing PTZ:  {}", if auto_framing { "ENABLED (EMA Motion Damped)" } else { "DISABLED" });
    println!("   ├─ Gaze Correction:   {} (Strength: {:.2})", if gaze_correction { "ENABLED" } else { "DISABLED" }, gaze_strength);
    println!("   ├─ Skin Smoothing:    ENABLED (Edge-Preserving Guided Filter)");
    println!("   └─ 3D LUT Preset:     {:?}", lut_preset);

    let mut pipeline = StudioVideoPipeline::new(camera, config);
    pipeline.start()?;

    let frame_writer = FrameWriter::new(&record_dir)?;

    println!("\n🔴 Live Studio Pipeline Active! [Press Ctrl+C to stop]\n");

    let mut frames_processed = 0u64;
    let max_frames = record_frames_count.unwrap_or(u64::MAX);

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n\n⏹️ Received Stop Signal (Ctrl+C). Finalizing Studio session...");
                break;
            }
            _ = sleep(Duration::from_millis(16)) => { // ~60 FPS loop cadence
                match pipeline.process_next_frame() {
                    Ok(frame) => {
                        frames_processed += 1;
                        let t = pipeline.telemetry();

                        // Print responsive console dashboard
                        print!(
                            "\r🎬 [Frame: {:06}] {:4.1} FPS | Latency: {:4.1} ms | Face: {} | Gaze: {} | Cameraman: [{}] ",
                            t.frame_index,
                            t.fps,
                            t.latency_ms,
                            if t.face_detected { "TRACKED" } else { "SEARCHING" },
                            if t.eye_contact_active { "CORRECTED" } else { "BYPASS" },
                            if auto_framing { "AUTO-PTZ" } else { "STATIC" }
                        );
                        let _ = io::stdout().flush();

                        // Save snapshot if requested and not yet saved
                        if let Some(ref path) = snapshot_path {
                            if frames_processed == 5 {
                                println!("\n📸 Saving styled snapshot to: {}", path.display());
                                if let Err(e) = FrameWriter::save_snapshot_png(&frame, path) {
                                    eprintln!("❌ Failed to save snapshot: {}", e);
                                }
                            }
                        }

                        // Record frame if sequence requested
                        if record_frames_count.is_some() {
                            let _ = frame_writer.save_frame_ppm(&frame, frames_processed);
                        }

                        if frames_processed >= max_frames {
                            println!("\n\n⏱️ Recorded specified frame count ({}). Done.", max_frames);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("\n⚠️ Frame processing error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    pipeline.stop()?;
    println!("\n✅ Studio session closed. Total frames processed: {}\n", frames_processed);
    Ok(())
}

fn print_usage() {
    println!("Usage: cargo run -p video_studio -- [OPTIONS]\n");
    println!("Options:");
    println!("  --devices                List all available video camera devices");
    println!("  --camera <INDEX>, -c     Select physical camera hardware index");
    println!("  --virtual, -v            Use virtual camera simulator (default)");
    println!("  --lut <PRESET>           Set 3D LUT preset: teal-orange, warm, minimal, film, none");
    println!("  --gaze <0.0-1.0>         Set teleprompter eye contact correction strength (default: 0.85)");
    println!("  --no-framing             Disable AI Cameraman auto-framing");
    println!("  --no-gaze                Disable teleprompter gaze correction");
    println!("  --snapshot <FILE.png>    Save a processed snapshot image to disk");
    println!("  --record-frames <N>      Record N processed frames as lossless PPM sequence");
    println!("  --out <DIR>, -o <DIR>    Set output directory for recordings");
    println!("  -h, --help               Show this help message\n");
}
