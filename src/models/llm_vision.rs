use anyhow::Result;
use crate::config::{AppConfig, ModelProfile};
use crate::ml_bridge::MLBridge;

/// Vision / Multimodal LLM ONNX Model Runner
/// Config-driven session loading via MLBridge.
pub struct LlmVisionModel {
    pub bridge: MLBridge,
    pub model_key: String,
    pub profile: ModelProfile,
}

impl LlmVisionModel {
    pub fn new(bridge: MLBridge, config: &AppConfig, model_key: &str) -> Result<Self> {
        let profile = config.get_model(model_key)?.clone();
        Ok(Self {
            bridge,
            model_key: model_key.to_string(),
            profile,
        })
    }

    // TODO: Implement forward/inference logic here using MLBridge
}