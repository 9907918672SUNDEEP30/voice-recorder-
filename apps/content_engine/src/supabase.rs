//! # Content Engine Supabase Cloud Integration
//!
//! Provides cloud persistence for omnichannel AI-generated media matrices:
//! 60-second shorts, YouTube long-form scripts, SEO blog articles, and Twitter (X) threads.
//! Database: Supabase PostgreSQL (`hbnwmjrrhvpibtqsucqz`).

#![allow(dead_code)]

use crate::generator::OmnichannelPackage;
use crate::recon::TrendDossier;
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPieceRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub topic: String,
    pub status: String, // "draft", "generated", "reviewing", "approved", "published", "archived"
    pub short_hook: Option<String>,
    pub short_cta: Option<String>,
    pub short_script_json: Option<Value>,
    pub full_spoken_transcript: Option<String>,
    pub long_form_script: Option<String>,
    pub blog_article: Option<String>,
    pub twitter_thread: Option<Value>,
    pub audio_url: Option<String>,
    pub captions_ass: Option<String>,
    pub video_preview_url: Option<String>,
    pub word_count: Option<i32>,
    pub estimated_duration_seconds: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub custom_metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingLogRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub content_id: String,
    pub platform: String, // "youtube_shorts", "youtube_long", "blog_seo", "twitter_x", "linkedin", "instagram_reels"
    pub external_post_id: Option<String>,
    pub publish_status: String, // "pending", "scheduled", "published", "failed"
    pub scheduled_at: Option<String>,
    pub published_at: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDossierRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub topic: String,
    pub detected_viral_hooks: Option<Value>,
    pub competitor_videos: Option<Value>,
    pub news_articles: Option<Value>,
    pub reverse_engineered_tools: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct SupabaseContentClient {
    url: Option<String>,
    api_key: Option<String>,
    http_client: reqwest::Client,
}

impl SupabaseContentClient {
    /// Initializes from system environment variables or framework config.json
    pub fn new_from_env_or_config(config: &serde_json::Value) -> Self {
        let is_valid_token = |t: &str| !t.trim().is_empty() && !t.contains("***") && t.len() > 8;

        let env_url = std::env::var("CONTENT_SUPABASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let env_key = std::env::var("CONTENT_SUPABASE_KEY")
            .or_else(|_| std::env::var("CONTENT_SUPABASE_SECRET"))
            .ok()
            .filter(|s| is_valid_token(s));

        let cfg_url = config["content_engine"]["supabase"]["url"]
            .as_str()
            .or_else(|| config["supabase"]["url"].as_str())
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty());

        let cfg_key = config["content_engine"]["supabase"]["key"]
            .as_str()
            .or_else(|| config["supabase"]["key"].as_str())
            .map(|s| s.to_string())
            .filter(|s| is_valid_token(s));

        let url = env_url.or(cfg_url);
        let api_key = env_key.or(cfg_key);

        if let (Some(u), Some(_)) = (&url, &api_key) {
            println!("✅ [Content Engine] Connected to Supabase Cloud: {}", u);
        } else {
            println!("ℹ️ [Content Engine] Running in local audit mode (credentials not configured).");
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

    /// Saves the synthesized OmnichannelPackage and associated media paths to Supabase Cloud
    pub async fn save_content_package(
        &self,
        pkg: &OmnichannelPackage,
        audio_url: Option<&str>,
        captions_ass: Option<&str>,
        video_preview_url: Option<&str>,
    ) -> Result<String> {
        let word_count = pkg.short_script.full_spoken_transcript.split_whitespace().count() as i32;
        let estimated_duration = (word_count as f32 / 2.5).max(3.0);

        let short_lines_json = serde_json::to_value(&pkg.short_script.lines).unwrap_or(json!([]));
        let twitter_thread_json = serde_json::to_value(&pkg.twitter_thread).unwrap_or(json!([]));

        let record = ContentPieceRecord {
            id: None,
            topic: pkg.topic.clone(),
            status: "generated".to_string(),
            short_hook: Some(pkg.short_script.hook.clone()),
            short_cta: Some(pkg.short_script.cta.clone()),
            short_script_json: Some(short_lines_json),
            full_spoken_transcript: Some(pkg.short_script.full_spoken_transcript.clone()),
            long_form_script: Some(pkg.long_form_script.clone()),
            blog_article: Some(pkg.blog_article.clone()),
            twitter_thread: Some(twitter_thread_json),
            audio_url: audio_url.map(|s| s.to_string()),
            captions_ass: captions_ass.map(|s| s.to_string()),
            video_preview_url: video_preview_url.map(|s| s.to_string()),
            word_count: Some(word_count),
            estimated_duration_seconds: Some(estimated_duration),
            tags: Some(vec!["ai".to_string(), "autonomous".to_string(), "viral".to_string()]),
            custom_metadata: Some(json!({
                "generator": "OmnichannelGenerator v1.0",
                "synced_at": Utc::now().to_rfc3339()
            })),
        };

        // 1. Audit write locally to disk
        let safe_topic = pkg
            .topic
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .take(64)
            .collect::<String>();

        let log_dir = Path::new("logs/generated_content");
        if let Ok(_) = fs::create_dir_all(log_dir) {
            let log_file = log_dir.join(format!("{}.json", safe_topic));
            if let Ok(json_str) = serde_json::to_string_pretty(&record) {
                let _ = fs::write(log_file, json_str);
            }
        }

        // 2. Cloud Supabase Insertion
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok("local_only_id".to_string()),
        };

        let endpoint = format!("{}/rest/v1/content_pieces", url.trim_end_matches('/'));

        let resp = self
            .http_client
            .post(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&record)
            .send()
            .await?;

        if resp.status().is_success() {
            let created: Vec<Value> = resp.json().await.unwrap_or_default();
            let new_id = created
                .first()
                .and_then(|v| v["id"].as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            println!("✅ [Content Engine] Final content piece saved to Supabase (ID: {})", new_id);
            Ok(new_id)
        } else {
            let err_text = resp.text().await.unwrap_or_default();
            eprintln!("⚠️ [Content Engine] Failed to save content piece: {}", err_text);
            Err(anyhow!("Supabase content insert failed: {}", err_text))
        }
    }

    /// Saves intelligence research trend dossier to Supabase
    pub async fn save_trend_dossier(&self, dossier: &TrendDossier) -> Result<String> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok("local_only_dossier".to_string()),
        };

        let endpoint = format!("{}/rest/v1/trend_dossiers", url.trim_end_matches('/'));

        let viral_hooks_json = serde_json::to_value(&dossier.detected_viral_hooks).unwrap_or(json!([]));
        let comp_videos_json = serde_json::to_value(&dossier.top_competitors).unwrap_or(json!([]));
        let news_json = serde_json::to_value(&dossier.breaking_news).unwrap_or(json!([]));

        let payload = json!({
            "topic": dossier.topic,
            "detected_viral_hooks": viral_hooks_json,
            "competitor_videos": comp_videos_json,
            "news_articles": news_json,
            "reverse_engineered_tools": dossier.reverse_engineered_tools
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .header("apikey", key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&payload)
            .send()
            .await?;

        if resp.status().is_success() {
            let created: Vec<Value> = resp.json().await.unwrap_or_default();
            let new_id = created
                .first()
                .and_then(|v| v["id"].as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            println!("📊 [Content Engine] Trend dossier saved to Supabase (ID: {})", new_id);
            Ok(new_id)
        } else {
            let err_text = resp.text().await.unwrap_or_default();
            Err(anyhow!("Supabase trend dossier insert failed: {}", err_text))
        }
    }

    /// Logs syndication or publishing event for a content piece
    pub async fn log_publishing_event(
        &self,
        content_id: &str,
        platform: &str,
        external_post_id: Option<&str>,
        status: &str,
    ) -> Result<()> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(()),
        };

        let endpoint = format!("{}/rest/v1/publishing_logs", url.trim_end_matches('/'));

        let payload = json!({
            "content_id": content_id,
            "platform": platform,
            "external_post_id": external_post_id,
            "publish_status": status,
            "published_at": Utc::now().to_rfc3339()
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
            println!("📡 [Content Engine] Published to {} logged in Supabase!", platform);
            Ok(())
        } else {
            Err(anyhow!("Failed to log publishing event: {}", resp.status()))
        }
    }

    /// Fetches recent campaigns from cloud
    pub async fn fetch_recent_campaigns(&self, limit: usize) -> Result<Vec<ContentPieceRecord>> {
        let (url, key) = match (&self.url, &self.api_key) {
            (Some(u), Some(k)) => (u, k),
            _ => return Ok(Vec::new()),
        };

        let endpoint = format!(
            "{}/rest/v1/content_pieces?order=created_at.desc&limit={}",
            url.trim_end_matches('/'),
            limit
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
            let pieces: Vec<ContentPieceRecord> = resp.json().await?;
            Ok(pieces)
        } else {
            Err(anyhow!("Failed to fetch campaigns: {}", resp.status()))
        }
    }
}
