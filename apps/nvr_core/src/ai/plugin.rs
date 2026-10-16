//! Pluggable, Fault-Tolerant AI Security Plugin Layer
//! Strictly decouples the AI intelligence from core CCTV recording and HDMI display.
//! If the AI model is disabled, runs out of memory, or crashes, the CCTV NVR continues 24/7 uninterrupted.

use super::model::{EdgeVisionModel, ThreatVerdict};
use crate::ingestion::VideoFrame;
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// Status of the AI Plugin Engine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiPluginStatus {
    /// AI is active and processing candidate frames
    Active,
    /// AI is disabled by configuration (Standby mode; CCTV runs alone)
    Disabled,
    /// AI crashed or failed to load model (Recovering; CCTV runs unaffected)
    FaultStandby,
}

/// Pluggable AI Security Controller
pub struct PluggableAiEngine {
    enabled: Arc<AtomicBool>,
    status: Arc<parking_lot::RwLock<AiPluginStatus>>,
    frame_sender: Option<Sender<(VideoFrame, f32)>>,
    verdict_receiver: Receiver<ThreatVerdict>,
    _worker_handle: Option<JoinHandle<()>>,
}

impl PluggableAiEngine {
    /// Initialize the pluggable AI engine. If enabled is false, AI thread is not even spawned.
    pub fn new(enabled: bool, model_name: &str) -> Self {
        let (frame_tx, frame_rx) = bounded::<(VideoFrame, f32)>(8); // Bounded queue: never buffers uncontrollably in RAM
        let (verdict_tx, verdict_rx) = bounded::<ThreatVerdict>(8);

        let is_enabled = Arc::new(AtomicBool::new(enabled));
        let status = Arc::new(parking_lot::RwLock::new(if enabled {
            AiPluginStatus::Active
        } else {
            AiPluginStatus::Disabled
        }));

        if !enabled {
            println!("🔌 [AI PLUGIN] Disabled by configuration. CCTV Core running in pure NVR mode.");
            return Self {
                enabled: is_enabled,
                status,
                frame_sender: None,
                verdict_receiver: verdict_rx,
                _worker_handle: None,
            };
        }

        let worker_enabled = Arc::clone(&is_enabled);
        let worker_status = Arc::clone(&status);
        let model_name_str = model_name.to_string();

        let handle = thread::Builder::new()
            .name("nvr-ai-worker".into())
            .spawn(move || {
                println!("🧠 [AI WORKER THREAD] Initialized with model: {}", model_name_str);
                let model = EdgeVisionModel::new(&model_name_str);

                while worker_enabled.load(Ordering::Relaxed) {
                    match frame_rx.recv() {
                        Ok((frame, dssim_score)) => {
                            // Safe catch_unwind isolation: An AI model crash never crashes the CCTV engine!
                            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                model.analyze_frame(&frame.image, dssim_score)
                            }));

                            match result {
                                Ok(verdict) => {
                                    let _ = verdict_tx.try_send(verdict);
                                }
                                Err(_) => {
                                    eprintln!("⚠️ [AI FAULT ISOLATION] AI Inference panicked! Isolating error...");
                                    *worker_status.write() = AiPluginStatus::FaultStandby;
                                }
                            }
                        }
                        Err(_) => break, // Channel closed
                    }
                }
                println!("🛑 [AI WORKER THREAD] Gracefully stopped.");
            })
            .ok();

        Self {
            enabled: is_enabled,
            status,
            frame_sender: Some(frame_tx),
            verdict_receiver: verdict_rx,
            _worker_handle: handle,
        }
    }

    /// Check if AI plugin is currently active
    pub fn is_active(&self) -> bool {
        *self.status.read() == AiPluginStatus::Active
    }

    /// Send a candidate anomaly frame to the AI layer (NON-BLOCKING).
    /// If AI is busy or queue is full, drops frame gracefully: CCTV RECORDING NEVER WAITS!
    pub fn submit_candidate_frame(&self, frame: VideoFrame, dssim_score: f32) {
        if !self.is_active() {
            return;
        }

        if let Some(sender) = &self.frame_sender {
            match sender.try_send((frame, dssim_score)) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => {
                    // Queue full: drop frame for AI, but CCTV video recording continues at full 25/30 FPS
                }
                Err(TrySendError::Disconnected(_)) => {
                    *self.status.write() = AiPluginStatus::FaultStandby;
                }
            }
        }
    }

    /// Poll for any available AI verdicts (NON-BLOCKING)
    pub fn poll_verdict(&self) -> Option<ThreatVerdict> {
        self.verdict_receiver.try_recv().ok()
    }

    /// Dynamically toggle AI on/off at runtime without restarting CCTV
    pub fn set_enabled(&self, enable: bool) {
        self.enabled.store(enable, Ordering::Relaxed);
        let mut st = self.status.write();
        *st = if enable {
            AiPluginStatus::Active
        } else {
            AiPluginStatus::Disabled
        };
        println!("🔌 [AI PLUGIN] Runtime toggle -> Active: {}", enable);
    }
}
