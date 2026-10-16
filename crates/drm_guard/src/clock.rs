//! # Anti-Clock-Tampering Guard Module
//!
//! Enforces monotonic time tracking to prevent system clock rollbacks
//! and offline license backdating attacks.

use chrono::Utc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct MonotonicClockGuard {
    start_instant: Instant,
    start_utc_ts: i64,
    last_monotonic_ts: Arc<AtomicI64>,
    max_drift_seconds: i64,
}

impl Default for MonotonicClockGuard {
    fn default() -> Self {
        Self::new(300) // Default 5 minutes tolerance
    }
}

impl MonotonicClockGuard {
    /// Creates a new monotonic clock guard with a tolerance threshold in seconds.
    pub fn new(max_drift_seconds: i64) -> Self {
        let now_utc = Utc::now().timestamp();
        Self {
            start_instant: Instant::now(),
            start_utc_ts: now_utc,
            last_monotonic_ts: Arc::new(AtomicI64::new(now_utc)),
            max_drift_seconds,
        }
    }

    /// Verifies system clock integrity.
    /// Returns `true` if clock is sound, or `false` if clock rollback/tampering is detected.
    pub fn check_integrity(&self) -> bool {
        let current_wall_clock = Utc::now().timestamp();
        let last_recorded = self.last_monotonic_ts.load(Ordering::SeqCst);

        // 1. Check if wall-clock moved backwards past the allowed tolerance window
        if current_wall_clock < last_recorded - self.max_drift_seconds {
            eprintln!(
                "[DRM SECURITY] Clock rollback detected! Wall-clock ({}) is behind last monotonic ({}) by > {}s.",
                current_wall_clock, last_recorded, self.max_drift_seconds
            );
            return false;
        }

        // 2. Cross-verify against CPU monotonic Instant elapsed
        let monotonic_elapsed_secs = self.start_instant.elapsed().as_secs() as i64;
        let expected_min_utc = self.start_utc_ts + monotonic_elapsed_secs - self.max_drift_seconds;

        if current_wall_clock < expected_min_utc {
            eprintln!(
                "[DRM SECURITY] Clock desynchronization detected! Wall-clock ({}) violates CPU monotonic progress (expected >= {}).",
                current_wall_clock, expected_min_utc
            );
            return false;
        }

        // Update last verified timestamp monotonically
        if current_wall_clock > last_recorded {
            self.last_monotonic_ts.store(current_wall_clock, Ordering::SeqCst);
        }

        true
    }

    /// Forces a manual update of the monotonic baseline (useful for synchronized NTP updates)
    pub fn update_baseline(&self, new_ts: i64) {
        self.last_monotonic_ts.store(new_ts, Ordering::SeqCst);
    }
}
