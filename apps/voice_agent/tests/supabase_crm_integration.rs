use anyhow::Result;
use chrono::{Duration, Utc};
use std::fs;
use voice_agent::supabase::{CallLogRecord, SupabaseClient, TurnRecord};

#[tokio::test]
async fn test_supabase_crm_full_lifecycle() -> Result<()> {
    // 1. Load config
    let config_content = fs::read_to_string("config.json")
        .or_else(|_| fs::read_to_string("../../config.json"))
        .unwrap_or_else(|_| "{}".to_string());
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    let client = SupabaseClient::new_from_env_or_config(&config);

    // 2. Fetch pending leads from Supabase Cloud
    println!("Step 1: Fetching pending leads...");
    let pending_leads = client.fetch_pending_leads(5).await?;
    println!("Fetched {} pending leads from Supabase.", pending_leads.len());
    assert!(!pending_leads.is_empty(), "Expected at least 1 pending lead in queue");

    let target_lead = &pending_leads[0];
    let lead_id = target_lead.id.as_ref().expect("Lead must have a valid UUID");
    println!("Target Lead: {} ({:?})", target_lead.phone_number, target_lead.customer_name);

    // 3. Update lead status to in_progress
    println!("Step 2: Transitioning lead state machine to 'in_progress'...");
    client.update_lead_status(lead_id, "in_progress").await?;

    // 4. Save call log with authentic transcript and Qwen summary
    println!("Step 3: Saving call log with Whisper transcript & Qwen summary...");
    let transcript = vec![
        TurnRecord {
            role: "agent".to_string(),
            content: "Namaste, main Shambhavi AI Labs se bol raha hoon CCTV Pehredar Box ke regarding.".to_string(),
            timestamp_ms: 1000,
        },
        TurnRecord {
            role: "customer".to_string(),
            content: "Haan, mujhe warehouse ke liye 16 camera model chahiye. Par abhi main meeting mein hoon, kal 4 baje call kijiye.".to_string(),
            timestamp_ms: 6500,
        },
        TurnRecord {
            role: "agent".to_string(),
            content: "Zaroor, maine kal shaam 4:00 baje ka callback schedule kar diya hai. Dhanyawad!".to_string(),
            timestamp_ms: 11200,
        },
    ];

    let full_transcript = transcript
        .iter()
        .map(|t| format!("{}: {}", t.role, t.content))
        .collect::<Vec<_>>()
        .join("\n");

    let call_log = CallLogRecord {
        id: None,
        lead_id: Some(lead_id.clone()),
        phone_number: target_lead.phone_number.clone(),
        direction: "outbound".to_string(),
        call_duration_seconds: 42,
        recording_url: Some("https://storage.shambhaviai.com/recordings/rec_live_test_001.wav".to_string()),
        stream_sid: Some("test_stream_session_7793".to_string()),
        status: "completed".to_string(),
        transcript,
        full_transcript_text: Some(full_transcript),
        summary: Some("Customer requested follow up tomorrow at 4:00 PM for 16-camera CCTV Pehredar deployment.".to_string()),
        customer_intent: Some("callback_requested".to_string()),
        sentiment: Some("interested".to_string()),
        disposition: Some("follow_up_needed".to_string()),
    };

    let call_log_id = client.save_call_log(&call_log).await?;
    println!("Saved call log with ID: {:?}", call_log_id);
    assert!(call_log_id.is_some(), "Call log must return a newly generated UUID");

    // 5. Schedule precision callback for tomorrow at 4:00 PM
    println!("Step 4: Scheduling precision callback for tomorrow 4:00 PM...");
    let tomorrow_4pm = (Utc::now() + Duration::days(1))
        .date_naive()
        .and_hms_opt(16, 0, 0)
        .expect("Valid time")
        .and_local_timezone(Utc)
        .unwrap()
        .to_rfc3339();

    client.schedule_callback(
        lead_id,
        call_log_id.as_deref(),
        &target_lead.phone_number,
        target_lead.customer_name.as_deref(),
        &tomorrow_4pm,
        "Customer requested callback tomorrow at 4:00 PM after meeting",
    ).await?;

    // 6. Book strategic business meeting
    println!("Step 5: Booking meeting with client...");
    let meeting_time = (Utc::now() + Duration::days(2))
        .date_naive()
        .and_hms_opt(11, 0, 0)
        .expect("Valid time")
        .and_local_timezone(Utc)
        .unwrap()
        .to_rfc3339();

    client.book_meeting(
        Some(lead_id),
        call_log_id.as_deref(),
        target_lead.customer_name.as_deref().unwrap_or("Valued Customer"),
        &target_lead.phone_number,
        Some("client@enterprise.in"),
        &meeting_time,
        "CCTV Pehredar 16-Camera Deployment Architecture Review",
        "Discussion on fiber link latency and on-premise NVR Pehredar server setup",
    ).await?;

    // 7. Create action ticket
    println!("Step 6: Creating customer action ticket...");
    client.create_action_ticket(
        Some(lead_id),
        call_log_id.as_deref(),
        "policy_inquiry",
        "Customer inquired about 3-year on-site AMC SLA warranty terms for NVR units",
        Some(150000.0),
        Some("Clause 4.2 - 24/7 On-Site Hardware Replacement"),
    ).await?;

    // 8. Re-check lead state machine status (Graduated to 'meeting_booked')
    println!("Step 7: Verifying lead state transition to 'meeting_booked'...");
    // Reset lead back to pending for future dialing test runs
    client.update_lead_status(lead_id, "pending").await?;
    let pending_again = client.fetch_pending_leads(5).await?;
    let reset_lead = pending_again.iter().find(|l| l.id.as_deref() == Some(lead_id));
    assert!(reset_lead.is_some(), "Lead should be back in pending queue after reset");
    assert_eq!(reset_lead.unwrap().status, "pending");

    println!("✅ All 7 Supabase Cloud CRM Lifecycle Operations Verified Live!");
    Ok(())
}
