//! Supabase Telephony Call Logger & Persistence Client
//!
//! Provides cloud persistence for telephone call sessions, transcripts,
//! caller metadata, and analytics. Seamlessly saves call session records
//! to Supabase REST API (`call_logs` table) when configured, and maintains
//! local persistent audit logs under `logs/call_records/`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnRecord {
    pub role: String,
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

#[derive(Clone)]
pub struct SupabaseClient {
    url: Option<String>,
    api_key: Option<String>,
    table_name: String,
    http_client: reqwest::Client,
}

impl SupabaseClient {
    /// Initializes from system environment variables (`SUPABASE_URL`, `SUPABASE_KEY`)
    /// or from framework `config.json`.
    pub fn new_from_env_or_config(config: &serde_json::Value) -> Self {
        let is_valid_token = |t: &str| !t.trim().is_empty() && !t.contains("***") && t.len() > 8;

        let env_url = std::env::var("SUPABASE_URL").ok().filter(|s| !s.trim().is_empty());
        let env_key = std::env::var("SUPABASE_KEY")
            .or_else(|_| std::env::var("SUPABASE_SERVICE_ROLE_KEY"))
            .or_else(|_| std::env::var("SUPABASE_ANON_KEY"))
            .ok()
            .filter(|s| is_valid_token(s));

        let cfg_url = config["supabase"]["url"].as_str().map(|s| s.to_string()).filter(|s| !s.trim().is_empty());
        let cfg_key = config["supabase"]["key"].as_str().map(|s| s.to_string()).filter(|s| is_valid_token(s));

        let url = env_url.or(cfg_url);
        let api_key = env_key.or(cfg_key);

        let table_name = config["supabase"]["table_name"]
            .as_str()
            .unwrap_or("voice_call_logs")
            .to_string();

        if let (Some(u), Some(_)) = (&url, &api_key) {
            println!("✅ [Supabase] Connected to Supabase Cloud Endpoint: {}", u);
        } else {
            println!("ℹ️ [Supabase] Running in local audit mode (SUPABASE_URL & SUPABASE_KEY not configured or masked).");
        }

        Self {
            url,
            api_key,
            table_name,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Logs a completed call session asynchronously without blocking the telephony loop
    pub async fn log_call(&self, record: &CallSessionRecord) {
        // 1. Always write local audit record to disk with anti-traversal sanitization
        let clean_sid: String = record
            .stream_sid
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .take(64)
            .collect();
        let safe_sid = if clean_sid.is_empty() { "unknown_session" } else { &clean_sid };

        let log_dir = Path::new("logs/call_records");
        if let Ok(_) = fs::create_dir_all(log_dir) {
            let log_file = log_dir.join(format!("{}.json", safe_sid));
            if let Ok(json_str) = serde_json::to_string_pretty(record) {
                let _ = fs::write(log_file, json_str);
            }
        }

        // 2. If Supabase credentials configured, send POST to Supabase REST endpoint
        if let (Some(url), Some(key)) = (&self.url, &self.api_key) {
            let endpoint = format!("{}/rest/v1/{}", url.trim_end_matches('/'), self.table_name);
            println!("[Supabase] Uploading call record ({}) to Supabase: {}", safe_sid, endpoint);

            match self
                .http_client
                .post(&endpoint)
                .header("apikey", key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .header("Prefer", "return=minimal")
                .json(record)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        println!("✅ [Supabase] Call record ({}) saved successfully to database!", record.stream_sid);
                    } else {
                        let status = resp.status();
                        let text = resp.text().await.unwrap_or_default();
                        println!("⚠️ [Supabase] HTTP {} response: {}", status, text);
                    }
                }
                Err(e) => {
                    println!("⚠️ [Supabase] Network error uploading call record: {}", e);
                }
            }
        }
    }
}

pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
