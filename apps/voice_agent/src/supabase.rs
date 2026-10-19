//! # Supabase Enterprise CRM & Autonomous Tele-Dialer Engine
//!
//! Provides cloud persistence, bulk lead queue management (10,000+ numbers),
//! call logging with Whisper STT & Qwen summaries, automated scheduled callbacks,
//! strategic meeting bookings, and autonomous policy/refund action tickets.

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnRecord {
    pub role: String, // "agent" | "customer" | "system"
    pub content: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSessionRecord {
    pub call_sid: String,
    pub stream_sid: String,
    pub caller_number: Option<String>,
    pub receiver_number: Option<String>,
    pub direction: String, // "inbound" | "outbound" | "browser_test"
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub duration_seconds: u64,
    pub status: String, // "completed" | "voicemail_hangup" | "caller_hangup"
    pub transcript: Vec<TurnRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub phone_number: String,
    pub customer_name: Option<String>,
    pub company_name: Option<String>,
    pub custom_context: Option<Value>,
    pub status: String, // "pending", "in_progress", "completed", "scheduled_callback", "meeting_booked", "not_interested"
    pub call_attempts: i32,
    pub max_retries: i32,
    pub scheduled_at: Option<String>,
    pub last_called_at: Option<String>,
    pub priority: i32,
    pub assigned_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallLogRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub lead_id: Option<String>,
    pub phone_number: String,
    pub direction: String,
    pub call_duration_seconds: u64,
    pub recording_url: Option<String>,
    pub stream_sid: Option<String>,
    pub status: String,
    pub transcript: Vec<TurnRecord>,
    pub full_transcript_text: Option<String>,
    pub summary: Option<String>,
    pub customer_intent: Option<String>,
    pub sentiment: Option<String>, // "positive" | "neutral" | "negative" | "interested" | "hostile"
    pub disposition: Option<String>, // "follow_up_needed" | "meeting_set" | "deal_closed" | "closed_lost" | "dnd"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledCallbackRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub lead_id: String,
    pub call_log_id: Option<String>,
    pub phone_number: String,
    pub customer_name: Option<String>,
    pub callback_time: String,
    pub reason: Option<String>,
    pub status: String, // "pending" | "processing" | "completed" | "cancelled"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingBookingRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub lead_id: Option<String>,
    pub call_log_id: Option<String>,
    pub attendee_name: String,
    pub attendee_phone: String,
    pub attendee_email: Option<String>,
    pub meeting_title: String,
    pub meeting_time: String,
    pub meeting_duration_minutes: i32,
    pub google_calendar_event_id: Option<String>,
    pub status: String, // "confirmed" | "rescheduled" | "cancelled"
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTicketRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub lead_id: Option<String>,
    pub call_log_id: Option<String>,
    pub ticket_type: String, // "refund_request", "policy_inquiry", "payment_support", "tech_issue"
    pub description: String,
    pub amount: Option<f64>,
    pub policy_clause_cited: Option<String>,
    pub status: String, // "open", "processing", "resolved", "rejected"
    pub resolution_notes: Option<String>,
}

#[derive(Clone)]
pub struct SupabaseClient {
    url: Option<String>,
    api_key: Option<String>,
    http_client: reqwest::Client,
}

impl SupabaseClient {
    /// Initializes from system environment variables or framework config.json
    pub fn new_from_env_or_config(config: &serde_json::Value) -> Self {
        let is_valid_token = |t: &str| !t.trim().is_empty() && !t.contains("***") && t.len() > 8;

        let env_url = std::env::var("SUPABASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let env_key = std::env::var("SUPABASE_KEY")
            .or_else(|_| std::env::var("SUPABASE_SERVICE_ROLE_KEY"))
            .or_else(|_| std::env::var("SUPABASE_SECRET_KEY"))
            .ok()
            .filter(|s| is_valid_token(s));

        let cfg_url = config["supabase"]["url"]
            .as_str()
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty());
        let cfg_key = config["supabase"]["key"]
            .as_str()
            .map(|s| s.to_string())
            .filter(|s| is_valid_token(s));

        let url = env_url.or(cfg_url);
        let api_key = env_key.or(cfg_key);

        if let (Some(u), Some(_)) = (&url, &api_key) {
            println!("✅ [Supabase] Connected to Supabase Cloud CRM: {}", u);
        } else {
            println!("ℹ️ [Supabase] Running in local audit mode (credentials not configured).");
        }

        Self {
            url,
            api_key,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Fetches a batch of pending leads prioritized by priority (High first) and scheduled time
    pub async fn fetch_pending_leads(&self, batch_size: usize) -> Result<Vec<LeadRecord>> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(Vec::new()),
        };

        let endpoint = format!(
            "{}/rest/v1/leads?status=in.(pending,scheduled_callback)&order=priority.desc,created_at.asc&limit={}",
            url.trim_end_matches('/'),
            batch_size
        );

        let resp = self
            .http_client
            .get(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status().is_success() {
            let leads: Vec<LeadRecord> = resp.json().await?;
            Ok(leads)
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(anyhow!("Failed to fetch pending leads ({}): {}", status, text))
        }
    }

    /// Checks for callbacks that are due for execution right now
    pub async fn fetch_due_callbacks(&self) -> Result<Vec<ScheduledCallbackRecord>> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(Vec::new()),
        };

        let now_iso = Utc::now().to_rfc3339();
        let endpoint = format!(
            "{}/rest/v1/scheduled_callbacks?status=eq.pending&callback_time=lte.{}&order=callback_time.asc&limit=25",
            url.trim_end_matches('/'),
            now_iso
        );

        let resp = self
            .http_client
            .get(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status().is_success() {
            let callbacks: Vec<ScheduledCallbackRecord> = resp.json().await?;
            Ok(callbacks)
        } else {
            Err(anyhow!("Failed to fetch due callbacks: {}", resp.status()))
        }
    }

    /// Updates lead status in state machine (e.g. "calling", "completed", "meeting_booked")
    pub async fn update_lead_status(&self, lead_id: &str, status: &str) -> Result<()> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(()),
        };

        let endpoint = format!("{}/rest/v1/leads?id=eq.{}", url.trim_end_matches('/'), lead_id);
        let now_iso = Utc::now().to_rfc3339();

        let payload = json!({
            "status": status,
            "last_called_at": now_iso,
            "updated_at": now_iso
        });

        let resp = self
            .http_client
            .patch(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&payload)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to update lead status: {}", resp.status()))
        }
    }

    /// Records completed call log with full Whisper STT transcript, Qwen summary, and sentiment
    pub async fn save_call_log(&self, record: &CallLogRecord) -> Result<Option<String>> {
        // 1. Audit write locally to disk
        let safe_sid = record
            .stream_sid
            .as_deref()
            .unwrap_or("unknown_session")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .take(64)
            .collect::<String>();

        let log_dir = Path::new("logs/call_records");
        if let Ok(_) = fs::create_dir_all(log_dir) {
            let log_file = log_dir.join(format!("{}.json", safe_sid));
            if let Ok(json_str) = serde_json::to_string_pretty(record) {
                let _ = fs::write(log_file, json_str);
            }
        }

        // 2. Cloud Supabase Insertion
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(None),
        };

        let endpoint = format!("{}/rest/v1/call_logs", url.trim_end_matches('/'));

        let resp = self
            .http_client
            .post(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(record)
            .send()
            .await?;

        if resp.status().is_success() {
            let created: Vec<Value> = resp.json().await.unwrap_or_default();
            let new_id = created.first().and_then(|v| v["id"].as_str()).map(|s| s.to_string());
            println!("✅ [Supabase CRM] Call log ({}) saved to cloud database!", safe_sid);
            Ok(new_id)
        } else {
            let err_text = resp.text().await.unwrap_or_default();
            eprintln!("⚠️ [Supabase CRM] Failed to insert call log: {}", err_text);
            Err(anyhow!("Supabase call log insert failed: {}", err_text))
        }
    }

    /// Telephony pipeline adapter: converts CallSessionRecord to CallLogRecord and logs to Supabase
    pub async fn log_call(&self, record: &CallSessionRecord) {
        let phone = record
            .caller_number
            .clone()
            .or_else(|| record.receiver_number.clone())
            .unwrap_or_else(|| "+919999999999".to_string());

        let full_text = record
            .transcript
            .iter()
            .map(|t| format!("{}: {}", t.role, t.content))
            .collect::<Vec<_>>()
            .join("\n");

        let log_rec = CallLogRecord {
            id: None,
            lead_id: None,
            phone_number: phone,
            direction: record.direction.clone(),
            call_duration_seconds: record.duration_seconds,
            recording_url: None,
            stream_sid: Some(record.stream_sid.clone()),
            status: record.status.clone(),
            transcript: record.transcript.clone(),
            full_transcript_text: Some(full_text),
            summary: None,
            customer_intent: None,
            sentiment: None,
            disposition: None,
        };

        let _ = self.save_call_log(&log_rec).await;
    }

    /// Schedules precision automated callback (e.g., "Tomorrow at 4:00 PM")
    pub async fn schedule_callback(
        &self,
        lead_id: &str,
        call_log_id: Option<&str>,
        phone_number: &str,
        customer_name: Option<&str>,
        callback_time: &str,
        reason: &str,
    ) -> Result<()> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(()),
        };

        let endpoint = format!("{}/rest/v1/scheduled_callbacks", url.trim_end_matches('/'));

        let payload = json!({
            "lead_id": lead_id,
            "call_log_id": call_log_id,
            "phone_number": phone_number,
            "customer_name": customer_name,
            "callback_time": callback_time,
            "reason": reason,
            "status": "pending"
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&payload)
            .send()
            .await?;

        if resp.status().is_success() {
            // Also update parent lead scheduled time
            let _ = self.update_lead_scheduled(lead_id, callback_time).await;
            println!("📅 [Supabase CRM] Precision callback scheduled for {} at {}", phone_number, callback_time);
            Ok(())
        } else {
            Err(anyhow!("Failed to schedule callback: {}", resp.status()))
        }
    }

    async fn update_lead_scheduled(&self, lead_id: &str, callback_time: &str) -> Result<()> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(()),
        };
        let endpoint = format!("{}/rest/v1/leads?id=eq.{}", url.trim_end_matches('/'), lead_id);
        let payload = json!({
            "status": "scheduled_callback",
            "scheduled_at": callback_time,
            "updated_at": Utc::now().to_rfc3339()
        });
        let _ = self.http_client.patch(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .json(&payload).send().await;
        Ok(())
    }

    /// Books strategic business meeting with calendar integration
    pub async fn book_meeting(
        &self,
        lead_id: Option<&str>,
        call_log_id: Option<&str>,
        attendee_name: &str,
        attendee_phone: &str,
        attendee_email: Option<&str>,
        meeting_time: &str,
        meeting_title: &str,
        notes: &str,
    ) -> Result<()> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(()),
        };

        let endpoint = format!("{}/rest/v1/meeting_bookings", url.trim_end_matches('/'));

        let payload = json!({
            "lead_id": lead_id,
            "call_log_id": call_log_id,
            "attendee_name": attendee_name,
            "attendee_phone": attendee_phone,
            "attendee_email": attendee_email,
            "meeting_title": meeting_title,
            "meeting_time": meeting_time,
            "meeting_duration_minutes": 30,
            "status": "confirmed",
            "notes": notes
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&payload)
            .send()
            .await?;

        if resp.status().is_success() {
            if let Some(lid) = lead_id {
                let _ = self.update_lead_status(lid, "meeting_booked").await;
            }
            println!("🤝 [Supabase CRM] Meeting booked for {} ({}) at {}", attendee_name, attendee_phone, meeting_time);
            Ok(())
        } else {
            Err(anyhow!("Failed to book meeting: {}", resp.status()))
        }
    }

    /// Issues autonomous support/refund ticket citing policy clauses
    pub async fn create_action_ticket(
        &self,
        lead_id: Option<&str>,
        call_log_id: Option<&str>,
        ticket_type: &str,
        description: &str,
        amount: Option<f64>,
        policy_clause: Option<&str>,
    ) -> Result<()> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(()),
        };

        let endpoint = format!("{}/rest/v1/action_tickets", url.trim_end_matches('/'));

        let payload = json!({
            "lead_id": lead_id,
            "call_log_id": call_log_id,
            "ticket_type": ticket_type,
            "description": description,
            "amount": amount,
            "policy_clause_cited": policy_clause,
            "status": "open"
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&payload)
            .send()
            .await?;

        if resp.status().is_success() {
            println!("🎫 [Supabase CRM] Action ticket ({}) created: {}", ticket_type, description);
            Ok(())
        } else {
            Err(anyhow!("Failed to create action ticket: {}", resp.status()))
        }
    }

    /// Bulk imports up to 10,000 phone numbers in high-throughput chunks
    pub async fn bulk_import_leads(&self, leads: &[LeadRecord]) -> Result<usize> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(0),
        };

        let endpoint = format!("{}/rest/v1/leads", url.trim_end_matches('/'));
        let mut total_inserted = 0;

        // Process in chunks of 500 for optimal HTTP payload size
        for chunk in leads.chunks(500) {
            let resp = self
                .http_client
                .post(&endpoint)
                .header("apikey", key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .header("Prefer", "resolution=ignore-duplicates, return=minimal")
                .json(&chunk)
                .send()
                .await?;

            if resp.status().is_success() {
                total_inserted += chunk.len();
            } else {
                let err = resp.text().await.unwrap_or_default();
                eprintln!("⚠️ [Supabase Bulk Import] Chunk warning: {}", err);
            }
        }

        println!("🚀 [Supabase CRM] Bulk imported {} leads into database.", total_inserted);
        Ok(total_inserted)
    }
}

pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
