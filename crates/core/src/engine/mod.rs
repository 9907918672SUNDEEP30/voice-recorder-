//! # Universal State-Machine Orchestrator
//!
//! Blind, application-agnostic sequential runner with explicit state verification.

pub mod context;
pub mod runner;
pub mod state;
pub mod step;

pub use context::ExecutionContext;
pub use runner::{Orchestrator, OrchestratorError};
pub use state::{transition, IllegalTransition, State};
pub use step::{FnStep, Step, StepOutcome};
