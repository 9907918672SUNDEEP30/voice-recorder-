use anyhow::{anyhow, Result};
use ndarray::{ArrayD, IxDyn};
use ort::inputs;
use ort::value::Tensor;
use std::sync::Mutex;
use crate::config::{AppConfig, ModelProfile};
use crate::ml_bridge::MLBridge;

/// Silero VAD ONNX Tensor Runner
/// 100% Config-Driven: No hardcoded sample rates, chunk sizes, or tensor shapes.
pub struct VadModel {
    bridge: MLBridge,
    model_key: String,
    profile: ModelProfile,
    state: Mutex<ArrayD<f32>>,
}

impl VadModel {
    pub fn new(bridge: MLBridge, config: &AppConfig, model_key: &str) -> Result<Self> {
        let profile = config.get_model(model_key)?.clone();
        
        let state_shape = profile.state_shape
            .as_ref()
            .ok_or_else(|| anyhow!("VAD model '{}' missing state_shape in config", model_key))?;
        
        let initial_state = ArrayD::zeros(IxDyn(state_shape));

        Ok(Self {
            bridge,
            model_key: model_key.to_string(),
            profile,
            state: Mutex::new(initial_state),
        })
    }

    /// ऑडियो चंक (Audio Samples) लेकर प्रोबेबिलिटी निकालता है
    pub fn process_chunk(&self, audio_chunk: &[f32]) -> Result<f32> {
        let chunk_size = self.profile.chunk_size
            .ok_or_else(|| anyhow!("chunk_size not configured for {}", self.model_key))?;
        let sample_rate = self.profile.sample_rate
            .ok_or_else(|| anyhow!("sample_rate not configured for {}", self.model_key))?;

        if audio_chunk.len() != chunk_size {
            return Err(anyhow!(
                "Audio chunk length ({}) does not match configured chunk_size ({})",
                audio_chunk.len(),
                chunk_size
            ));
        }

        let session = self.bridge.get_or_load_session(&self.model_key)?;
        let mut session_guard = session.lock().map_err(|_| anyhow!("Session mutex poisoned"))?;
        let mut state_guard = self.state.lock().map_err(|_| anyhow!("Mutex poisoned"))?;

        // 1. Prepare dynamic tensors from configuration
        let audio_tensor = Tensor::from_array(([1usize, chunk_size], audio_chunk.to_vec().into_boxed_slice()))?;
        let sr_tensor = Tensor::from_array(([1usize], vec![sample_rate as i64].into_boxed_slice()))?;
        
        let state_shape = state_guard.shape().to_vec();
        let state_slice = state_guard.as_slice()
            .ok_or_else(|| anyhow!("State array is not contiguous"))?
            .to_vec()
            .into_boxed_slice();
        let state_tensor = Tensor::from_array((state_shape, state_slice))?;

        if self.profile.input_names.len() < 3 {
            return Err(anyhow!("VAD requires at least 3 input names (input, state, sr) in config"));
        }

        // 2. Bind inputs based on configured input_names
        let inputs = inputs![
            self.profile.input_names[0].as_str() => audio_tensor,
            self.profile.input_names[1].as_str() => state_tensor,
            self.profile.input_names[2].as_str() => sr_tensor,
        ];

        // 3. Run ONNX tensor inference
        let outputs = session_guard.run(inputs)?;

        // 4. Extract probability & update recurrent state dynamically
        let output_name = &self.profile.output_names[0];
        let prob_view = outputs[output_name.as_str()].try_extract_array::<f32>()?;
        let prob = prob_view[[0, 0]];

        if self.profile.output_names.len() > 1 {
            let state_out_name = &self.profile.output_names[1];
            if let Ok(new_state) = outputs[state_out_name.as_str()].try_extract_array::<f32>() {
                *state_guard = new_state.to_owned();
            }
        }

        Ok(prob)
    }

    /// थ्रेशोल्ड के आधार पर आवाज़ का पता लगाना (Voice Activity Detection)
    pub fn is_speech(&self, audio_chunk: &[f32]) -> Result<bool> {
        let threshold = self.profile.threshold.unwrap_or(0.5);
        let prob = self.process_chunk(audio_chunk)?;
        Ok(prob >= threshold)
    }

    /// VAD की रिकरेंट स्टेट को रीसेट करना
    pub fn reset_state(&self) -> Result<()> {
        let state_shape = self.profile.state_shape
            .as_ref()
            .ok_or_else(|| anyhow!("state_shape not found in config"))?;
        let mut state_guard = self.state.lock().map_err(|_| anyhow!("Mutex poisoned"))?;
        *state_guard = ArrayD::zeros(IxDyn(state_shape));
        Ok(())
    }
}