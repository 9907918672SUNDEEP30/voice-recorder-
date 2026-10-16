//! # Module 5: Chrome DevTools Protocol (CDP) Autonomous Deep Web Research Engine
//!
//! Direct headless browser automation and web scraping using `chromiumoxide`
//! (CDP via native WebSockets). Zero Selenium/Puppeteer wrappers.
//! Scrapes real-time exchange announcements, RBI/PIB circulars, and sector reports.
//! Synthesizes macro policy sentiment via local Qwen LLM (`C:\AI_Models\qwen2-vl-2b`).

use ai_core::models::adapters::LlmAdapter;
use ai_core::models::onnx_adapters::QwenLocal;
use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Structured Intelligence Extracted from Web Portals via CDP
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrapedIntelligence {
    pub url: String,
    pub title: String,
    pub cleaned_text_snippet: String,
    pub key_takeaways: Vec<String>,
    pub policy_impact_score: f32, // -5.0 (Severe Headwind) to +5.0 (Massive Tailwinds)
    pub sentiment: String,
}

pub struct CdpAutonomousScraper {
    clean_html_regex: Regex,
    http_client: reqwest::Client,
    local_qwen: Option<Arc<QwenLocal>>,
}

impl Default for CdpAutonomousScraper {
    fn default() -> Self {
        Self::new(None)
    }
}

impl CdpAutonomousScraper {
    pub fn new(models_dir: Option<&Path>) -> Self {
        let clean_html_regex = Regex::new(r"(?s)<script.*?</script>|<style.*?</style>|<[^>]+>|\s+").unwrap();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();

        let local_qwen = Self::try_load_qwen(models_dir);

        Self {
            clean_html_regex,
            http_client,
            local_qwen,
        }
    }

    /// Attempt to load local Qwen LLM from C:\AI_Models\qwen2-vl-2b
    fn try_load_qwen(models_dir: Option<&Path>) -> Option<Arc<QwenLocal>> {
        let base_path = models_dir
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| Path::new("C:/AI_Models").to_path_buf());

        let qwen_model = base_path.join("qwen2-vl-2b").join("llmmodel.onnx");
        let vision_model = base_path.join("qwen2-vl-2b").join("visionmodel.onnx");
        let vision_encoder = base_path.join("qwen2-vl-2b").join("vision_encoder.onnx");
        let tokenizer = base_path.join("qwen2-vl-2b").join("tokenizer.json");

        if qwen_model.exists() && tokenizer.exists() && vision_model.exists() && vision_encoder.exists() {
            if let (Some(m_str), Some(v_str), Some(ve_str), Some(t_str)) = (
                qwen_model.to_str(),
                vision_model.to_str(),
                vision_encoder.to_str(),
                tokenizer.to_str(),
            ) {
                match QwenLocal::new(m_str, v_str, ve_str, t_str) {
                    Ok(adapter) => {
                        println!("Successfully attached Local Qwen LLM from {:?}", qwen_model);
                        return Some(Arc::new(adapter));
                    }
                    Err(e) => eprintln!("Failed to initialize Local Qwen LLM: {}", e),
                }
            }
        }
        None
    }

    /// Cleans raw HTML to semantic plain text
    pub fn clean_html(&self, html: &str) -> String {
        let cleaned = self.clean_html_regex.replace_all(html, " ");
        cleaned.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    /// Autonomous CDP Headless Browser Scraping via native WebSocket
    pub async fn scrape_with_cdp(&self, target_url: &str) -> Result<ScrapedIntelligence> {
        let config = BrowserConfig::builder()
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .build()
            .map_err(|e| anyhow::anyhow!("CDP BrowserConfig error: {}", e))?;

        let (mut browser, mut handler) = Browser::launch(config)
            .await
            .context("Failed to launch headless browser via CDP")?;

        let handle = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        let page = browser
            .new_page(target_url)
            .await
            .context("Failed to open page via CDP")?;

        let page_title = page.get_title().await.unwrap_or_default().unwrap_or_else(|| "Financial Page".to_string());
        let html_content = page.content().await.context("Failed to get HTML content")?;

        let _ = browser.close().await;
        let _ = handle.await;

        let cleaned = self.clean_html(&html_content);
        let snippet = cleaned.chars().take(2000).collect::<String>();

        self.evaluate_macro_intelligence(target_url, &page_title, &snippet).await
    }

    /// Fast HTTP scraping fallback when full browser CDP execution is not required (e.g. RSS / APIs)
    pub async fn scrape_with_fast_http(&self, target_url: &str) -> Result<ScrapedIntelligence> {
        let res = self
            .http_client
            .get(target_url)
            .send()
            .await
            .context("Fast HTTP scrape failed")?;

        let html = res.text().await.context("Failed to read response body")?;
        let cleaned = self.clean_html(&html);
        let snippet = cleaned.chars().take(2000).collect::<String>();

        self.evaluate_macro_intelligence(target_url, "Public Financial Intelligence", &snippet).await
    }

    /// Evaluates text using local Qwen LLM (`C:\AI_Models\qwen2-vl-2b`) if available, or deterministic rules
    async fn evaluate_macro_intelligence(
        &self,
        url: &str,
        title: &str,
        snippet: &str,
    ) -> Result<ScrapedIntelligence> {
        let mut key_takeaways = Vec::new();
        let mut policy_impact_score = 0.0f32;
        let sentiment: String;

        if let Some(qwen) = &self.local_qwen {
            let prompt = format!(
                "Analyze the following financial/policy text:\nTitle: {}\nContent: {}\nList 2 key takeaways and state whether it is Bullish, Bearish, or Neutral.",
                title, snippet
            );
            let cancel_token = CancellationToken::new();
            if let Ok(mut rx) = qwen
                .generate_stream(
                    "You are a professional equity macro analyst. Be concise.",
                    vec![("user".to_string(), prompt)],
                    cancel_token,
                )
                .await
            {
                let mut full_llm_response = String::new();
                while let Some(chunk) = rx.recv().await {
                    full_llm_response.push_str(&chunk);
                }
                key_takeaways.push(full_llm_response.trim().to_string());
            }
        }

        // Deterministic Fallback & keyword scoring
        let lower = snippet.to_lowercase();
        if lower.contains("pli scheme") || lower.contains("tariff hike") || lower.contains("capacity expansion") {
            policy_impact_score += 3.0;
        }
        if lower.contains("interest rate cut") || lower.contains("subsidy approved") {
            policy_impact_score += 2.0;
        }
        if lower.contains("tax increase") || lower.contains("anti-dumping duty revoked") || lower.contains("investigation") {
            policy_impact_score -= 3.0;
        }

        sentiment = if policy_impact_score > 1.5 {
            "Strong Macro Tailwinds".to_string()
        } else if policy_impact_score < -1.5 {
            "Severe Regulatory Headwinds".to_string()
        } else {
            "Neutral Policy Landscape".to_string()
        };

        if key_takeaways.is_empty() {
            key_takeaways.push(format!("Scraped {} chars of real-time intelligence.", snippet.len()));
        }

        Ok(ScrapedIntelligence {
            url: url.to_string(),
            title: title.to_string(),
            cleaned_text_snippet: snippet.to_string(),
            key_takeaways,
            policy_impact_score,
            sentiment,
        })
    }
}
