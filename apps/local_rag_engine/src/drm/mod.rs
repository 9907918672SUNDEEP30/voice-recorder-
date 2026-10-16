//! # DRM Module
//!
//! Exposes hardware fingerprinting, 24-hour leasing, and anti-tamper security guard.

pub mod heartbeat;
pub mod machine;

pub use heartbeat::{DrmGuard, DrmInfo, HeartbeatRequest, HeartbeatResponse, LicenseStatus};
pub use machine::MachineFingerprint;
