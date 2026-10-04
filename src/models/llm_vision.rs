use anyhow::Result;
use ort::session::Session;
use crate::config::AppConfig;
use crate::ml_bridge::MLBridge;

pub struct LlmVisionModel {
    pub session: Session,
    pub model_path: String,
}

impl LlmVisionModel {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let path = &config.vision_model;
        
        println!("📂 LLM Vision Model Path: {}", path);
        let session = MLBridge::create_session(path)?;
        
        Ok(Self {
            session,
            model_path: path.clone(),
        })
    }

    pub fn generate(&self, prompt: &str) -> Result<()> {
        println!("🤖 LLM Vision prompt process kar raha hai: {}", prompt);
        Ok(())
    }
}