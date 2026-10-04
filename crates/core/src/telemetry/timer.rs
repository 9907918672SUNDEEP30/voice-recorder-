//! # Monotonic Millisecond Timer
//!
//! Zero-bloat high-precision timer built strictly on Rust standard library primitives.

use std::time::Instant;

/// Monotonic clock for calculating millisecond deltas from an anchor point.
#[derive(Debug, Clone)]
pub struct MonotonicClock {
    epoch: Instant,
}

impl MonotonicClock {
    /// Starts a new monotonic clock anchored at current instant.
    pub fn start() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }

    /// Elapsed time in milliseconds since the clock was started.
    #[inline]
    pub fn elapsed_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    /// Returns the raw anchor instant.
    #[inline]
    pub fn epoch(&self) -> Instant {
        self.epoch
    }
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self::start()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn timer_tracks_elapsed_milliseconds() {
        let timer = MonotonicClock::start();
        sleep(Duration::from_millis(5));
        assert!(timer.elapsed_ms() >= 4);
    }
}
