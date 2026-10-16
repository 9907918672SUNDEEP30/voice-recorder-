//! # Local RAG Engine Integration Tests

use local_rag_engine::{
    CrmManager, CustomerRecord, DrmGuard, LicenseStatus, LocalRagEngine, MachineFingerprint,
};
use std::sync::Arc;

#[tokio::test]
async fn test_machine_fingerprint_generation() {
    let fp1 = MachineFingerprint::generate();
    let fp2 = MachineFingerprint::generate();

    assert!(!fp1.machine_uuid.is_empty());
    assert_eq!(fp1.machine_uuid.len(), 64); // SHA-256 hex string
    // Deterministic on same host
    assert_eq!(fp1.machine_uuid, fp2.machine_uuid);
}

#[tokio::test]
async fn test_drm_guard_autonomous_lease_and_zeroization() {
    let drm = DrmGuard::new(
        "test_client".to_string(),
        "TEST-LICENSE-KEY".to_string(),
        None, // Autonomous mode
    );

    // Initial state is unlicensed
    assert!(!drm.is_active().await);

    // Perform heartbeat
    let res = drm.perform_heartbeat().await;
    assert!(res.is_ok());

    // License is active and has valid SessionKey
    assert!(drm.is_active().await);
    let key = drm.get_session_key().await;
    assert!(key.is_some());

    // Test zeroization
    drm.invalidate_and_zeroize(LicenseStatus::Expired).await;
    assert!(!drm.is_active().await);
    assert!(drm.get_session_key().await.is_none());
}

#[tokio::test]
async fn test_crm_manager_operations() {
    let crm = CrmManager::new();
    let customers = crm.list_customers().await;
    assert!(customers.len() >= 2);

    let new_cust = CustomerRecord {
        id: "CUST-TEST".to_string(),
        name: "Test User".to_string(),
        company: "Test Corp".to_string(),
        email: "test@corp.com".to_string(),
        phone: "123456789".to_string(),
        deal_value: 500000.0,
        tags: vec!["Test".to_string()],
        associated_docs: vec![],
        notes: vec![],
        created_at: "".to_string(),
        updated_at: "".to_string(),
    };

    crm.upsert_customer(new_cust).await;
    let fetched = crm.get_customer("CUST-TEST").await;
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().company, "Test Corp");

    crm.associate_document("CUST-TEST", "DOC-TEST-001").await;
    let updated = crm.get_customer("CUST-TEST").await.unwrap();
    assert!(updated.associated_docs.contains(&"DOC-TEST-001".to_string()));
}

#[tokio::test]
async fn test_local_rag_engine_ingest_and_query() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_path_buf();

    let drm = Arc::new(DrmGuard::new(
        "sme_client_001".to_string(),
        "DEMO-KEY".to_string(),
        None,
    ));
    drm.perform_heartbeat().await.unwrap();

    let crm = CrmManager::new();
    let engine = LocalRagEngine::new(&db_path, None, drm, crm).unwrap();

    // Ingest plain text business document
    let text = "Invoice INV-2026-88: Total payable amount is 450,000 INR for Q1 enterprise maintenance services. Due date is March 31, 2026.";
    let ingest_res = engine
        .ingest_text("INV-2026-88", text, "Invoice", Some("CUST-001"))
        .await
        .unwrap();

    assert_eq!(ingest_res.doc_id, "INV-2026-88");
    assert!(ingest_res.chunks_created >= 1);

    // Query RAG
    let query_res = engine.query("What is the invoice amount for Q1?", 3).await.unwrap();
    assert!(!query_res.answer.is_empty());
    assert!(!query_res.citations.is_empty());
    assert_eq!(query_res.citations[0].doc_id, "INV-2026-88");
}
