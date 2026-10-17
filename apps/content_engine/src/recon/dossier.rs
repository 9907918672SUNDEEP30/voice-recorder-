use crate::recon::web_rss::NewsArticle;
use crate::recon::youtube::CompetitorVideo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDossier {
    pub topic: String,
    pub timestamp_utc: String,
    pub top_competitors: Vec<CompetitorVideo>,
    pub breaking_news: Vec<NewsArticle>,
    pub detected_viral_hooks: Vec<String>,
    pub reverse_engineered_tools: Vec<String>,
    pub target_audience_pain_point: String,
    pub recommended_angle: String,
}

pub struct DossierBuilder;

impl DossierBuilder {
    pub fn build(
        topic: &str,
        competitors: Vec<CompetitorVideo>,
        news: Vec<NewsArticle>,
    ) -> TrendDossier {
        let detected_viral_hooks = Self::extract_hooks_from_competitors(&competitors, topic);
        let reverse_engineered_tools = Self::detect_tools(topic, &competitors, &news);
        let (pain_point, recommended_angle) = Self::synthesize_angle(topic, &competitors);

        let timestamp_utc = chrono::Utc::now().to_rfc3339();

        TrendDossier {
            topic: topic.to_string(),
            timestamp_utc,
            top_competitors: competitors,
            breaking_news: news,
            detected_viral_hooks,
            reverse_engineered_tools,
            target_audience_pain_point: pain_point,
            recommended_angle,
        }
    }

    fn extract_hooks_from_competitors(
        competitors: &[CompetitorVideo],
        topic: &str,
    ) -> Vec<String> {
        let mut hooks = Vec::new();
        for v in competitors.iter().take(4) {
            hooks.push(format!("Pattern Interrupt: '{}'", v.title));
        }

        // Add psychological hook formulas tailored to the topic
        hooks.push(format!(
            "Curiosity Gap: Stop scrolling if you want to understand {} before tomorrow.",
            topic
        ));
        hooks.push(format!(
            "Contrarian: Why 99% of people are completely wrong about {}.",
            topic
        ));
        hooks.push(format!(
            "Secret Tool Reveal: The free tool that lets anyone master {} in 60 seconds.",
            topic
        ));

        hooks
    }

    fn detect_tools(
        topic: &str,
        competitors: &[CompetitorVideo],
        _news: &[NewsArticle],
    ) -> Vec<String> {
        let mut tools = Vec::new();
        let topic_lower = topic.to_lowercase();

        // Check if topic is retro/image/avatar trend
        if topic_lower.contains("80s")
            || topic_lower.contains("photo")
            || topic_lower.contains("image")
            || topic_lower.contains("avatar")
            || topic_lower.contains("yearbook")
        {
            tools.push("Midjourney v6 / SDXL (Retro Prompt Synthesis)".to_string());
            tools.push("Epik AI / Remini (Neural Face Styling)".to_string());
            tools.push("CodeFormer / GFPGAN (4K Face Detail Upscaling)".to_string());
            tools.push("CapCut / DaVinci Resolve (Fast 80s Grain & VHS Filter)".to_string());
        } else if topic_lower.contains("stock")
            || topic_lower.contains("finance")
            || topic_lower.contains("market")
        {
            tools.push("TradingView / Bloomberg Terminal (Real-time Candlesticks)".to_string());
            tools.push("Yahoo Finance API (Volume & Volatility Radar)".to_string());
            tools.push("Python Quant Engine (Black-Scholes & Mean Reversion)".to_string());
        } else {
            // General AI & Tech Tools
            tools.push("Qwen 2.5 / DeepSeek R1 (Advanced Reasoning LLM)".to_string());
            tools.push("LivePortrait / FasterLivePortrait (Zero-Shot Facial Animation)".to_string());
            tools.push("Whisper STT (Word-by-word Subtitle Alignment)".to_string());
            tools.push("FFmpeg NVENC (60fps Hardware Composition)".to_string());
        }

        // Scan competitor titles for tool mentions
        for c in competitors {
            let t = c.title.to_lowercase();
            if t.contains("midjourney") && !tools.iter().any(|x| x.contains("Midjourney")) {
                tools.push("Midjourney AI".to_string());
            }
            if t.contains("chatgpt") && !tools.iter().any(|x| x.contains("ChatGPT")) {
                tools.push("ChatGPT Plus / OpenAI".to_string());
            }
        }

        tools
    }

    fn synthesize_angle(
        topic: &str,
        competitors: &[CompetitorVideo],
    ) -> (String, String) {
        let pain_point = format!(
            "Audience wants fast, actionable clarity on {} without complicated jargon or paid paywalls.",
            topic
        );

        let recommended_angle = if !competitors.is_empty() {
            format!(
                "Fast Breakdown & Practical Tutorial: Combine the high-velocity hook of '{}' with a zero-cost step-by-step workflow.",
                competitors[0].title
            )
        } else {
            format!(
                "Exclusive Insider Breakdown: Demystify {} with immediate, real-world proof.",
                topic
            )
        };

        (pain_point, recommended_angle)
    }
}
