use anyhow::Result;
use ort::session::Session;
use crate::config::AppConfig;
use crate::ml_bridge::MLBridge;

pub struct SttModel {
    pub session: Session,
    pub model_path: String,
}

impl SttModel {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let path = &config.stt_encoder;
        
        println!("📂 STT Model Path: {}", path);
        let session = MLBridge::create_session(path)?;
        
        Ok(Self {
            session,
            model_path: path.clone(),
        })
    }

    pub fn transcribe(&self, audio_path: &str) -> Result<()> {
        println!("🎤 STT audio transcribe kar raha hai: {}", audio_path);
        Ok(())
    }
}