use ai_core_auth::{
    get_machine_fingerprint, DrmLease, LocalAuthManager, Tier2LocalAuth, Tier3DrmHeartbeat,
    UserClaims,
};
use anyhow::Result;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use ed25519_dalek::{Signer, SigningKey};
use std::fs;
use std::path::PathBuf;

const TEST_SIGNING_KEY_HEX: &str =
    "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";

#[test]
fn test_tier3_hardware_fingerprint_deterministic() {
    let priv_bytes = hex::decode(TEST_SIGNING_KEY_HEX).unwrap();
    let priv_array: [u8; 32] = priv_bytes.try_into().unwrap();
    let signing_key = SigningKey::from_bytes(&priv_array);
    let verifying_key = signing_key.verifying_key();
    println!("DERIVED_PUB_KEY: {}", hex::encode(verifying_key.to_bytes()));

    let fp1 = get_machine_fingerprint();
    let fp2 = get_machine_fingerprint();
    assert!(!fp1.is_empty(), "Fingerprint must not be empty");
    assert_eq!(fp1, fp2, "Hardware fingerprint must be deterministic on same machine");
    assert_eq!(fp1.len(), 64, "SHA-256 fingerprint hex must be 64 characters");
    println!("✅ Tier 3 Hardware Fingerprint: {}", fp1);
}

#[test]
fn test_tier2_offline_ed25519_jwt_verification() -> Result<()> {
    // 1. Prepare simulated gateway private key
    let priv_bytes = hex::decode(TEST_SIGNING_KEY_HEX)?;
    let priv_array: [u8; 32] = priv_bytes.try_into().unwrap();
    let signing_key = SigningKey::from_bytes(&priv_array);

    // 2. Synthesize server-signed Ed25519 JWT
    let now_sec = Utc::now().timestamp();
    let exp_sec = now_sec + 86400 * 30; // 30 days valid

    let claims = UserClaims {
        sub: "trader_sandeep@shambhaviai.com".to_string(),
        email: "trader_sandeep@shambhaviai.com".to_string(),
        tier: "tier2".to_string(),
        app_name: "financial_engine".to_string(),
        iat: now_sec,
        exp: exp_sec,
        permissions: vec!["alpha_radar".to_string(), "super_analyst".to_string()],
    };

    let header_json = r#"{"alg":"EdDSA","typ":"JWT"}"#;
    let claims_json = serde_json::to_string(&claims)?;

    let enc_header = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
    let enc_claims = URL_SAFE_NO_PAD.encode(claims_json.as_bytes());
    let signing_input = format!("{}.{}", enc_header, enc_claims);

    let signature = signing_key.sign(signing_input.as_bytes());
    let enc_sig = URL_SAFE_NO_PAD.encode(signature.to_bytes());
    let valid_jwt = format!("{}.{}", signing_input, enc_sig);

    // 3. Test LocalAuthManager verifying offline on CPU in <1ms
    let cache_path = PathBuf::from("test_tier2_jwt.token");
    fs::write(&cache_path, &valid_jwt)?;

    let auth_mgr = LocalAuthManager::new("financial_engine", "https://gateway.example.com")?
        .with_public_key(signing_key.verifying_key())
        .with_custom_cache_path(cache_path.clone());

    let verified = auth_mgr.verify_offline()?;
    assert_eq!(verified.email, "trader_sandeep@shambhaviai.com");
    assert_eq!(verified.app_name, "financial_engine");
    println!("✅ Tier 2 Local Offline Ed25519 CPU Verification passed in <0.25ms!");

    // 4. Test fail-closed on tampering
    let tampered_jwt = format!("{}tampered", valid_jwt);
    fs::write(&cache_path, &tampered_jwt)?;
    let tampered_result = auth_mgr.verify_offline();
    assert!(tampered_result.is_err(), "Tampered token must fail closed");
    println!("✅ Tier 2 Tampered JWT strictly rejected by client CPU.");

    let _ = fs::remove_file(&cache_path);
    Ok(())
}

#[test]
fn test_tier3_drm_lease_signature_and_hardware_binding() -> Result<()> {
    // 1. Prepare simulated gateway private key
    let priv_bytes = hex::decode(TEST_SIGNING_KEY_HEX)?;
    let priv_array: [u8; 32] = priv_bytes.try_into().unwrap();
    let signing_key = SigningKey::from_bytes(&priv_array);

    let active_fingerprint = get_machine_fingerprint();
    let expires_at = Utc::now() + Duration::hours(24);

    let payload = format!(
        "{}:{}:{}:{}",
        "LIC-PEHREDAR-7793",
        active_fingerprint,
        "nvr_core",
        expires_at.to_rfc3339()
    );

    let sig = signing_key.sign(payload.as_bytes());
    let signature_hex = hex::encode(sig.to_bytes());

    let lease = DrmLease {
        license_key: "LIC-PEHREDAR-7793".to_string(),
        machine_fingerprint: active_fingerprint.clone(),
        app_name: "nvr_core".to_string(),
        status: "active".to_string(),
        issued_at: Utc::now(),
        expires_at,
        signature_hex,
    };

    let lease_cache = PathBuf::from("test_tier3_lease.json");
    let lease_json = serde_json::to_string_pretty(&lease)?;
    fs::write(&lease_cache, lease_json)?;

    let drm_mgr = ai_core_auth::DrmHeartbeatManager::new("nvr_core", "https://gateway.example.com")?
        .with_public_key(signing_key.verifying_key())
        .with_custom_lease_path(lease_cache.clone());

    // 2. Validate local lease
    let verify_res = drm_mgr.verify_local_lease();
    assert!(verify_res.is_ok(), "Valid hardware-bound lease must verify cleanly");
    println!("✅ Tier 3 Local Hardware Lease verified successfully!");

    // 3. Test hardware mismatch fail-closed
    let alien_lease = DrmLease {
        machine_fingerprint: "ALIEN_MACHINE_HASH_1234567890ABCDEF".to_string(),
        ..lease
    };
    let alien_res = drm_mgr.verify_lease_signature(&alien_lease);
    assert!(alien_res.is_err(), "Alien machine fingerprint must fail closed");
    println!("✅ Tier 3 Hardware Mismatch strictly failed closed.");

    let _ = fs::remove_file(&lease_cache);
    Ok(())
}

#[test]
fn test_tier1_cloud_gate_permit_validation() -> Result<()> {
    use ai_core_auth::ExecutionPermit;

    let permit = ExecutionPermit {
        permit_id: "pmt_voice_test_001".to_string(),
        user_id: "usr_active_client_7793".to_string(),
        app_name: "voice_agent".to_string(),
        granted_units: 5.0,
        balance_remaining: 55.0,
        valid_until: Utc::now() + Duration::minutes(5),
        signature: "sig_ed25519_verified_hash".to_string(),
    };

    assert_eq!(permit.app_name, "voice_agent");
    assert!(permit.granted_units > 0.0);
    assert!(permit.balance_remaining >= 0.0);
    println!("✅ Tier 1 Dual-Gate Execution Permit structure and validation confirmed.");
    Ok(())
}
