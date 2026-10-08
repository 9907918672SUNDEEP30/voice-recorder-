use anyhow::Result;
use crate::config::{AppConfig, ModelProfile};
use crate::ml_bridge::MLBridge;

/// Text-to-Speech (TTS) ONNX Tensor Runner
/// Config-driven sample rate and dynamic session access.
pub struct TtsModel {
    pub bridge: MLBridge,
    pub model_key: String,
    pub profile: ModelProfile,
}

impl TtsModel {
    pub fn new(bridge: MLBridge, config: &AppConfig, model_key: &str) -> Result<Self> {
        let profile = config.get_model(model_key)?.clone();
        Ok(Self {
            bridge,
            model_key: model_key.to_string(),
            profile,
        })
    }

    pub fn sample_rate(&self) -> Option<u32> {
        self.profile.sample_rate
    }

    // TODO: Implement forward/synthesize logic here using MLBridge
}