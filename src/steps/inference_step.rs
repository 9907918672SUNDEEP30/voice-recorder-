//! # Generic ML Inference Step
//!
//! Bridges crates/core's agnostic State-Machine Orchestrator with the MLBridge inference runner
//! while strictly preserving the Pure Factory Principle.

use ai_core::engine::context::ExecutionContext;
use ai_core::engine::step::{Step, StepOutcome};
use crate::ml_bridge::MLBridge;

/// A universal step executing ML inference via MLBridge against an ExecutionContext.
pub struct GenericMlStep<F>
where
    F: FnMut(&MLBridge, &mut ExecutionContext) -> Result<(), String> + Send,
{
    id: String,
    bridge: MLBridge,
    retries: u32,
    action: F,
}

impl<F> GenericMlStep<F>
where
    F: FnMut(&MLBridge, &mut ExecutionContext) -> Result<(), String> + Send,
{
    pub fn new(id: impl Into<String>, bridge: MLBridge, action: F) -> Self {
        Self {
            id: id.into(),
            bridge,
            retries: 0,
            action,
        }
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }
}

impl<F> Step for GenericMlStep<F>
where
    F: FnMut(&MLBridge, &mut ExecutionContext) -> Result<(), String> + Send,
{
    fn id(&self) -> &str {
        &self.id
    }

    fn max_retries(&self) -> u32 {
        self.retries
    }

    fn execute(&mut self, ctx: &mut ExecutionContext) -> StepOutcome {
        match (self.action)(&self.bridge, ctx) {
            Ok(()) => StepOutcome::Success,
            Err(err_msg) => StepOutcome::Retry(err_msg),
        }
    }
}
