//! # CDM Engine Library
//!
//! Sandeep Ji's Deterministic Context-Data Matrix (CDM) Engine.
//! 100% Zero-RAG • Zero Vectors • Single-Model Qwen2-VL-2B • 25-Year Lifespan
//! Sovereign Enterprise Security & Anti-Tamper Protected.

pub mod audit;
pub mod query_engine;
pub mod schema;
pub mod security;
pub mod vision_extractor;
pub mod watcher;
pub mod workspace;

pub use audit::{AuditAction, AuditLogger};
pub use query_engine::{CdmQueryEngine, DeterministicQueryResult};
pub use schema::initialize_schema;
pub use security::SecurityManager;
pub use vision_extractor::VisionExtractor;
pub use watcher::SourceDocsWatcher;
pub use workspace::ClientWorkspace;
