use anyhow::Result;
use ort::session::Session;
use crate::config::AppConfig;
use crate::ml_bridge::MLBridge;

pub struct EmbeddingModel {
    pub session: Session,
    pub model_path: String,
}

impl EmbeddingModel {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let path = &config.embedding_model;
        
        println!("📂 Embedding Model Path: {}", path);
        let session = MLBridge::create_session(path)?;
        
        Ok(Self {
            session,
            model_path: path.clone(),
        })
    }

    pub fn encode(&self, text: &str) -> Result<()> {
        println!("🔍 Embedding generate ki ja rahi hai text ke liye: {}", text);
        Ok(())
    }
}