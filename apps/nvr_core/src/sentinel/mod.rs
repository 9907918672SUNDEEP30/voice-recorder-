//! Sentinel Subsystem (Tier-1 Edge Gatekeeper "पहरेदार")

pub mod dssim;
pub mod gatekeeper;
pub mod luma;

pub use dssim::StructuralComparator;
pub use gatekeeper::{GatekeeperDecision, SentinelGatekeeper};
pub use luma::{LumaDownscaler, LumaGrid};
