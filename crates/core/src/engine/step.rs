//! # Generic Step Abstractions
//!
//! Defines the universal interface for any unit of execution within the engine.
//! The engine treats every step as an opaque black box.

use crate::engine::context::ExecutionContext;

/// The outcome of executing a single step attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    /// Step finished successfully, engine can advance to the next step.
    Success,
    /// Step encountered a transient/recoverable failure; retry if policy allows.
    Retry(String),
    /// Step encountered an unrecoverable failure; terminate run immediately.
    Fatal(String),
}

/// The universal contract every executable step must implement.
pub trait Step: Send {
    /// Unique identifier for telemetry and step tracking.
    fn id(&self) -> &str;

    /// Maximum number of retries allowed for this step upon recoverable failure.
    /// Defaults to 0 (single attempt).
    fn max_retries(&self) -> u32 {
        0
    }

    /// Executes the step blindly against the runtime execution context.
    fn execute(&mut self, ctx: &mut ExecutionContext) -> StepOutcome;
}

/// A lightweight helper to construct a step from a closure.
pub struct FnStep<F>
where
    F: FnMut(&mut ExecutionContext) -> StepOutcome + Send,
{
    id: String,
    retries: u32,
    func: F,
}

impl<F> FnStep<F>
where
    F: FnMut(&mut ExecutionContext) -> StepOutcome + Send,
{
    pub fn new(id: impl Into<String>, func: F) -> Self {
        Self {
            id: id.into(),
            retries: 0,
            func,
        }
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }
}

impl<F> Step for FnStep<F>
where
    F: FnMut(&mut ExecutionContext) -> StepOutcome + Send,
{
    fn id(&self) -> &str {
        &self.id
    }

    fn max_retries(&self) -> u32 {
        self.retries
    }

    fn execute(&mut self, ctx: &mut ExecutionContext) -> StepOutcome {
        (self.func)(ctx)
    }
}
