//! # CDM Engine Library
//!
//! Sandeep Ji's Deterministic Context-Data Matrix (CDM) Engine.
//! 100% Zero-RAG • Zero Vectors • 2-Stage Deterministic Stream & Vision • 25-Year Lifespan
//! Sovereign Enterprise Security & Anti-Tamper Protected.

pub mod audit;
pub mod automata_parser;
pub mod query_engine;
pub mod schema;
pub mod security;
pub mod vision_extractor;
pub mod watcher;
pub mod workspace;

pub use audit::{AuditAction, AuditLogger};
pub use automata_parser::{DeterministicAutomataParser, ExtractedFinancialData};
pub use query_engine::{CdmQueryEngine, DeterministicQueryResult};
pub use schema::initialize_schema;
pub use security::SecurityManager;
pub use vision_extractor::VisionExtractor;
pub use watcher::SourceDocsWatcher;
pub use workspace::ClientWorkspace;
