//! # DRM Guard Crate
//!
//! Universal Shared DRM Guard & Metered Usage Licensing Framework for AI Core Framework.

pub mod clock;
pub mod guard;
pub mod hardware;
pub mod heartbeat;
pub mod session_key;
pub mod telemetry;
pub mod updater;

pub use clock::MonotonicClockGuard;
pub use guard::{DRMGuard, DRMGuardBuilder};
pub use hardware::MachineFingerprint;
pub use heartbeat::{DrmStatusInfo, HeartbeatRequest, HeartbeatResponse, LicenseStatus, UpdateMetadata};
pub use session_key::{AesGcmCipher, VolatileSessionKey};
pub use telemetry::{
    CctvGuardMetrics, DspRecorderMetrics, DynamicMetrics, FinancialMetrics, ManifestationMetrics,
    NoOpMetrics, RagEngineMetrics, UsageMetricProvider, VideoStudioMetrics, VoiceAgentMetrics,
};
pub use updater::{OtaStatus, OtaUpdater};



