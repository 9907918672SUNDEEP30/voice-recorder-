//! # Universal State-Machine Orchestrator (The Master Runner)
//!
//! A blind execution engine that executes arbitrary steps sequentially, enforcing
//! explicit state transitions and recording millisecond-level telemetry.

use crate::engine::context::ExecutionContext;
use crate::engine::state::{transition, IllegalTransition, State};
use crate::engine::step::{Step, StepOutcome};
use crate::telemetry::event::LifecycleEvent;
use crate::telemetry::sink::TelemetrySink;
use crate::telemetry::timer::MonotonicClock;

/// Errors emitted by the orchestrator during run execution.
#[derive(Debug, PartialEq, Eq)]
pub enum OrchestratorError {
    /// A step experienced a fatal, unrecoverable error.
    StepFailed {
        step_id: String,
        reason: String,
    },
    /// A step failed and exhausted all allowed retry attempts.
    MaxRetriesExceeded {
        step_id: String,
        attempts: u32,
        last_error: String,
    },
    /// An illegal state transition was attempted.
    InvalidTransition(IllegalTransition),
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StepFailed { step_id, reason } => {
                write!(f, "step '{}' failed fatally: {}", step_id, reason)
            }
            Self::MaxRetriesExceeded {
                step_id,
                attempts,
                last_error,
            } => {
                write!(
                    f,
                    "step '{}' exceeded max retries ({} attempts); last error: {}",
                    step_id, attempts, last_error
                )
            }
            Self::InvalidTransition(e) => write!(f, "state error: {}", e),
        }
    }
}

impl std::error::Error for OrchestratorError {}

impl From<IllegalTransition> for OrchestratorError {
    fn from(err: IllegalTransition) -> Self {
        Self::InvalidTransition(err)
    }
}

/// The Universal Orchestrator.
pub struct Orchestrator<T: TelemetrySink> {
    telemetry: T,
}

impl<T: TelemetrySink> Orchestrator<T> {
    /// Creates a new orchestrator wired to a telemetry sink.
    pub fn new(telemetry: T) -> Self {
        Self { telemetry }
    }

    /// Access mutable reference to telemetry sink.
    pub fn telemetry_mut(&mut self) -> &mut T {
        &mut self.telemetry
    }

    /// Blind sequential execution loop over a collection of Steps.
    pub fn run(
        &mut self,
        ctx: &mut ExecutionContext,
        steps: &mut [Box<dyn Step>],
    ) -> Result<State, OrchestratorError> {
        let run_clock = MonotonicClock::start();
        let mut run_state = State::Start;

        self.telemetry.record(LifecycleEvent::RunStarted {
            run_id: &ctx.run_id,
        });

        run_state = transition(run_state, State::Running)?;

        for step in steps.iter_mut() {
            let step_id = step.id().to_string();
            let max_retries = step.max_retries();
            let max_attempts = 1 + max_retries;

            let mut attempt = 0;
            let mut step_state = State::Start;

            loop {
                attempt += 1;
                let step_clock = MonotonicClock::start();

                self.telemetry.record(LifecycleEvent::StepStarted {
                    run_id: &ctx.run_id,
                    step_id: &step_id,
                    attempt,
                    max_attempts,
                });

                step_state = transition(step_state, State::Running)?;

                let outcome = step.execute(ctx);
                let elapsed_ms = step_clock.elapsed_ms();

                match outcome {
                    StepOutcome::Success => {
                        let _ = transition(step_state, State::Success)?;
                        self.telemetry.record(LifecycleEvent::StepTransition {
                            run_id: &ctx.run_id,
                            step_id: &step_id,
                            from: State::Running,
                            to: State::Success,
                            attempt,
                            elapsed_step_ms: elapsed_ms,
                            message: None,
                        });
                        break;
                    }
                    StepOutcome::Retry(reason) => {
                        if attempt < max_attempts {
                            step_state = transition(step_state, State::Retry)?;
                            self.telemetry.record(LifecycleEvent::StepTransition {
                                run_id: &ctx.run_id,
                                step_id: &step_id,
                                from: State::Running,
                                to: State::Retry,
                                attempt,
                                elapsed_step_ms: elapsed_ms,
                                message: Some(&reason),
                            });
                            // Next iteration in loop will transition Retry -> Running
                            continue;
                        } else {
                            let _ = transition(step_state, State::Failed)?;
                            self.telemetry.record(LifecycleEvent::StepTransition {
                                run_id: &ctx.run_id,
                                step_id: &step_id,
                                from: State::Running,
                                to: State::Failed,
                                attempt,
                                elapsed_step_ms: elapsed_ms,
                                message: Some(&reason),
                            });

                            let _ = transition(run_state, State::Failed);
                            self.telemetry.record(LifecycleEvent::RunFinished {
                                run_id: &ctx.run_id,
                                final_state: State::Failed,
                                total_duration_ms: run_clock.elapsed_ms(),
                                error: Some(&reason),
                            });

                            return Err(OrchestratorError::MaxRetriesExceeded {
                                step_id,
                                attempts: attempt,
                                last_error: reason,
                            });
                        }
                    }
                    StepOutcome::Fatal(reason) => {
                        let _ = transition(step_state, State::Failed)?;
                        self.telemetry.record(LifecycleEvent::StepTransition {
                            run_id: &ctx.run_id,
                            step_id: &step_id,
                            from: State::Running,
                            to: State::Failed,
                            attempt,
                            elapsed_step_ms: elapsed_ms,
                            message: Some(&reason),
                        });

                        let _ = transition(run_state, State::Failed);
                        self.telemetry.record(LifecycleEvent::RunFinished {
                            run_id: &ctx.run_id,
                            final_state: State::Failed,
                            total_duration_ms: run_clock.elapsed_ms(),
                            error: Some(&reason),
                        });

                        return Err(OrchestratorError::StepFailed {
                            step_id,
                            reason,
                        });
                    }
                }
            }
        }

        run_state = transition(run_state, State::Success)?;
        self.telemetry.record(LifecycleEvent::RunFinished {
            run_id: &ctx.run_id,
            final_state: State::Success,
            total_duration_ms: run_clock.elapsed_ms(),
            error: None,
        });

        Ok(run_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::step::FnStep;
    use crate::telemetry::sink::NoopSink;

    #[test]
    fn executes_steps_sequentially_to_success() {
        let mut orch = Orchestrator::new(NoopSink);
        let mut ctx = ExecutionContext::new("test_run_success");

        let step1 = Box::new(FnStep::new("step1", |ctx: &mut ExecutionContext| {
            ctx.set("step1", "done");
            StepOutcome::Success
        }));

        let step2 = Box::new(FnStep::new("step2", |ctx: &mut ExecutionContext| {
            assert_eq!(ctx.get("step1"), Some("done"));
            ctx.set("step2", "done");
            StepOutcome::Success
        }));

        let mut steps: Vec<Box<dyn Step>> = vec![step1, step2];
        let result = orch.run(&mut ctx, &mut steps);

        assert_eq!(result, Ok(State::Success));
        assert_eq!(ctx.get("step2"), Some("done"));
    }

    #[test]
    fn handles_retry_flow_and_recovers() {
        let mut orch = Orchestrator::new(NoopSink);
        let mut ctx = ExecutionContext::new("test_run_retry");

        let mut attempt_counter = 0;
        let step = Box::new(
            FnStep::new("flaky_step", move |_ctx: &mut ExecutionContext| {
                attempt_counter += 1;
                if attempt_counter < 3 {
                    StepOutcome::Retry(format!("temporary failure #{}", attempt_counter))
                } else {
                    StepOutcome::Success
                }
            })
            .with_retries(3),
        );

        let mut steps: Vec<Box<dyn Step>> = vec![step];
        let result = orch.run(&mut ctx, &mut steps);

        assert_eq!(result, Ok(State::Success));
    }

    #[test]
    fn fails_when_retries_exhausted() {
        let mut orch = Orchestrator::new(NoopSink);
        let mut ctx = ExecutionContext::new("test_run_fail");

        let step = Box::new(
            FnStep::new("always_fail", |_ctx: &mut ExecutionContext| {
                StepOutcome::Retry("permanent disk error".to_string())
            })
            .with_retries(1),
        );

        let mut steps: Vec<Box<dyn Step>> = vec![step];
        let result = orch.run(&mut ctx, &mut steps);

        match result {
            Err(OrchestratorError::MaxRetriesExceeded {
                step_id,
                attempts,
                last_error,
            }) => {
                assert_eq!(step_id, "always_fail");
                assert_eq!(attempts, 2);
                assert_eq!(last_error, "permanent disk error");
            }
            other => panic!("expected MaxRetriesExceeded, got: {:?}", other),
        }
    }

    #[test]
    fn aborts_immediately_on_fatal_error() {
        let mut orch = Orchestrator::new(NoopSink);
        let mut ctx = ExecutionContext::new("test_run_fatal");

        let step1 = Box::new(FnStep::new("step1", |_ctx: &mut ExecutionContext| {
            StepOutcome::Fatal("critical safety stop".to_string())
        }));

        let step2 = Box::new(FnStep::new("step2", |_ctx: &mut ExecutionContext| {
            panic!("step2 should never be called after fatal error");
        }));

        let mut steps: Vec<Box<dyn Step>> = vec![step1, step2];
        let result = orch.run(&mut ctx, &mut steps);

        assert!(matches!(result, Err(OrchestratorError::StepFailed { .. })));
    }
}
