use drm_guard::OtaUpdater;
use std::io::Write;

#[test]
fn test_semver_comparison() {
    assert_eq!(OtaUpdater::compare_semver("1.2.0", "1.1.0"), 1);
    assert_eq!(OtaUpdater::compare_semver("v2.0.0", "1.9.9"), 1);
    assert_eq!(OtaUpdater::compare_semver("1.0.0", "1.0.0"), 0);
    assert_eq!(OtaUpdater::compare_semver("0.9.5", "1.0.0"), -1);
    assert_eq!(OtaUpdater::compare_semver("1.0.1", "1.0.0"), 1);

    let updater = OtaUpdater::new("1.0.0");
    assert!(updater.is_newer_version("1.0.1"));
    assert!(updater.is_newer_version("1.2.0"));
    assert!(!updater.is_newer_version("1.0.0"));
    assert!(!updater.is_newer_version("0.9.9"));
}

#[test]
fn test_sha256_computation_and_integrity() {
    let binary_bytes = b"SIMULATED_PROD_BINARY_EXECUTABLE_CONTENT_V2";
    let computed_hash = OtaUpdater::compute_sha256(binary_bytes);

    assert!(!computed_hash.is_empty());
    assert_eq!(computed_hash.len(), 64);

    // Matching hash passes
    assert!(OtaUpdater::verify_integrity(binary_bytes, &computed_hash));

    // Tampered payload fails
    let tampered_bytes = b"TAMPERED_MALICIOUS_PAYLOAD";
    assert!(!OtaUpdater::verify_integrity(tampered_bytes, &computed_hash));

    // Wrong hash string fails
    assert!(!OtaUpdater::verify_integrity(
        binary_bytes,
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));
}

#[test]
fn test_atomic_binary_swap() {
    let temp_dir = tempfile::tempdir().unwrap();
    let target_exe = temp_dir.path().join("cctv_guard.exe");

    // 1. Create simulated original running executable v1
    let original_payload = b"ORIGINAL_RUNNING_BINARY_V1";
    {
        let mut file = std::fs::File::create(&target_exe).unwrap();
        file.write_all(original_payload).unwrap();
    }
    assert_eq!(std::fs::read(&target_exe).unwrap(), original_payload);

    // 2. Perform atomic binary swap with new v2 binary
    let new_payload = b"UPGRADED_VERIFIED_BINARY_V2";
    let updater = OtaUpdater::new("1.0.0");
    let swapped_path = updater.apply_binary_swap(new_payload, Some(&target_exe)).unwrap();

    assert_eq!(swapped_path, target_exe);
    assert_eq!(std::fs::read(&target_exe).unwrap(), new_payload);

    // 3. Verify backup file was created
    #[cfg(target_os = "windows")]
    {
        let backup_path = temp_dir.path().join("cctv_guard.exe.old");
        assert!(backup_path.exists());
        assert_eq!(std::fs::read(&backup_path).unwrap(), original_payload);
    }
}
