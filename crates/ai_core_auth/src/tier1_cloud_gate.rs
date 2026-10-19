//! # Tier 1: High-Value Cloud-Connected Dual-Gate Verification
//!
//! Target Apps: `apps/voice_agent`, `apps/content_engine`
//! Policy: Dual-Gate Live Verification (Cloudflare Edge Worker + Supabase Credit Ledger)
//! Mandate: NO offline caching or local execution bypass. Every incoming request must pass
//! live server-side token validation and credit availability checks before inference.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPermit {
    pub permit_id: String,
    pub user_id: String,
    pub app_name: String,
    pub granted_units: f64,
    pub balance_remaining: f64,
    pub valid_until: DateTime<Utc>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreFlightRequest {
    pub app_name: String,
    pub requested_units: f64,
    pub operation_type: String, // e.g. "telephony_outbound_dial", "video_render_job", "qwen_batch_generation"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitUsageRequest {
    pub permit_id: String,
    pub app_name: String,
    pub actual_units_consumed: f64,
}

#[async_trait]
pub trait Tier1CloudGate: Send + Sync {
    /// Validates bearer credentials and confirms sufficient credit balance before starting inference
    async fn request_execution_permit(
        &self,
        app_name: &str,
        requested_units: f64,
        operation_type: &str,
    ) -> Result<ExecutionPermit>;

    /// Atomically deducts consumed units from the user's ledger upon job completion or call hangup
    async fn commit_execution_usage(
        &self,
        permit: &ExecutionPermit,
        actual_units_consumed: f64,
    ) -> Result<()>;
}

#[derive(Clone)]
pub struct CloudGateClient {
    gateway_url: String,
    auth_token: String,
    http_client: reqwest::Client,
}

impl CloudGateClient {
    pub fn new(gateway_url: &str, auth_token: &str) -> Self {
        Self {
            gateway_url: gateway_url.trim_end_matches('/').to_string(),
            auth_token: auth_token.trim().to_string(),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl Tier1CloudGate for CloudGateClient {
    async fn request_execution_permit(
        &self,
        app_name: &str,
        requested_units: f64,
        operation_type: &str,
    ) -> Result<ExecutionPermit> {
        let endpoint = format!("{}/v1/cloud/execute-gate", self.gateway_url);

        let req_body = PreFlightRequest {
            app_name: app_name.to_string(),
            requested_units,
            operation_type: operation_type.to_string(),
        };

        let response = self
            .http_client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let permit: ExecutionPermit = response.json().await?;
            println!(
                "✅ [Tier 1 Dual-Gate] Execution permit granted for '{}' (Permit: {})",
                app_name, permit.permit_id
            );
            Ok(permit)
        } else if status == reqwest::StatusCode::PAYMENT_REQUIRED {
            let err_msg = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "HTTP 402 Payment Required: Insufficient credits for '{}': {}",
                app_name,
                err_msg
            ))
        } else if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            let err_msg = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "HTTP 403 Forbidden: Invalid or expired token: {}",
                err_msg
            ))
        } else {
            let err_msg = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "Gateway rejected execution request (HTTP {}): {}",
                status,
                err_msg
            ))
        }
    }

    async fn commit_execution_usage(
        &self,
        permit: &ExecutionPermit,
        actual_units_consumed: f64,
    ) -> Result<()> {
        let endpoint = format!("{}/v1/cloud/commit-usage", self.gateway_url);

        let req_body = CommitUsageRequest {
            permit_id: permit.permit_id.clone(),
            app_name: permit.app_name.clone(),
            actual_units_consumed,
        };

        let response = self
            .http_client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await?;

        if response.status().is_success() {
            println!(
                "💳 [Tier 1 Dual-Gate] Successfully committed {:.2} units for permit {}",
                actual_units_consumed, permit.permit_id
            );
            Ok(())
        } else {
            let err_msg = response.text().await.unwrap_or_default();
            Err(anyhow!("Failed to commit usage to gateway: {}", err_msg))
        }
    }
}
