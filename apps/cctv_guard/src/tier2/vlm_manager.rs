use crate::rtsp::frame::CctvFrame;
use crate::tier2::qwen_vlm::{QwenVlmEngine, ThreatAssessment};
use anyhow::Result;
use std::time::{Duration, Instant};

/// Configuration options for Tier-2 VLM dynamic lifecycle management.
#[derive(Debug, Clone)]
pub struct VlmManagerConfig {
    pub model_path: String,
    /// Unload model from memory if idle for this many seconds (0 = keep resident)
    pub unload_after_idle_secs: u64,
}

impl Default for VlmManagerConfig {
    fn default() -> Self {
        Self {
            model_path: "C:\\AI_Models\\qwen2_vl_2b_int4.onnx".to_string(),
            unload_after_idle_secs: 60,
        }
    }
}

/// Dynamic manager controlling the lifecycle of the Tier-2 Qwen VLM model.
/// Keeps the heavy 2B model completely unloaded or idle until Tier-1 explicitly triggers it.
pub struct VlmLifecycleManager {
    config: VlmManagerConfig,
    engine: QwenVlmEngine,
    last_invocation_time: Option<Instant>,
    total_triggers: usize,
    total_inference_time_ms: u64,
}

impl VlmLifecycleManager {
    /// Constructs a new VLM lifecycle manager.
    pub fn new(config: VlmManagerConfig) -> Self {
        let engine = QwenVlmEngine::new(&config.model_path);
        Self {
            config,
            engine,
            last_invocation_time: None,
            total_triggers: 0,
            total_inference_time_ms: 0,
        }
    }

    /// Invokes the VLM on the high-resolution event frame.
    /// Loads the model on-demand if not already resident in memory.
    pub fn evaluate_event(&mut self, frame: &CctvFrame) -> Result<ThreatAssessment> {
        // Ensure model is loaded on-demand
        if !self.engine.is_loaded() {
            println!("[VLM Lifecycle] Tier-1 triggered! Dynamically activating Tier-2 Qwen VLM...");
            self.engine.load_model()?;
        }

        let assessment = self.engine.analyze_frame(frame)?;
        self.last_invocation_time = Some(Instant::now());
        self.total_triggers += 1;
        self.total_inference_time_ms += assessment.inference_time_ms;

        Ok(assessment)
    }

    /// Performs idle housekeeping. If quiet for longer than `unload_after_idle_secs`,
    /// unloads the model to reclaim RAM on resource-constrained devices.
    pub fn tick_housekeeping(&mut self) {
        if self.config.unload_after_idle_secs == 0 || !self.engine.is_loaded() {
            return;
        }

        if let Some(last_time) = self.last_invocation_time {
            if last_time.elapsed() >= Duration::from_secs(self.config.unload_after_idle_secs) {
                println!(
                    "[VLM Lifecycle] Scene quiet for {}s. Unloading Tier-2 model to reclaim system RAM.",
                    self.config.unload_after_idle_secs
                );
                self.engine.unload_model();
            }
        }
    }

    /// Returns the total number of VLM invocations performed.
    pub fn total_triggers(&self) -> usize {
        self.total_triggers
    }

    /// Computes the average inference time per trigger event in milliseconds.
    pub fn average_inference_time_ms(&self) -> f64 {
        if self.total_triggers > 0 {
            (self.total_inference_time_ms as f64) / (self.total_triggers as f64)
        } else {
            0.0
        }
    }
}
