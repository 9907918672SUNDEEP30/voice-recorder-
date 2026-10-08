use anyhow::{anyhow, Result};
use ort::inputs;
use ort::value::Tensor;
use crate::config::{AppConfig, ModelProfile};
use crate::ml_bridge::MLBridge;

/// Universal ONNX Embedding Runner
/// 100% Config-Driven: Sequence lengths, embedding dimensions, input and output tensor names
/// are resolved entirely from AppConfig.
pub struct EmbeddingModel {
    bridge: MLBridge,
    model_key: String,
    profile: ModelProfile,
}

impl EmbeddingModel {
    pub fn new(bridge: MLBridge, config: &AppConfig, model_key: &str) -> Result<Self> {
        let profile = config.get_model(model_key)?.clone();
        Ok(Self {
            bridge,
            model_key: model_key.to_string(),
            profile,
        })
    }

    /// टोकन इनपुट्स से डेंस एम्बेडिंग वेक्टर निकालना
    pub fn forward_tokens(
        &self,
        input_ids: &[i64],
        attention_mask: &[i64],
    ) -> Result<Vec<f32>> {
        let seq_len = input_ids.len();
        if seq_len != attention_mask.len() {
            return Err(anyhow!(
                "input_ids length ({}) does not match attention_mask length ({})",
                seq_len,
                attention_mask.len()
            ));
        }

        if let Some(max_len) = self.profile.max_sequence_length {
            if seq_len > max_len {
                return Err(anyhow!(
                    "Token sequence length {} exceeds configured max_sequence_length {}",
                    seq_len,
                    max_len
                ));
            }
        }

        let session = self.bridge.get_or_load_session(&self.model_key)?;
        let mut session_guard = session.lock().map_err(|_| anyhow!("Session mutex poisoned"))?;

        let ids_tensor = Tensor::from_array(([1usize, seq_len], input_ids.to_vec().into_boxed_slice()))?;
        let mask_tensor = Tensor::from_array(([1usize, seq_len], attention_mask.to_vec().into_boxed_slice()))?;

        let input_id_name = self.profile.input_names.first()
            .ok_or_else(|| anyhow!("input_names empty in embedding config"))?;
        let mask_name = self.profile.input_names.get(1)
            .ok_or_else(|| anyhow!("attention_mask input name missing in embedding config"))?;

        let outputs = if self.profile.input_names.len() >= 3 {
            let token_type_ids = Tensor::from_array(([1usize, seq_len], vec![0i64; seq_len].into_boxed_slice()))?;
            let type_name = &self.profile.input_names[2];
            session_guard.run(inputs![
                input_id_name.as_str() => ids_tensor,
                mask_name.as_str() => mask_tensor,
                type_name.as_str() => token_type_ids,
            ])?
        } else {
            session_guard.run(inputs![
                input_id_name.as_str() => ids_tensor,
                mask_name.as_str() => mask_tensor,
            ])?
        };

        let output_name = self.profile.output_names.first()
            .ok_or_else(|| anyhow!("output_names empty in embedding config"))?;

        let output_array = outputs[output_name.as_str()].try_extract_array::<f32>()?;
        let shape = output_array.shape();

        // 3D tensor: [batch_size, seq_len, hidden_dim] -> Mean pooling over sequence
        if shape.len() == 3 {
            let dim = shape[2];
            let mut pooled = vec![0.0f32; dim];
            for token_idx in 0..seq_len {
                for d in 0..dim {
                    pooled[d] += output_array[[0, token_idx, d]];
                }
            }
            for d in 0..dim {
                pooled[d] /= seq_len as f32;
            }
            Ok(pooled)
        } else if shape.len() == 2 {
            // Already pooled: [batch_size, hidden_dim]
            let dim = shape[1];
            let mut pooled = vec![0.0f32; dim];
            for d in 0..dim {
                pooled[d] = output_array[[0, d]];
            }
            Ok(pooled)
        } else {
            Err(anyhow!("Unexpected embedding tensor output shape: {:?}", shape))
        }
    }
}