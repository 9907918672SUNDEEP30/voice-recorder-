use crate::rtsp::frame::CctvFrame;
use crate::storage::evidence::{EvidenceVault, EvidenceVaultConfig, IncidentLogRecord};
use crate::tier1::gatekeeper::{GatekeeperConfig, GatekeeperDecision, Tier1Gatekeeper};
use crate::tier2::vlm_manager::{VlmLifecycleManager, VlmManagerConfig};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

/// Configuration for the unified CCTV Guard system.
#[derive(Debug, Clone)]
pub struct CctvGuardConfig {
    pub camera_id: String,
    pub gatekeeper: GatekeeperConfig,
    pub vlm: VlmManagerConfig,
    pub storage: EvidenceVaultConfig,
}

impl Default for CctvGuardConfig {
    fn default() -> Self {
        Self {
            camera_id: "cam_bank_front_01".to_string(),
            gatekeeper: GatekeeperConfig::default(),
            vlm: VlmManagerConfig::default(),
            storage: EvidenceVaultConfig::default(),
        }
    }
}

/// Statistics and telemetry emitted during 24/7 background execution.
#[derive(Debug, Default, Clone)]
pub struct GuardTelemetry {
    pub total_frames_processed: u64,
    pub tier1_anomalies_detected: u64,
    pub tier2_vlm_triggers: u64,
    pub critical_threats_logged: u64,
    pub average_fps: f32,
}

/// Master CCTV Guard engine orchestrating Tier-1 Edge Gatekeeper,
/// on-demand Tier-2 Qwen VLM inference, and evidence recording.
pub struct CctvGuardEngine {
    config: CctvGuardConfig,
    gatekeeper: Tier1Gatekeeper,
    vlm_manager: VlmLifecycleManager,
    vault: EvidenceVault,
    telemetry: GuardTelemetry,
    start_time: Instant,
    is_running: Arc<AtomicBool>,
}

impl CctvGuardEngine {
    /// Constructs a new CCTV Guard engine with millisecond telemetry.
    pub fn new(config: CctvGuardConfig) -> Result<Self> {
        let gatekeeper = Tier1Gatekeeper::new(config.gatekeeper.clone());
        let vlm_manager = VlmLifecycleManager::new(config.vlm.clone());
        let vault = EvidenceVault::new(config.storage.clone())?;

        Ok(Self {
            config,
            gatekeeper,
            vlm_manager,
            vault,
            telemetry: GuardTelemetry::default(),
            start_time: Instant::now(),
            is_running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Evaluates a single CCTV video frame through the two-tier pipeline.
    /// Returns an optional `IncidentLogRecord` if a Tier-2 trigger occurred.
    pub fn process_single_frame(&mut self, frame: CctvFrame) -> Result<Option<IncidentLogRecord>> {
        self.telemetry.total_frames_processed += 1;

        // 1. Tier-1 Edge Gatekeeper analysis (< 5% CPU)
        let decision = self.gatekeeper.evaluate_frame(frame.clone());

        match decision {
            GatekeeperDecision::Normal { anomaly_score: _ } => {
                // Housekeeping: unload VLM if idle for too long
                self.vlm_manager.tick_housekeeping();
                Ok(None)
            }
            GatekeeperDecision::SuppressedCooldown { anomaly_score, remaining_secs } => {
                if self.telemetry.total_frames_processed % 30 == 0 {
                    self.log_status(&format!(
                        "[{}] Ongoing event (score: {:.2}). Suppressed by cooldown ({:.1}s remaining).",
                        self.config.camera_id, anomaly_score, remaining_secs
                    ));
                }
                Ok(None)
            }
            GatekeeperDecision::TriggerVlm {
                anomaly_score,
                event_frame,
                pre_buffer: _,
                trigger_reason,
            } => {
                self.telemetry.tier1_anomalies_detected += 1;
                self.telemetry.tier2_vlm_triggers += 1;

                self.log_status(&format!(
                    "🚨 [{}] TIER-1 TRIGGER: {}. Invoking Tier-2 Qwen VLM...",
                    self.config.camera_id, trigger_reason
                ));

                // 2. Tier-2 Qwen VLM Inference
                let assessment = self.vlm_manager.evaluate_event(&event_frame)?;

                self.log_status(&format!(
                    "🧠 [{}] VLM Assessment: {:?} | {:?} | Conf: {:.2} | Latency: {}ms\n    Details: {}",
                    self.config.camera_id,
                    assessment.threat_level,
                    assessment.category,
                    assessment.confidence,
                    assessment.inference_time_ms,
                    assessment.description
                ));

                if matches!(assessment.threat_level, crate::tier2::ThreatLevel::Critical) {
                    self.telemetry.critical_threats_logged += 1;
                }

                // 3. Save snapshot & record structured JSON log
                let record = self.vault.record_incident(
                    &event_frame,
                    &assessment,
                    anomaly_score,
                    &self.config.camera_id,
                )?;

                self.log_status(&format!(
                    "💾 [{}] Incident logged to {:?}. Snapshot: {:?}",
                    self.config.camera_id,
                    self.vault.log_path(),
                    record.snapshot_path
                ));

                Ok(Some(record))
            }
        }
    }

    /// Runs the continuous 24/7 background guard loop on an incoming RTSP frame channel.
    pub async fn run_loop(
        &mut self,
        mut frame_rx: mpsc::Receiver<CctvFrame>,
        is_running: Arc<AtomicBool>,
    ) -> Result<()> {
        self.is_running = is_running.clone();
        let start_time = Instant::now();

        self.log_status(&format!(
            "🟢 [{}] CCTV Guard 24/7 background engine started. Awaiting frames...",
            self.config.camera_id
        ));

        while is_running.load(Ordering::SeqCst) {
            match frame_rx.recv().await {
                Some(frame) => {
                    let _ = self.process_single_frame(frame)?;

                    if self.telemetry.total_frames_processed % 100 == 0 {
                        let elapsed_sec = start_time.elapsed().as_secs_f32();
                        if elapsed_sec > 0.0 {
                            self.telemetry.average_fps =
                                self.telemetry.total_frames_processed as f32 / elapsed_sec;
                        }

                        println!(
                            "[Telemetry] Processed: {} frames | FPS: {:.1} | Tier-1 Triggers: {} | VLM Invocations: {} | Critical: {}",
                            self.telemetry.total_frames_processed,
                            self.telemetry.average_fps,
                            self.telemetry.tier1_anomalies_detected,
                            self.telemetry.tier2_vlm_triggers,
                            self.telemetry.critical_threats_logged
                        );
                    }
                }
                None => {
                    self.log_status("Frame channel closed. Terminating guard loop.");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Internal logger writing formatted status messages to stdout.
    fn log_status(&self, msg: &str) {
        println!("[+{:06}ms] {}", self.start_time.elapsed().as_millis(), msg);
    }

    /// Returns current telemetry stats.
    pub fn telemetry(&self) -> &GuardTelemetry {
        &self.telemetry
    }
}
