//! # Telemetry Lifecycle Events
//!
//! Strongly typed representation of execution lifecycle events across run and step levels.

use crate::engine::state::State;

/// Telemetry event emitted during orchestrator execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleEvent<'a> {
    /// The entire execution run has been initiated.
    RunStarted {
        run_id: &'a str,
    },
    /// A step attempt has begun.
    StepStarted {
        run_id: &'a str,
        step_id: &'a str,
        attempt: u32,
        max_attempts: u32,
    },
    /// A step attempt has transitioned to an explicit state.
    StepTransition {
        run_id: &'a str,
        step_id: &'a str,
        from: State,
        to: State,
        attempt: u32,
        elapsed_step_ms: u64,
        message: Option<&'a str>,
    },
    /// The execution run has reached a terminal state (Success or Failed).
    RunFinished {
        run_id: &'a str,
        final_state: State,
        total_duration_ms: u64,
        error: Option<&'a str>,
    },
}
