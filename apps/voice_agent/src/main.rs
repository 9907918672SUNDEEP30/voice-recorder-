mod codec;
mod pipeline;
mod telephony;
mod prompts;
mod supabase;

use anyhow::Result;
use std::fs;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use ai_core::models::onnx_adapters::{SileroVadLocal, WhisperTinyLocal, QwenLocal, OpenVoiceLocal};
use ai_core::models::adapters::{VADAdapter, SttAdapter, LlmAdapter, TtsAdapter};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎙️ ========================================================");
    println!("🎙️  Voice AI Calling Agent (Telephony + Browser Simulator) ");
    println!("🎙️  Shambhavi AI Lab - 100% Local ONNX Production Core     ");
    println!("🎙️ ========================================================");

    // Robust multi-path resolution for config.json
    let config_paths = ["config.json", "../../config.json", "../config.json"];
    let mut config_data = "{}".to_string();
    for p in &config_paths {
        if let Ok(c) = fs::read_to_string(p) {
            println!("📄 Loaded configuration from: {}", p);
            config_data = c;
            break;
        }
    }
    let config: Value = serde_json::from_str(&config_data).unwrap_or(serde_json::json!({}));
    
    let base_dir = config["models_dir"].as_str().unwrap_or("C:\\AI_Models");
    let silero_path = format!("{}\\{}", base_dir, config["models"]["silero_vad"]["path"].as_str().unwrap_or("silero_vad.onnx"));
    
    let whisper_enc = format!("{}\\{}", base_dir, "whisper_tiny_onnx\\encoder_model_q4.onnx");
    let whisper_dec = format!("{}\\{}", base_dir, "whisper_tiny_onnx\\decoder_model_q4.onnx");
    let whisper_tok = format!("{}\\{}", base_dir, "whisper_tiny_onnx\\tokenizer.json");
    
    let qwen_llm = format!("{}\\{}", base_dir, "qwen2-vl-2b\\llmmodel.onnx");
    let qwen_vis = format!("{}\\{}", base_dir, "qwen2-vl-2b\\visionmodel.onnx");
    let qwen_vis_enc = format!("{}\\{}", base_dir, "qwen2-vl-2b\\vision_encoder.onnx");
    let qwen_tok = format!("{}\\{}", base_dir, "qwen2-vl-2b\\tokenizer.json");

    let openvoice_color_path = format!("{}\\{}", base_dir, config["models"]["openvoice_tts"]["path_tone_color"].as_str().unwrap_or("OpenVoice-ONNX-v2\\tone_color.onnx"));
    let openvoice_extract_path = format!("{}\\{}", base_dir, config["models"]["openvoice_tts"]["path_tone_extract"].as_str().unwrap_or("OpenVoice-ONNX-v2\\tone_extract.onnx"));
    let piper_path = format!("{}\\{}", base_dir, config["models"]["piper_tts"]["path"].as_str().unwrap_or("piper_hi\\hi_IN-priyamvada-medium-int8.onnx"));

    println!("\n🔍 Active Local Model Configuration:");
    println!(" - VAD: {}", silero_path);
    println!(" - STT Encoder: {}", whisper_enc);
    println!(" - STT Decoder: {}", whisper_dec);
    println!(" - LLM Brain: {}", qwen_llm);
    println!(" - TTS Base (Piper): {}", piper_path);
    println!();
    
    ort::init()
        .with_name("voice_agent_onnx")
        .commit();
    println!("⚡ ONNX Runtime Initialized.");

    // 1. Initialize Adapters
    let vad_adapter = Arc::new(Mutex::new(SileroVadLocal::new(&silero_path)?)) as Arc<Mutex<dyn VADAdapter>>;
    println!("✅ Silero VAD Loaded successfully.");
    
    let stt_adapter = Arc::new(WhisperTinyLocal::new(&whisper_enc, &whisper_dec, &whisper_tok)?) as Arc<dyn SttAdapter>;
    println!("✅ Whisper STT Loaded successfully.");

    let llm_adapter = Arc::new(QwenLocal::new(&qwen_llm, &qwen_vis, &qwen_vis_enc, &qwen_tok)?) as Arc<dyn LlmAdapter>;
    println!("✅ Qwen Multimodal LLM Loaded successfully.");

    let reference_audio_path = "sweet_girl_reference.wav";
    let tts_adapter = Arc::new(OpenVoiceLocal::new(&piper_path, &openvoice_color_path, &openvoice_extract_path, reference_audio_path)?) as Arc<dyn TtsAdapter>;
    println!("✅ Piper + OpenVoice TTS Loaded successfully.");

    // 2. Initialize Supabase / Cloud Call Logger
    let supabase_client = Arc::new(supabase::SupabaseClient::new_from_env_or_config(&config));

    // 3. Start Telephony Gateway & Browser Simulator
    telephony::start_telephony_server(vad_adapter, stt_adapter, llm_adapter, tts_adapter, supabase_client).await?;

    Ok(())
}
