//! # Zero-Bloat Telemetry Engine
//!
//! Standard-library based millisecond precision telemetry and logging.

pub mod event;
pub mod sink;
pub mod timer;

pub use event::LifecycleEvent;
pub use sink::{MillisecondLogger, NoopSink, TelemetrySink};
pub use timer::MonotonicClock;
