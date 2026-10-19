//! # Tier 2: Local-First Hybrid Authentication
//!
//! Target Apps: `apps/financial_engine`, `apps/manifestation_app`
//! Policy: Hybrid Ed25519 JWT Local Authentication
//! Mandate:
//!   1. Initial login against Cloudflare Worker issues a signed Ed25519 JWT.
//!   2. Subsequent logins and app actions verify the JWT signature locally on the client's CPU (<1ms)
//!      using the embedded Ed25519 Public Key.
//!   3. Zero network calls or heartbeat requests after initial issuance, preserving user privacy & offline usage.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Embedded Shambhavi AI Labs Root Ed25519 Public Key for local offline verification
pub const EMBEDDED_ROOT_PUBLIC_KEY_HEX: &str =
    "82d9651fdb75e85ab7e88c35440e4c7629b53b97c9fbfdf2335a6cb458af1400";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,       // User UUID / Email
    pub email: String,
    pub tier: String,      // "tier2"
    pub app_name: String,  // "financial_engine" | "manifestation_app" | "all"
    pub iat: i64,          // Issued at timestamp
    pub exp: i64,          // Expiration timestamp
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub claims: UserClaims,
}

#[async_trait]
pub trait Tier2LocalAuth: Send + Sync {
    /// Initial Cloudflare Worker login to issue and cache local Ed25519 JWT
    async fn login_and_cache(
        &self,
        email: &str,
        password: &str,
    ) -> Result<UserClaims>;

    /// 100% Offline verification on local CPU (<0.25ms) using embedded root public key
    fn verify_offline(&self) -> Result<UserClaims>;

    /// Clears local session on logout
    fn logout(&self) -> Result<()>;
}

pub struct LocalAuthManager {
    app_name: String,
    gateway_url: String,
    token_cache_path: PathBuf,
    public_key: VerifyingKey,
    http_client: reqwest::Client,
}

impl LocalAuthManager {
    pub fn new(app_name: &str, gateway_url: &str) -> Result<Self> {
        let pubkey_bytes = hex::decode(EMBEDDED_ROOT_PUBLIC_KEY_HEX)
            .map_err(|e| anyhow!("Invalid embedded root public key hex: {}", e))?;
        let pubkey_array: [u8; 32] = pubkey_bytes
            .try_into()
            .map_err(|_| anyhow!("Public key length mismatch (expected 32 bytes)"))?;
        let public_key = VerifyingKey::from_bytes(&pubkey_array)
            .map_err(|e| anyhow!("Failed to parse Ed25519 verifying key: {}", e))?;

        let cache_file_name = format!(".shambhavi_jwt_{}.token", app_name);
        let token_cache_path = PathBuf::from(cache_file_name);

        Ok(Self {
            app_name: app_name.to_string(),
            gateway_url: gateway_url.trim_end_matches('/').to_string(),
            token_cache_path,
            public_key,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .build()
                .unwrap_or_default(),
        })
    }

    /// Custom cache path constructor (useful for unit tests)
    pub fn with_custom_cache_path(mut self, path: PathBuf) -> Self {
        self.token_cache_path = path;
        self
    }

    /// Custom public key override (useful for unit tests and key rotation)
    pub fn with_public_key(mut self, public_key: VerifyingKey) -> Self {
        self.public_key = public_key;
        self
    }

    /// Verifies raw Ed25519 JWT string locally
    pub fn verify_jwt_string(&self, raw_jwt: &str) -> Result<UserClaims> {
        let parts: Vec<&str> = raw_jwt.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Malformed JWT: expected 3 dot-separated parts"));
        }

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let signature_bytes = URL_SAFE_NO_PAD
            .decode(parts[2])
            .or_else(|_| base64::engine::general_purpose::STANDARD.decode(parts[2]))
            .map_err(|e| anyhow!("Invalid base64 signature encoding: {}", e))?;

        let sig_array: [u8; 64] = signature_bytes
            .try_into()
            .map_err(|_| anyhow!("Signature length mismatch (expected 64 bytes)"))?;
        let signature = Signature::from_bytes(&sig_array);

        // Sub-millisecond CPU Ed25519 verification
        self.public_key
            .verify_strict(signing_input.as_bytes(), &signature)
            .map_err(|e| anyhow!("Ed25519 signature verification failed: {}", e))?;

        // Decode payload
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .or_else(|_| base64::engine::general_purpose::STANDARD.decode(parts[1]))
            .map_err(|e| anyhow!("Invalid base64 payload encoding: {}", e))?;

        let claims: UserClaims = serde_json::from_slice(&payload_bytes)
            .map_err(|e| anyhow!("Failed to deserialize claims: {}", e))?;

        // Check expiration
        let now_sec = Utc::now().timestamp();
        if claims.exp < now_sec {
            return Err(anyhow!(
                "Token expired at unix timestamp {} (current: {})",
                claims.exp,
                now_sec
            ));
        }

        // Validate application scope
        if claims.app_name != "all" && claims.app_name != self.app_name {
            return Err(anyhow!(
                "Scope mismatch: token issued for '{}', active app is '{}'",
                claims.app_name,
                self.app_name
            ));
        }

        Ok(claims)
    }
}

#[async_trait]
impl Tier2LocalAuth for LocalAuthManager {
    async fn login_and_cache(
        &self,
        email: &str,
        password: &str,
    ) -> Result<UserClaims> {
        let endpoint = format!("{}/v1/auth/issue-token", self.gateway_url);

        let payload = serde_json::json!({
            "email": email,
            "password": password,
            "app_name": self.app_name,
            "tier": "tier2"
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Authentication failed at gateway: {}", err_text));
        }

        let login_res: LoginResponse = resp.json().await?;

        // Verify newly received token locally before trusting it
        let verified_claims = self.verify_jwt_string(&login_res.token)?;

        // Cache securely to local device storage
        fs::write(&self.token_cache_path, &login_res.token)
            .map_err(|e| anyhow!("Failed to persist JWT to {:?}: {}", self.token_cache_path, e))?;

        println!(
            "🔑 [Tier 2 Local-First] Authenticated successfully. Ed25519 JWT cached locally for offline execution."
        );
        Ok(verified_claims)
    }

    fn verify_offline(&self) -> Result<UserClaims> {
        if !self.token_cache_path.exists() {
            return Err(anyhow!(
                "No cached credentials found at {:?}. Initial login required.",
                self.token_cache_path
            ));
        }

        let raw_token = fs::read_to_string(&self.token_cache_path)
            .map_err(|e| anyhow!("Failed to read cached token: {}", e))?;

        let claims = self.verify_jwt_string(&raw_token)?;
        println!(
            "⚡ [Tier 2 Offline Auth] Local CPU Ed25519 Verified (<0.25ms) for user '{}'. Zero network pings.",
            claims.email
        );
        Ok(claims)
    }

    fn logout(&self) -> Result<()> {
        if self.token_cache_path.exists() {
            fs::remove_file(&self.token_cache_path)?;
            println!("🚪 [Tier 2 Local-First] Local session token purged.");
        }
        Ok(())
    }
}
