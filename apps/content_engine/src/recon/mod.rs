pub mod cdp_scraper;
pub mod dossier;
pub mod web_rss;
pub mod youtube;

pub use cdp_scraper::CdpScraper;
pub use dossier::{DossierBuilder, TrendDossier};
pub use web_rss::{NewsArticle, WebRssRecon};
pub use youtube::{CompetitorVideo, YouTubeRecon};

use anyhow::Result;

pub struct TrendReconEngine {
    cdp: CdpScraper,
    youtube: YouTubeRecon,
    rss: WebRssRecon,
}

impl TrendReconEngine {
    pub fn new(cdp_host: &str, cdp_port: u16) -> Self {
        Self {
            cdp: CdpScraper::new(cdp_host, cdp_port),
            youtube: YouTubeRecon::new(),
            rss: WebRssRecon::new(),
        }
    }

    /// Performs complete cross-platform trend reconnaissance on any topic
    pub async fn analyze_topic(&self, topic: &str) -> Result<TrendDossier> {
        println!("🔍 [Recon] Starting Universal Trend Reconnaissance for: '{}'", topic);

        // Check Chrome DevTools Protocol availability
        if self.cdp.is_chrome_available().await {
            println!("  ✅ Connected to Google Chrome CDP (Port: {})", self.cdp.port);
        } else {
            println!("  ℹ️ Chrome CDP not detected at port {}. Operating in stealth HTTP/RSS mode.", self.cdp.port);
        }

        // 1. YouTube Competitor & View Velocity analysis
        println!("  📺 Scanning YouTube competitor videos and view velocity...");
        let competitors = self.youtube.search_competitors(topic).await.unwrap_or_default();
        println!("  ✅ Found {} high-performing competitor videos.", competitors.len());

        // 2. Breaking News & Community Buzz
        println!("  📰 Fetching breaking news & Reddit discussions...");
        let mut news = self.rss.fetch_google_news(topic).await.unwrap_or_default();
        if let Ok(mut reddit_items) = self.rss.fetch_reddit_buzz(topic).await {
            news.append(&mut reddit_items);
        }
        println!("  ✅ Ingested {} public articles & community threads.", news.len());

        // 3. Build Structured Trend Dossier & Reverse-Engineer Tools
        println!("  🧠 Reverse-engineering viral hooks and tool mechanics...");
        let dossier = DossierBuilder::build(topic, competitors, news);
        println!("  🎉 Trend Dossier compiled successfully!");

        Ok(dossier)
    }
}
