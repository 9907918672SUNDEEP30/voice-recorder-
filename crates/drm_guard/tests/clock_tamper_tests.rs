use drm_guard::MonotonicClockGuard;

#[test]
fn test_clock_normal_progress() {
    let guard = MonotonicClockGuard::new(300);
    assert!(guard.check_integrity());
}

#[test]
fn test_clock_tamper_rollback_detection() {
    let guard = MonotonicClockGuard::new(10); // Strict 10-second tolerance
    assert!(guard.check_integrity());

    // Artificially simulate a future checkpoint
    let future_ts = chrono::Utc::now().timestamp() + 5000;
    guard.update_baseline(future_ts);

    // Current wall-clock will now be severely behind the baseline checkpoint
    assert!(!guard.check_integrity());
}
