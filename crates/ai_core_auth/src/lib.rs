//! # AI Core Auth: Unified Tiered Authentication, DRM & Billing Engine
//!
//! Provides a standardized, production-grade security architecture across all 8 applications
//! in the Shambhavi AI Core Framework.
//!
//! ## Tier 1: High-Value Cloud-Connected Dual-Gate (`apps/voice_agent`, `apps/content_engine`)
//! - Strict server-side validation via Cloudflare Worker + Supabase balance ledger.
//! - Rejects execution with 402/403 if credits/tokens are insufficient or invalid.
//!
//! ## Tier 2: Local-First Hybrid (`apps/financial_engine`, `apps/manifestation_app`)
//! - One-time Cloudflare login issues signed Ed25519 JWT stored locally.
//! - Subsequent executions verify JWT signature locally on client CPU (<0.25ms) with zero network calls.
//!
//! ## Tier 3: Local Hardware & NVR Desktop Tools (`apps/cctv_guard`, `apps/local_rag_engine`, `apps/dsp_recorder`, `apps/video_studio`, `apps/nvr_core`)
//! - Machine-bound license key (CPU ID + Motherboard Serial), no username/password.
//! - Periodic 24-hour heartbeat check to Cloudflare edge; locks instantly if revoked.

pub mod hardware;
pub mod tier1_cloud_gate;
pub mod tier2_local_auth;
pub mod tier3_drm_heartbeat;

pub use hardware::get_machine_fingerprint;
pub use tier1_cloud_gate::{
    CloudGateClient, CommitUsageRequest, ExecutionPermit, PreFlightRequest, Tier1CloudGate,
};
pub use tier2_local_auth::{
    LoginResponse, LocalAuthManager, Tier2LocalAuth, UserClaims, EMBEDDED_ROOT_PUBLIC_KEY_HEX,
};
pub use tier3_drm_heartbeat::{
    DrmHeartbeatManager, DrmLease, HeartbeatRequest, HeartbeatResponse, Tier3DrmHeartbeat,
};
