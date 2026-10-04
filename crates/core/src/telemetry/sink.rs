//! # Telemetry Sinks
//!
//! Formatted log sinks and collectors for engine telemetry events.
//! Built strictly with `std::io::Write` and standard library formatting.

use std::io::Write;
use crate::telemetry::event::LifecycleEvent;
use crate::telemetry::timer::MonotonicClock;

/// Interface for receiving telemetry lifecycle events.
pub trait TelemetrySink: Send {
    fn record(&mut self, event: LifecycleEvent<'_>);
}

/// Zero-bloat millisecond console/stream logger.
pub struct MillisecondLogger<W: Write + Send> {
    writer: W,
    clock: MonotonicClock,
}

impl<W: Write + Send> MillisecondLogger<W> {
    /// Creates a logger outputting to any `std::io::Write` instance.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            clock: MonotonicClock::start(),
        }
    }

    /// Resets the internal clock anchor to current instant.
    pub fn reset_clock(&mut self) {
        self.clock = MonotonicClock::start();
    }
}

impl MillisecondLogger<std::io::Stdout> {
    /// Constructs a logger writing to `std::io::stdout()`.
    pub fn stdout() -> Self {
        Self::new(std::io::stdout())
    }
}

impl MillisecondLogger<std::io::Stderr> {
    /// Constructs a logger writing to `std::io::stderr()`.
    pub fn stderr() -> Self {
        Self::new(std::io::stderr())
    }
}

impl<W: Write + Send> TelemetrySink for MillisecondLogger<W> {
    fn record(&mut self, event: LifecycleEvent<'_>) {
        let ts = self.clock.elapsed_ms();
        match event {
            LifecycleEvent::RunStarted { run_id } => {
                let _ = writeln!(self.writer, "[+{:06}ms] [run:{}] RUN_STARTED", ts, run_id);
            }
            LifecycleEvent::StepStarted {
                run_id,
                step_id,
                attempt,
                max_attempts,
            } => {
                let _ = writeln!(
                    self.writer,
                    "[+{:06}ms] [run:{}] [step:{}] START -> RUNNING (attempt {}/{})",
                    ts, run_id, step_id, attempt, max_attempts
                );
            }
            LifecycleEvent::StepTransition {
                run_id,
                step_id,
                from,
                to,
                attempt,
                elapsed_step_ms,
                message,
            } => {
                if let Some(msg) = message {
                    let _ = writeln!(
                        self.writer,
                        "[+{:06}ms] [run:{}] [step:{}] {} -> {} (attempt {}, elapsed: {}ms, info: {})",
                        ts,
                        run_id,
                        step_id,
                        from.label().to_uppercase(),
                        to.label().to_uppercase(),
                        attempt,
                        elapsed_step_ms,
                        msg
                    );
                } else {
                    let _ = writeln!(
                        self.writer,
                        "[+{:06}ms] [run:{}] [step:{}] {} -> {} (attempt {}, elapsed: {}ms)",
                        ts,
                        run_id,
                        step_id,
                        from.label().to_uppercase(),
                        to.label().to_uppercase(),
                        attempt,
                        elapsed_step_ms
                    );
                }
            }
            LifecycleEvent::RunFinished {
                run_id,
                final_state,
                total_duration_ms,
                error,
            } => {
                if let Some(err) = error {
                    let _ = writeln!(
                        self.writer,
                        "[+{:06}ms] [run:{}] RUN_FINISHED -> {} (total: {}ms, error: {})",
                        ts,
                        run_id,
                        final_state.label().to_uppercase(),
                        total_duration_ms,
                        err
                    );
                } else {
                    let _ = writeln!(
                        self.writer,
                        "[+{:06}ms] [run:{}] RUN_FINISHED -> {} (total: {}ms)",
                        ts,
                        run_id,
                        final_state.label().to_uppercase(),
                        total_duration_ms
                    );
                }
            }
        }
        let _ = self.writer.flush();
    }
}

/// No-op sink for scenarios where telemetry is disabled or in quiet mode.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopSink;

impl TelemetrySink for NoopSink {
    #[inline(always)]
    fn record(&mut self, _event: LifecycleEvent<'_>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::state::State;

    #[test]
    fn millisecond_logger_formats_correctly() {
        let mut buffer = Vec::new();
        let mut logger = MillisecondLogger::new(&mut buffer);

        logger.record(LifecycleEvent::RunStarted { run_id: "test-run" });
        logger.record(LifecycleEvent::StepStarted {
            run_id: "test-run",
            step_id: "step-1",
            attempt: 1,
            max_attempts: 1,
        });
        logger.record(LifecycleEvent::StepTransition {
            run_id: "test-run",
            step_id: "step-1",
            from: State::Running,
            to: State::Success,
            attempt: 1,
            elapsed_step_ms: 10,
            message: None,
        });
        logger.record(LifecycleEvent::RunFinished {
            run_id: "test-run",
            final_state: State::Success,
            total_duration_ms: 12,
            error: None,
        });

        let output = String::from_utf8(buffer).expect("valid utf8");
        assert!(output.contains("[run:test-run] RUN_STARTED"));
        assert!(output.contains("[step:step-1] START -> RUNNING (attempt 1/1)"));
        assert!(output.contains("[step:step-1] RUNNING -> SUCCESS (attempt 1, elapsed: 10ms)"));
        assert!(output.contains("[run:test-run] RUN_FINISHED -> SUCCESS (total: 12ms)"));
    }
}
