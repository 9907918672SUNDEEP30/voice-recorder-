pub mod frame_diff;
pub mod gatekeeper;
pub mod ssim;

pub use frame_diff::{FrameDifferencer, MotionMetrics};
pub use gatekeeper::{GatekeeperConfig, GatekeeperDecision, Tier1Gatekeeper};
pub use ssim::StructuralDissimilarity;
