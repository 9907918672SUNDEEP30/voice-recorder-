pub mod guard;
pub mod rtsp;
pub mod storage;
pub mod tier1;
pub mod tier2;

pub use guard::{CctvGuardConfig, CctvGuardEngine, GuardTelemetry};
pub use rtsp::{CctvFrame, DownscaledFrame, RtspClient, RtspClientConfig, RtspStreamSource};
pub use storage::{EvidenceVault, EvidenceVaultConfig, IncidentLogRecord};
pub use tier1::{FrameDifferencer, GatekeeperConfig, GatekeeperDecision, Tier1Gatekeeper};
pub use tier2::{
    QwenVlmEngine, ThreatAssessment, ThreatCategory, ThreatLevel, VlmLifecycleManager,
    VlmManagerConfig,
};
