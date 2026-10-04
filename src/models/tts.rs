use anyhow::Result;
use ort::session::Session;
use crate::config::AppConfig;
use crate::ml_bridge::MLBridge;

pub struct TtsModel {
    pub session: Session,
    pub model_path: String,
}

impl TtsModel {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let path = &config.tts_model;
        
        println!("📂 TTS Model Path: {}", path);
        let session = MLBridge::create_session(path)?;
        
        Ok(Self {
            session,
            model_path: path.clone(),
        })
    }

    pub fn synthesize(&self, text: &str) -> Result<()> {
        println!("🔊 TTS text ko speech me convert kar raha hai: {}", text);
        Ok(())
    }
}