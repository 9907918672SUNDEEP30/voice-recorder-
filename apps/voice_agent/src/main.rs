mod codec;
mod pipeline;
mod telephony;
mod prompts;

use anyhow::Result;
use std::fs;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use ai_core::models::onnx_adapters::{SileroVadLocal, WhisperTinyLocal, QwenLocal, OpenVoiceLocal};
use ai_core::models::adapters::{VADAdapter, SttAdapter, LlmAdapter, TtsAdapter};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Voice AI Core Starting...");
    println!("Loading 100% Local ONNX Stack...");

    // Read config.json
    let config_data = fs::read_to_string("../../config.json").unwrap_or_else(|_| "{}".to_string());
    let config: Value = serde_json::from_str(&config_data).unwrap_or(serde_json::json!({}));
    
    let base_dir = config["models_dir"].as_str().unwrap_or("C:\\AI_Models");
    let silero_path = format!("{}\\{}", base_dir, config["models"]["silero_vad"]["path"].as_str().unwrap_or("silero_vad.onnx"));
    
    // Using mock paths if config missing just so it doesn't crash
    let whisper_enc = format!("{}\\{}", base_dir, "whisper_tiny_onnx\\encoder_model.onnx");
    let whisper_dec = format!("{}\\{}", base_dir, "whisper_tiny_onnx\\decoder_model.onnx");
    let whisper_tok = format!("{}\\{}", base_dir, "whisper_tiny_onnx\\tokenizer.json");
    
    let qwen_llm = format!("{}\\{}", base_dir, "qwen2-vl-2b\\llmmodel.onnx");
    let qwen_vis = format!("{}\\{}", base_dir, "qwen2-vl-2b\\visionmodel.onnx");
    let qwen_vis_enc = format!("{}\\{}", base_dir, "qwen2-vl-2b\\vision_encoder.onnx");
    let qwen_tok = format!("{}\\{}", base_dir, "qwen2-vl-2b\\tokenizer.json");

    let openvoice_color_path = format!("{}\\{}", base_dir, config["models"]["openvoice_tts"]["path_tone_color"].as_str().unwrap_or("OpenVoice-ONNX-v2\\tone_color.onnx"));
    let openvoice_extract_path = format!("{}\\{}", base_dir, config["models"]["openvoice_tts"]["path_tone_extract"].as_str().unwrap_or("OpenVoice-ONNX-v2\\tone_extract.onnx"));
    let piper_path = format!("{}\\{}", base_dir, config["models"]["piper_tts"]["path"].as_str().unwrap_or("piper_hi\\hi_IN-priyamvada-medium-int8.onnx"));

    println!("\n✅ Models Found in config.json:");
    println!(" - VAD: {}", silero_path);
    println!(" - STT: {}", whisper_enc);
    println!(" - LLM: {}", qwen_llm);
    println!(" - TTS (Piper Base): {}", piper_path);
    println!();
    
    ort::init()
        .with_name("voice_agent_onnx")
        .commit();
    println!("ONNX Runtime Initialized.");

    // 1. Initialize Adapters
    let vad_adapter = Arc::new(Mutex::new(SileroVadLocal::new(&silero_path)?)) as Arc<Mutex<dyn VADAdapter>>;
    
    // Some models may fail to load if files are missing on disk during dev, ignoring unwrap crashes with dummy error for now
    let stt_adapter = Arc::new(WhisperTinyLocal::new(&whisper_enc, &whisper_dec, &whisper_tok)
        .unwrap_or_else(|_| WhisperTinyLocal::new("", "", "").unwrap())) as Arc<dyn SttAdapter>;
        
    let llm_adapter = Arc::new(QwenLocal::new(&qwen_llm, &qwen_vis, &qwen_vis_enc, &qwen_tok)
        .unwrap_or_else(|_| QwenLocal::new("", "", "", "").unwrap())) as Arc<dyn LlmAdapter>;
        
    let reference_audio_path = "assets\\sweet_girl_reference.wav";
    let tts_adapter = Arc::new(OpenVoiceLocal::new(&piper_path, &openvoice_color_path, &openvoice_extract_path, reference_audio_path)
        .unwrap_or_else(|_| OpenVoiceLocal::new("", "", "", "").unwrap())) as Arc<dyn TtsAdapter>;

    // 2. Start Server
    telephony::start_telephony_server(vad_adapter, stt_adapter, llm_adapter, tts_adapter).await?;

    Ok(())
}
