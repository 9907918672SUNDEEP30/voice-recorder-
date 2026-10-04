use anyhow::Result;
use ort::session::Session;
use crate::config::AppConfig;
use crate::ml_bridge::MLBridge;

pub struct RerankerModel {
    pub session: Session,
    pub model_path: String,
}

impl RerankerModel {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let path = &config.classifier_model;
        
        println!("📂 Reranker Model Path: {}", path);
        let session = MLBridge::create_session(path)?;
        
        Ok(Self {
            session,
            model_path: path.clone(),
        })
    }

    pub fn rerank(&self, query: &str) -> Result<()> {
        println!("📊 Reranker query ko rank kar raha hai: {}", query);
        Ok(())
    }
}