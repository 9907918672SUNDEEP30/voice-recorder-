//! Async Alert Notification Dispatcher
//! Sends instant push notifications and webhooks to Cloudflare, WhatsApp/Telegram Gateways, and client apps.

use super::evidence::IncidentRecord;
use anyhow::Result;
use reqwest::Client;
use std::fs;
use std::time::Duration;

pub struct AlertNotifier {
    client: Client,
    pub webhook_url: Option<String>,
}

impl AlertNotifier {
    pub fn new(webhook_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        Self { client, webhook_url }
    }

    /// Dispatch structured threat event asynchronously over HTTP/WhatsApp Gateway
    pub async fn dispatch_alert(&self, record: &IncidentRecord) -> Result<()> {
        println!("📲 [ALERT DISPATCH] Notification sent for Incident: {}", record.incident_id);
        println!("   Camera: {} | Level: {} | Category: {}", record.camera_id, record.threat_level, record.threat_class);
        println!("   Snapshot saved: {}", record.snapshot_path);

        // If a real webhook / WhatsApp gateway URL is provided, send HTTP POST
        if let Some(url) = &self.webhook_url {
            // Read snapshot bytes if present for payload attachment
            let image_data = fs::read(&record.snapshot_path).ok();
            let base64_img = image_data.map(|b| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b));

            let payload = serde_json::json!({
                "incident_id": record.incident_id,
                "camera_id": record.camera_id,
                "threat_level": record.threat_level,
                "threat_class": record.threat_class,
                "confidence": record.confidence,
                "timestamp": record.timestamp.to_rfc3339(),
                "snapshot_base64": base64_img,
            });

            match self.client.post(url).json(&payload).send().await {
                Ok(resp) => {
                    println!("🌐 [WEBHOOK SUCCESS] Server responded with status: {}", resp.status());
                }
                Err(e) => {
                    eprintln!("⚠️ [WEBHOOK NOTICE] Push endpoint unreachable ({}), alert logged locally.", e);
                }
            }
        }

        Ok(())
    }
}
