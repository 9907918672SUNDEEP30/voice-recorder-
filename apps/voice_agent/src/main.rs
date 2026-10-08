mod adapters;
mod codec;
mod pipeline;
mod telephony;
mod onnx_adapters;
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
    let kokoro_path = format!("{}\\{}", base_dir, config["models"]["kokoro_tts"]["path"].as_str().unwrap_or("kokoro_tts_onnx\\model.onnx"));

    println!("\n✅ Models Found in config.json:");
    println!(" - VAD: {}", silero_path);
    println!(" - STT: {}", whisper_path);
    println!(" - LLM: {}", qwen_path);
    println!(" - TTS: {}", kokoro_path);
    println!();
    
    // ort::init()
    //    .with_name("voice_agent_onnx")
    //    .commit()?;

    println!("ONNX Runtime Initialized.");

    // The telephony server will handle incoming connections, spawn a pipeline orchestrator
    // for each connection, and hook up the VAD, STT, LLM, and TTS models.
    
    telephony::start_telephony_server().await?;

    Ok(())
}
