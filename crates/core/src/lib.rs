//! # AI Agent Core Engine
//!
//! High-performance, application-agnostic, zero-bloat agent runtime core.
//! Built strictly on the Rust standard library.

pub mod engine;
pub mod ipc;
pub mod telemetry;
pub mod models;

// Convenience re-exports
pub use engine::{
    transition, ExecutionContext, FnStep, IllegalTransition, Orchestrator, OrchestratorError,
    State, Step, StepOutcome,
};
pub use ipc::{IpcBridge, InboundCommand, OutboundMessage};
pub use telemetry::{LifecycleEvent, MillisecondLogger, MonotonicClock, NoopSink, TelemetrySink};
