use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpTab {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub tab_type: String,
    pub url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub websocket_debugger_url: Option<String>,
}

pub struct CdpScraper {
    pub client: Client,
    pub host: String,
    pub port: u16,
}

impl CdpScraper {
    pub fn new(host: &str, port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            client,
            host: host.to_string(),
            port,
        }
    }

    /// Checks if Google Chrome is reachable on the remote debugging port (e.g., 9222)
    pub async fn is_chrome_available(&self) -> bool {
        let url = format!("http://{}:{}/json/version", self.host, self.port);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Lists all active browser tabs
    pub async fn list_tabs(&self) -> Result<Vec<CdpTab>> {
        let url = format!("http://{}:{}/json/list", self.host, self.port);
        let resp = self.client.get(&url).send().await?;
        let tabs = resp.json::<Vec<CdpTab>>().await?;
        Ok(tabs)
    }

    /// Opens a new tab pointing to target URL
    pub async fn open_tab(&self, target_url: &str) -> Result<CdpTab> {
        let url = format!(
            "http://{}:{}/json/new?{}",
            self.host,
            self.port,
            urlencoding_simple(target_url)
        );
        let resp = self.client.put(&url).send().await?;
        let tab = resp.json::<CdpTab>().await?;
        Ok(tab)
    }

    /// Closes a tab by ID
    pub async fn close_tab(&self, tab_id: &str) -> Result<()> {
        let url = format!("http://{}:{}/json/close/{}", self.host, self.port, tab_id);
        let _ = self.client.get(&url).send().await?;
        Ok(())
    }
}

fn urlencoding_simple(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char)
            }
            _ => encoded.push_str(&format!("%{:02X}", b)),
        }
    }
    encoded
}
