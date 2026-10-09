mod codec;
mod pipeline;
mod telephony;
mod prompts;

use anyhow::Result;
use std::fs;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Voice AI Core Starting...");
    println!("Loading 100% Local ONNX Stack...");

    // Read config.json
    let config_data = fs::read_to_string("../../config.json").unwrap_or_else(|_| "{}".to_string());
    let config: Value = serde_json::from_str(&config_data).unwrap_or(serde_json::json!({}));
    
    let base_dir = config["models_dir"].as_str().unwrap_or("C:\\AI_Models");
    let silero_path = format!("{}\\{}", base_dir, config["models"]["silero_vad"]["path"].as_str().unwrap_or("silero_vad.onnx"));
    let whisper_path = format!("{}\\{}", base_dir, config["models"]["whisper_encoder"]["path"].as_str().unwrap_or("whisper_tiny_onnx\\encoder_model.onnx"));
    let qwen_path = format!("{}\\{}", base_dir, config["models"]["qwen_llm"]["path"].as_str().unwrap_or("qwen2-vl-2b\\llmmodel.onnx"));
    let openvoice_color_path = format!("{}\\{}", base_dir, config["models"]["openvoice_tts"]["path_tone_color"].as_str().unwrap_or("OpenVoice-ONNX-v2\\tone_color.onnx"));
    let openvoice_extract_path = format!("{}\\{}", base_dir, config["models"]["openvoice_tts"]["path_tone_extract"].as_str().unwrap_or("OpenVoice-ONNX-v2\\tone_extract.onnx"));
    let piper_path = format!("{}\\{}", base_dir, config["models"]["piper_tts"]["path"].as_str().unwrap_or("piper_hi\\hi_IN-priyamvada-medium-int8.onnx"));

    println!("\n✅ Models Found in config.json:");
    println!(" - VAD: {}", silero_path);
    println!(" - STT: {}", whisper_path);
    println!(" - LLM: {}", qwen_path);
    println!(" - TTS (Piper Base): {}", piper_path);
    println!(" - TTS (OpenVoice Tone Color): {}", openvoice_color_path);
    println!(" - TTS (OpenVoice Extract): {}", openvoice_extract_path);
    println!();
    
    ort::init()
        .with_name("voice_agent_onnx")
        .commit();

    println!("ONNX Runtime Initialized.");

    // App-specific reference audio for TTS (e.g., Sweet Girl Voice for this Voice Agent app)
    // The core framework expects this path and caches the tone embedding automatically!
    let reference_audio_path = "assets\\sweet_girl_reference.wav";
    
    // Initialize the neutral TTS adapter with Piper as Base
    let _tts_adapter = ai_core::models::onnx_adapters::OpenVoiceLocal::new(
        &piper_path,
        &openvoice_color_path,
        &openvoice_extract_path,
        reference_audio_path,
    )?;

    // The telephony server will handle incoming connections, spawn a pipeline orchestrator
    // for each connection, and hook up the VAD, STT, LLM, and TTS models.
    
    telephony::start_telephony_server().await?;

    Ok(())
}
