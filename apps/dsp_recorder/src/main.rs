use anyhow::Result;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use dsp_recorder::capture::AudioCaptureStream;
use dsp_recorder::dsp::ToneCloner;
use dsp_recorder::pipeline::{PipelineConfig, PipelineMode, StudioAudioPipeline};
use dsp_recorder::storage::BitDepth;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=================================================================");
    println!("🎛️ Pure Rust Audio Recorder & Tone Cloner (DSP Engine)");
    println!("   - ⚡ Real-Time Zero-Latency Audio Capture (CPAL 128/256 Buffers)");
    println!("   - 🎻 Precision IIR Biquad Filters (Low-Shelf, Presence, Air)");
    println!("   - 🧠 Fast Fourier Spectral Tone Cloning (rustfft 2048-pt)");
    println!("   - 💾 Lossless High-Fidelity WAV Serialization (hound 16/24-bit)");
    println!("=================================================================\n");

    let args: Vec<String> = env::args().collect();

    // Check for help flag
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return Ok(());
    }

    // 1. Devices List Command
    if args.iter().any(|a| a == "--devices") {
        println!("🎤 Enumerating system audio input devices...");
        match AudioCaptureStream::list_input_devices() {
            Ok(devices) => {
                if devices.is_empty() {
                    println!("⚠️ No audio input devices detected.");
                } else {
                    for (i, dev) in devices.iter().enumerate() {
                        println!("   [{}] {}", i + 1, dev);
                    }
                }
            }
            Err(e) => eprintln!("❌ Failed to query audio devices: {}", e),
        }
        return Ok(());
    }

    // 2. Spectral Analysis Command
    if let Some(pos) = args.iter().position(|a| a == "--analyze") {
        if let Some(file_path) = args.get(pos + 1) {
            println!("🔍 Performing 2048-point spectral FFT analysis on '{}'...", file_path);
            let mut cloner = ToneCloner::new();
            match cloner.analyze_wav_file(file_path) {
                Ok(profile) => {
                    println!("📊 Spectral Acoustic Profile Summary:");
                    println!("   ├─ Sub-Bass  (20-60Hz):     {:.4}", profile.band_energies[0]);
                    println!("   ├─ Bass      (60-250Hz):    {:.4}", profile.band_energies[1]);
                    println!("   ├─ Low-Mids  (250-500Hz):   {:.4}", profile.band_energies[2]);
                    println!("   ├─ Mids      (500-2000Hz):  {:.4}", profile.band_energies[3]);
                    println!("   ├─ High-Mids (2000-4000Hz): {:.4}", profile.band_energies[4]);
                    println!("   ├─ Presence  (4000-8000Hz): {:.4}", profile.band_energies[5]);
                    println!("   ├─ Air/Sheen (8k-20kHz):    {:.4}", profile.band_energies[6]);
                    println!("   ├─ Spectral Centroid:       {:.1} Hz", profile.spectral_centroid_hz);
                    println!("   ├─ Average RMS Level:       {:.4}", profile.rms_level);
                    println!("   └─ Frames Analyzed:         {}", profile.frames_analyzed);
                }
                Err(e) => eprintln!("❌ Failed to analyze WAV file: {}", e),
            }
            return Ok(());
        } else {
            eprintln!("❌ Missing WAV file path argument for --analyze");
            return Ok(());
        }
    }

    // 3. Live Recording & DSP Tone Cloning Studio
    let mut output_path = PathBuf::from("recording_master.wav");
    let mut reference_wav: Option<PathBuf> = None;
    let mut bass_boost_db: f32 = 3.5;
    let mut presence_db: f32 = 2.5;
    let mut air_db: f32 = 2.0;
    let mut bit_depth = BitDepth::Pcm16;
    let mut sample_rate = 48000u32;
    let mut buffer_size = 256u32;
    let mut duration_secs: Option<u64> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--out" | "-o" => {
                if let Some(val) = args.get(i + 1) {
                    output_path = PathBuf::from(val);
                    i += 1;
                }
            }
            "--clone" | "-c" => {
                if let Some(val) = args.get(i + 1) {
                    reference_wav = Some(PathBuf::from(val));
                    i += 1;
                }
            }
            "--bass" => {
                if let Some(val) = args.get(i + 1) {
                    bass_boost_db = val.parse().unwrap_or(3.5);
                    i += 1;
                }
            }
            "--presence" => {
                if let Some(val) = args.get(i + 1) {
                    presence_db = val.parse().unwrap_or(2.5);
                    i += 1;
                }
            }
            "--air" => {
                if let Some(val) = args.get(i + 1) {
                    air_db = val.parse().unwrap_or(2.0);
                    i += 1;
                }
            }
            "--rate" => {
                if let Some(val) = args.get(i + 1) {
                    sample_rate = val.parse().unwrap_or(48000);
                    i += 1;
                }
            }
            "--buffer" => {
                if let Some(val) = args.get(i + 1) {
                    buffer_size = val.parse().unwrap_or(256);
                    i += 1;
                }
            }
            "--depth" => {
                if let Some(val) = args.get(i + 1) {
                    bit_depth = match val.as_str() {
                        "24" => BitDepth::Pcm24,
                        "32" => BitDepth::Float32,
                        _ => BitDepth::Pcm16,
                    };
                    i += 1;
                }
            }
            "--duration" | "-d" => {
                if let Some(val) = args.get(i + 1) {
                    duration_secs = val.parse().ok();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("⚙️ Pipeline Configuration:");
    println!("   ├─ Target Output:  {}", output_path.display());
    println!("   ├─ Sample Rate:    {} Hz", sample_rate);
    println!("   ├─ Buffer Size:    {} frames ({:.2} ms latency)", buffer_size, (buffer_size as f32 / sample_rate as f32) * 1000.0);
    println!("   ├─ Bit Depth:      {:?}", bit_depth);

    let config = PipelineConfig {
        output_wav_path: output_path.clone(),
        sample_rate,
        buffer_size,
        bit_depth,
        mode: PipelineMode::Continuous,
        vad_model_path: None,
        vad_threshold: 0.5,
        initial_bass_db: bass_boost_db,
        initial_presence_db: presence_db,
        initial_air_db: air_db,
    };

    let mut pipeline = StudioAudioPipeline::new(config)?;

    // If reference tone cloning was requested, calibrate now
    if let Some(ref ref_path) = reference_wav {
        println!("\n🧠 Calibrating tone profile from reference: '{}'...", ref_path.display());
        match pipeline.calibrate_from_reference(ref_path) {
            Ok(eq_match) => {
                println!("🎯 Tone Cloner Computed EQ Adjustments:");
                println!("   ├─ Bass Shelf:   {:+.2} dB", eq_match.bass_shelf_db);
                println!("   ├─ Low-Mid Body: {:+.2} dB", eq_match.low_mid_db);
                println!("   ├─ Presence:     {:+.2} dB", eq_match.presence_db);
                println!("   └─ Air / Sheen:  {:+.2} dB", eq_match.high_shelf_air_db);
            }
            Err(e) => {
                eprintln!("⚠️ Failed to calibrate from reference: {}. Using manual DSP sliders.", e);
            }
        }
    } else {
        println!("🎚️ Active IIR Biquad DSP Sliders:");
        println!("   ├─ Low-Shelf Bass Boost: {:+.2} dB", bass_boost_db);
        println!("   ├─ Vocal Presence:       {:+.2} dB", presence_db);
        println!("   └─ High-Shelf Air:       {:+.2} dB", air_db);
    }

    // Start Recording Pipeline
    println!("\n🔴 Starting audio stream... [Press Ctrl+C to stop and finalize]");
    pipeline.start()?;

    let start_time = std::time::Instant::now();

    // Live VU meter and statistics loop
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n\n⏹️ Received Stop Signal (Ctrl+C). Finalizing lossless WAV output...");
                break;
            }
            _ = sleep(Duration::from_millis(100)) => {
                let stats = pipeline.get_live_stats();
                let elapsed = start_time.elapsed().as_secs();

                // Draw live VU Meter in console
                let vu_bar = render_vu_bar(stats.rms_db);
                print!(
                    "\r🎙️ [Time: {:02}:{:02}] VU: [{}] {:5.1} dB | Peak: {:.2}  ",
                    elapsed / 60,
                    elapsed % 60,
                    vu_bar,
                    stats.rms_db,
                    stats.peak_level
                );
                let _ = io::stdout().flush();

                if let Some(limit) = duration_secs {
                    if elapsed >= limit {
                        println!("\n\n⏱️ Reached duration limit ({}s). Finalizing output...", limit);
                        break;
                    }
                }
            }
        }
    }

    // Stop and finalize WAV header
    let total_samples = pipeline.stop()?;
    let duration = total_samples as f32 / sample_rate as f32;
    println!("✅ Audio recording saved successfully!");
    println!("   ├─ File:     {}", output_path.display());
    println!("   ├─ Samples:  {}", total_samples);
    println!("   └─ Duration: {:.2} seconds\n", duration);

    Ok(())
}

/// Renders a dynamic visual ASCII VU meter bar for audio level monitoring.
fn render_vu_bar(db: f32) -> String {
    let clamped_db = db.clamp(-60.0, 0.0);
    let normalized = ((clamped_db + 60.0) / 60.0).clamp(0.0, 1.0);
    let total_chars = 24;
    let filled_chars = (normalized * total_chars as f32).round() as usize;

    let mut bar = String::with_capacity(total_chars);
    for i in 0..total_chars {
        if i < filled_chars {
            if i < 16 {
                bar.push('█'); // Green zone
            } else if i < 21 {
                bar.push('▒'); // Yellow zone
            } else {
                bar.push('!'); // Red / Peaking zone
            }
        } else {
            bar.push('░');
        }
    }
    bar
}

fn print_usage() {
    println!("Usage: cargo run -p dsp_recorder -- [OPTIONS]\n");
    println!("Options:");
    println!("  --devices                List all available audio input devices");
    println!("  --analyze <FILE>         Perform 2048-point spectral FFT analysis on a WAV file");
    println!("  --clone <REF_WAV>        Extract acoustic tone signature from reference WAV and apply it live");
    println!("  --out <FILE>, -o <FILE>  Set output WAV destination (default: recording_master.wav)");
    println!("  --bass <dB>              Set low-shelf bass gain in dB (default: +3.5 dB)");
    println!("  --presence <dB>          Set vocal presence gain in dB (default: +2.5 dB)");
    println!("  --air <dB>               Set high-shelf treble/air gain in dB (default: +2.0 dB)");
    println!("  --rate <Hz>              Audio sampling rate: 44100 or 48000 (default: 48000)");
    println!("  --buffer <N>             Low-latency buffer size: 128 or 256 (default: 256)");
    println!("  --depth <16|24|32>       PCM bit depth (default: 16)");
    println!("  --duration <SECS>, -d    Auto-stop after specified duration in seconds");
    println!("  -h, --help               Display this help message\n");
}
