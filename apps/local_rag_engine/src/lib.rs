//! # Local RAG Engine Library
//!
//! Native Rust Local RAG Document Search, CRM, & 24-Hour Cloudflare DRM Engine.

pub mod crm;
pub mod drm;
pub mod engine;
pub mod server;

pub use crm::{CrmManager, CustomerRecord};
pub use drm::{DrmGuard, DrmInfo, LicenseStatus, MachineFingerprint};
pub use engine::{Citation, EngineStats, IngestResult, LocalRagEngine, QueryResult};
pub use server::build_router;
