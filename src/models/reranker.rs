use anyhow::{anyhow, Result};
use ort::inputs;
use ort::value::Tensor;
use crate::config::{AppConfig, ModelProfile};
use crate::ml_bridge::MLBridge;

/// BGE / Cross-Encoder Reranker ONNX Tensor Runner
/// 100% Config-Driven: Input names, output names, and sequence constraints from config.
pub struct RerankerModel {
    bridge: MLBridge,
    model_key: String,
    profile: ModelProfile,
}

impl RerankerModel {
    pub fn new(bridge: MLBridge, config: &AppConfig, model_key: &str) -> Result<Self> {
        let profile = config.get_model(model_key)?.clone();
        Ok(Self {
            bridge,
            model_key: model_key.to_string(),
            profile,
        })
    }

    /// रीरैंकर के माध्यम से स्कोर निकालना
    pub fn score_pair(
        &self,
        input_ids: &[i64],
        attention_mask: &[i64],
    ) -> Result<f32> {
        let seq_len = input_ids.len();
        if seq_len != attention_mask.len() {
            return Err(anyhow!("input_ids and attention_mask length mismatch"));
        }

        let session = self.bridge.get_or_load_session(&self.model_key)?;
        let mut session_guard = session.lock().map_err(|_| anyhow!("Session mutex poisoned"))?;

        let ids_tensor = Tensor::from_array(([1usize, seq_len], input_ids.to_vec().into_boxed_slice()))?;
        let mask_tensor = Tensor::from_array(([1usize, seq_len], attention_mask.to_vec().into_boxed_slice()))?;

        let input_id_name = self.profile.input_names.first()
            .ok_or_else(|| anyhow!("input_names empty in reranker config"))?;
        let mask_name = self.profile.input_names.get(1)
            .ok_or_else(|| anyhow!("attention_mask input name missing in reranker config"))?;

        let outputs = session_guard.run(inputs![
            input_id_name.as_str() => ids_tensor,
            mask_name.as_str() => mask_tensor,
        ])?;

        let output_name = self.profile.output_names.first()
            .ok_or_else(|| anyhow!("output_names empty in reranker config"))?;

        let logits_array = outputs[output_name.as_str()].try_extract_array::<f32>()?;
        
        let score = logits_array.iter().next().copied().unwrap_or(0.0);
        Ok(score)
    }
}