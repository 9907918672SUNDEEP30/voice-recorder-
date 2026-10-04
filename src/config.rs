use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::Result;

/// आपके पूरे AI एजेंट फ्रेमवर्क की सेटिंग्स और मॉडल्स के पाथ यहाँ डिफाइन हैं
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub models_dir: PathBuf,
    pub llm_model: String,
    pub vision_model: String,
    pub embedding_model: String,
    pub stt_encoder: String,
    pub stt_decoder: String,
    pub stt_decoder_merged: String,
    pub stt_decoder_with_past: String,
    pub tts_model: String,
    pub classifier_model: String,
    pub silero_vad_model: String, // <-- Silero VAD model ka path yahan joda gaya hai
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            models_dir: PathBuf::from("C:\\AI_Models"),
            llm_model: "C:\\AI_Models\\qwen2-vl-2b\\model.onnx".to_string(),
            vision_model: "C:\\AI_Models\\qwen2-vl-2b\\vision_model.onnx".to_string(),
            embedding_model: "C:\\AI_Models\\bge_small_embedding\\model.onnx".to_string(),
            stt_encoder: "C:\\AI_Models\\whisper_tiny_onnx\\encoder_model.onnx".to_string(),
            stt_decoder: "C:\\AI_Models\\whisper_tiny_onnx\\decoder_model.onnx".to_string(),
            stt_decoder_merged: "C:\\AI_Models\\whisper_tiny_onnx\\decoder_model_merged.onnx".to_string(),
            stt_decoder_with_past: "C:\\AI_Models\\whisper_tiny_onnx\\decoder_model_with_past.onnx".to_string(),
            tts_model: "C:\\AI_Models\\kokoro_tts_onnx\\model.onnx".to_string(),
            classifier_model: "C:\\AI_Models\\bge_reranker\\model.onnx".to_string(),
            silero_vad_model: "C:\\AI_Models\\silero_vad.onnx".to_string(), // <-- Silero VAD ka default path
        }
    }
}

impl AppConfig {
    /// कॉन्फिगरेशन फाइल को लोड करने का फंक्शन
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        
        if path_ref.exists() {
            let content = std::fs::read_to_string(path_ref)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            println!("⚙️ कॉन्फिगरेशन सफलतापूर्वक लोड कर ली गई है!");
            Ok(config)
        } else {
            println!("⚠️ कॉन्फिगरेशन फाइल नहीं मिली। डिफ़ॉल्ट सेटिंग्स से नई फाइल बनाई जा रही है...");
            let default_config = Self::default();
            
            if let Some(parent) = path_ref.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            
            let json = serde_json::to_string_pretty(&default_config)?;
            std::fs::write(path_ref, json)?;
            
            Ok(default_config)
        }
    }
}