use anyhow::{anyhow, Result};
use ort::inputs;
use ort::value::Tensor;
use crate::config::{AppConfig, ModelProfile};
use crate::ml_bridge::MLBridge;

/// Speech-to-Text / Audio Encoder ONNX Tensor Runner
/// 100% Config-Driven: Feature dimensions, input names, output names are dynamic.
pub struct SttModel {
    bridge: MLBridge,
    model_key: String,
    profile: ModelProfile,
}

impl SttModel {
    pub fn new(bridge: MLBridge, config: &AppConfig, model_key: &str) -> Result<Self> {
        let profile = config.get_model(model_key)?.clone();
        Ok(Self {
            bridge,
            model_key: model_key.to_string(),
            profile,
        })
    }

    /// ऑडियो फीचर स्पेक्ट्रोग्राम (Mel Spectrogram) से एनकोडर हिडन स्टेट्स निकालना
    pub fn encode_features(&self, features_flat: &[f32]) -> Result<Vec<f32>> {
        let feature_dims = self.profile.feature_dimensions
            .as_ref()
            .ok_or_else(|| anyhow!("feature_dimensions not configured for STT model '{}'", self.model_key))?;

        let total_expected: usize = feature_dims.iter().product();
        if features_flat.len() != total_expected {
            return Err(anyhow!(
                "Provided feature buffer size ({}) does not match configured dimensions {:?} (expected: {})",
                features_flat.len(),
                feature_dims,
                total_expected
            ));
        }

        let session = self.bridge.get_or_load_session(&self.model_key)?;
        let mut session_guard = session.lock().map_err(|_| anyhow!("Session mutex poisoned"))?;

        let tensor = Tensor::from_array((feature_dims.clone(), features_flat.to_vec().into_boxed_slice()))?;

        let input_name = self.profile.input_names.first()
            .ok_or_else(|| anyhow!("input_names empty in STT model config"))?;

        let outputs = session_guard.run(inputs![
            input_name.as_str() => tensor,
        ])?;

        let output_name = self.profile.output_names.first()
            .ok_or_else(|| anyhow!("output_names empty in STT model config"))?;

        let out_array = outputs[output_name.as_str()].try_extract_array::<f32>()?;
        
        // Return flattened tensor vector
        Ok(out_array.iter().copied().collect())
    }
}