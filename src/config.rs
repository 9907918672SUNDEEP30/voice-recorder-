use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// कॉन्फिग-ड्रिवेन मॉडल प्रोफाइल — नो मैजिक नंबर्स!
/// All model inputs, outputs, tensor shapes, sample rates, and thresholds are dynamic.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelProfile {
    pub path: String,
    #[serde(default)]
    pub input_names: Vec<String>,
    #[serde(default)]
    pub output_names: Vec<String>,
    #[serde(default)]
    pub sample_rate: Option<u32>,
    #[serde(default)]
    pub chunk_size: Option<usize>,
    #[serde(default)]
    pub state_shape: Option<Vec<usize>>,
    #[serde(default)]
    pub feature_dimensions: Option<Vec<usize>>,
    #[serde(default)]
    pub max_sequence_length: Option<usize>,
    #[serde(default)]
    pub embedding_dimension: Option<usize>,
    #[serde(default)]
    pub threshold: Option<f32>,
}

/// आपके पूरे AI एजेंट फ्रेमवर्क की सेटिंग्स और मॉडल्स के पाथ
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub models_dir: PathBuf,
    #[serde(default = "default_execution_device")]
    pub execution_device: String, // "auto" (default), "gpu", "cuda", "directml", "cpu"
    
    // Config-driven generic model profiles registry
    #[serde(default)]
    pub models: HashMap<String, ModelProfile>,
}

fn default_execution_device() -> String {
    "auto".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        let base_dir = std::env::var("AI_MODELS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                #[cfg(target_os = "windows")]
                { PathBuf::from("C:\\AI_Models") }
                #[cfg(not(target_os = "windows"))]
                { PathBuf::from("./models") }
            });

        let to_model_path = |sub: &str| -> String {
            base_dir.join(sub).to_string_lossy().to_string()
        };

        let mut models = HashMap::new();
        models.insert(
            "silero_vad".to_string(),
            ModelProfile {
                path: to_model_path("silero_vad.onnx"),
                input_names: vec!["input".to_string(), "state".to_string(), "sr".to_string()],
                output_names: vec!["output".to_string(), "stateN".to_string()],
                sample_rate: Some(16000),
                chunk_size: Some(512),
                state_shape: Some(vec![2, 1, 128]),
                threshold: Some(0.5),
                ..Default::default()
            },
        );
        models.insert(
            "bge_small_embedding".to_string(),
            ModelProfile {
                path: to_model_path("bge_small_embedding/model.onnx"),
                input_names: vec!["input_ids".to_string(), "attention_mask".to_string(), "token_type_ids".to_string()],
                output_names: vec!["last_hidden_state".to_string()],
                max_sequence_length: Some(512),
                embedding_dimension: Some(384),
                ..Default::default()
            },
        );
        models.insert(
            "whisper_encoder".to_string(),
            ModelProfile {
                path: to_model_path("whisper_tiny_onnx/encoder_model.onnx"),
                input_names: vec!["input_features".to_string()],
                output_names: vec!["last_hidden_state".to_string()],
                feature_dimensions: Some(vec![1, 80, 3000]),
                ..Default::default()
            },
        );

        Self {
            models_dir: base_dir.clone(),
            execution_device: "auto".to_string(),
            models,
        }
    }
}

impl AppConfig {
    /// Retrieve a model profile by key (pure generic factory design)
    pub fn get_model(&self, key: &str) -> Result<&ModelProfile> {
        self.models
            .get(key)
            .ok_or_else(|| anyhow!("Model profile '{}' not found in config", key))
    }

    /// Resolves the absolute filesystem path for any configured model
    pub fn resolve_path(&self, key: &str) -> Result<PathBuf> {
        if let Some(profile) = self.models.get(key) {
            let p = Path::new(&profile.path);
            if p.is_absolute() {
                Ok(p.to_path_buf())
            } else {
                Ok(self.models_dir.join(p))
            }
        } else {
            Err(anyhow!("Model profile '{}' not found in config", key))
        }
    }

    /// कॉन्फिगरेशन फाइल को लोड करने का फंक्शन
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        
        if path_ref.exists() {
            let content = std::fs::read_to_string(path_ref)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            println!("⚙️ कॉन्फिगरेशन सफलतापूर्वक लोड कर ली गई है!");
            Ok(config)
        } else {
            println!("⚠️ कॉन्फिगरेशन फाइल नहीं मिली। डिफ़ॉल्ट सेटिंग्स से नई फाइल बनाई जा रही है...");
            let default_config = Self::default();
            
            if let Some(parent) = path_ref.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            
            let json = serde_json::to_string_pretty(&default_config)?;
            std::fs::write(path_ref, json)?;
            
            Ok(default_config)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_execution_device() {
        let config = AppConfig::default();
        assert_eq!(config.execution_device, "auto");
    }

    #[test]
    fn test_model_profile_resolution() {
        let config = AppConfig::default();
        let vad = config.get_model("silero_vad").expect("vad profile exists");
        assert_eq!(vad.sample_rate, Some(16000));
        assert_eq!(vad.chunk_size, Some(512));
        assert_eq!(vad.state_shape, Some(vec![2, 1, 128]));
        assert_eq!(vad.threshold, Some(0.5));
    }
}