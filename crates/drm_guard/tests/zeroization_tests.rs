use drm_guard::{AesGcmCipher, VolatileSessionKey};

#[test]
fn test_aes_gcm_roundtrip_encryption() {
    let key = VolatileSessionKey::generate_random();
    let secret_payload = b"CRITICAL_SME_FINANCIAL_OR_CCTV_RECORD";

    let (ciphertext, nonce) = AesGcmCipher::encrypt(&key, secret_payload).unwrap();
    assert_ne!(ciphertext, secret_payload);

    let decrypted = AesGcmCipher::decrypt(&key, &ciphertext, &nonce).unwrap();
    assert_eq!(decrypted, secret_payload);
}

#[test]
fn test_wrong_key_fails_decryption() {
    let key1 = VolatileSessionKey::generate_random();
    let key2 = VolatileSessionKey::generate_random();
    let secret_payload = b"LOCAL_RAG_DOCUMENT_RECORD";

    let (ciphertext, nonce) = AesGcmCipher::encrypt(&key1, secret_payload).unwrap();
    let res = AesGcmCipher::decrypt(&key2, &ciphertext, &nonce);
    assert!(res.is_err());
}
