use drm_guard::MachineFingerprint;

#[test]
fn test_hardware_fingerprint_determinism() {
    let fp1 = MachineFingerprint::generate();
    let fp2 = MachineFingerprint::generate();

    assert!(!fp1.machine_uuid.is_empty());
    assert_eq!(fp1.machine_uuid.len(), 64); // SHA-256 hex string
    assert_eq!(fp1.machine_uuid, fp2.machine_uuid);
    assert!(fp1.matches(&fp2.machine_uuid));
}
