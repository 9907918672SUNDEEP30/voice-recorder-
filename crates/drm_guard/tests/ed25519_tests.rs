use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

#[test]
fn test_ed25519_key_derivation_and_signature_verification() {
    let seed = b"SHAMBHAVI_AI_LABS_ROOT_SEED_2026";
    assert_eq!(seed.len(), 32);

    let signing_key = SigningKey::from_bytes(seed);
    let verifying_key = signing_key.verifying_key();
    let pubkey_bytes = verifying_key.to_bytes();
    let pubkey_hex = hex::encode(pubkey_bytes);

    println!("Ed25519 Root Public Key Hex: {}", pubkey_hex);

    // Canonical lease message: client_id:machine_uuid:app_name:expires_at:nonce:session_key_hex
    let canonical_msg = "sme_001:uuid_1234:cctv_guard:1776512345:nonce_9876:session_key_hex";
    let signature = signing_key.sign(canonical_msg.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());

    // Verify signature
    let decoded_pub = VerifyingKey::from_bytes(&pubkey_bytes).expect("valid pubkey");
    let decoded_sig_bytes = hex::decode(&sig_hex).expect("valid hex");
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&decoded_sig_bytes);
    let decoded_sig = Signature::from_bytes(&sig_arr);

    assert!(decoded_pub.verify(canonical_msg.as_bytes(), &decoded_sig).is_ok());

    // Tampered message must fail
    let tampered_msg = "sme_001:uuid_1234:cctv_guard:9999999999:nonce_9876:session_key_hex";
    assert!(decoded_pub.verify(tampered_msg.as_bytes(), &decoded_sig).is_err());
}
