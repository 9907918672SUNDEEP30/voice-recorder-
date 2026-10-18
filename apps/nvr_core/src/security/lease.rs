//! 24-Hour Rotating Cryptographic Cloud Lease Verifier
//! Validates Ed25519 signed license tokens from Shambhavi AI Labs Cloudflare Worker & Supabase.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of the local NVR appliance license lease
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaseStatus {
    /// Active and licensed. System operates at full capacity.
    Active {
        tenant_id: String,
        expires_at: DateTime<Utc>,
        cameras_entitled: u32,
    },
    /// In grace period (e.g. within 2 hours of expiry, warning on screen).
    GracePeriod {
        tenant_id: String,
        expires_at: DateTime<Utc>,
        warning: String,
    },
    /// Expired or locked out. Recording & AI are paused; screen displays renewal QR code.
    Locked {
        reason: String,
        renewal_url: String,
    },
}

/// Cryptographic Lease Token payload received from Cloudflare Worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseTokenPayload {
    pub box_id: String,
    pub machine_hash: String,
    pub tenant_id: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cameras_entitled: u32,
    pub signature_hex: String,
}

/// Lease Guardian Engine
#[derive(Debug, Clone)]
pub struct LeaseGuardian {
    pub machine_hash: String,
    pub current_status: LeaseStatus,
}

impl LeaseGuardian {
    pub fn new(machine_hash: String) -> Self {
        // FAIL CLOSED: Default to Locked status until cryptographically verified lease is active
        Self {
            machine_hash,
            current_status: LeaseStatus::Locked {
                reason: "Appliance startup. Cryptographic lease verification required.".to_string(),
                renewal_url: "https://billing.shambhavi-ai-labs.com/activate".to_string(),
            },
        }
    }

    /// Check if the appliance is currently authorized to record and monitor
    pub fn is_authorized(&self) -> bool {
        match &self.current_status {
            LeaseStatus::Active { expires_at, .. } => Utc::now() < *expires_at,
            LeaseStatus::GracePeriod { expires_at, .. } => Utc::now() < *expires_at,
            LeaseStatus::Locked { .. } => false,
        }
    }

    /// Return maximum camera channels entitled under current paid subscription
    pub fn max_allowed_cameras(&self) -> u32 {
        match &self.current_status {
            LeaseStatus::Active { cameras_entitled, .. } => *cameras_entitled,
            LeaseStatus::GracePeriod { .. } => 8, // Maintain service during grace period
            LeaseStatus::Locked { .. } => 0,      // All channels locked if subscription unpaid
        }
    }

    /// Unlocks local NVR lease when unified DRMGuard validates an Ed25519 cloud lease
    pub fn unlock_from_drm(&mut self, cameras_entitled: u32, expires_at: DateTime<Utc>) {
        self.current_status = LeaseStatus::Active {
            tenant_id: "shambhavi_cloud_lease".to_string(),
            expires_at,
            cameras_entitled,
        };
    }

    /// Verify an incoming lease payload from Cloudflare Worker
    pub fn apply_lease_token(&mut self, payload: LeaseTokenPayload) -> Result<(), String> {
        if payload.machine_hash != self.machine_hash {
            self.current_status = LeaseStatus::Locked {
                reason: "Hardware fingerprint mismatch: Token issued for different physical box".to_string(),
                renewal_url: "https://billing.shambhavi-ai-labs.com/support".to_string(),
            };
            return Err("Hardware mismatch".to_string());
        }

        let now = Utc::now();
        if now > payload.expires_at {
            self.current_status = LeaseStatus::Locked {
                reason: "Subscription expired. Please complete recurring payment.".to_string(),
                renewal_url: "https://billing.shambhavi-ai-labs.com/pay".to_string(),
            };
            return Err("Token expired".to_string());
        }

        // Strict Ed25519 signature verification against root key
        let canonical_msg = format!(
            "{}:{}:{}:{}:{}",
            payload.box_id,
            payload.machine_hash,
            payload.tenant_id,
            payload.expires_at.timestamp(),
            payload.cameras_entitled
        );

        if let Err(e) = drm_guard::verify_ed25519_signature(
            drm_guard::ROOT_ED25519_PUBLIC_KEY_HEX,
            &canonical_msg,
            &payload.signature_hex,
        ) {
            self.current_status = LeaseStatus::Locked {
                reason: format!("Cryptographic signature verification failed: {}", e),
                renewal_url: "https://billing.shambhavi-ai-labs.com/support".to_string(),
            };
            return Err(format!("Cryptographic signature mismatch: {}", e));
        }

        // Apply active lease
        self.current_status = LeaseStatus::Active {
            tenant_id: payload.tenant_id,
            expires_at: payload.expires_at,
            cameras_entitled: payload.cameras_entitled,
        };
        Ok(())
    }

    /// Update lease status periodically (checking for grace period or expiry)
    pub fn tick(&mut self) {
        let now = Utc::now();
        if let LeaseStatus::Active { tenant_id, expires_at, cameras_entitled: _ } = &self.current_status {
            let diff = *expires_at - now;
            if diff <= chrono::Duration::zero() {
                self.current_status = LeaseStatus::Locked {
                    reason: "24-hour lease expired. Internet disconnected or subscription paused.".to_string(),
                    renewal_url: "https://billing.shambhavi-ai-labs.com/renew".to_string(),
                };
            } else if diff < chrono::Duration::hours(2) {
                self.current_status = LeaseStatus::GracePeriod {
                    tenant_id: tenant_id.clone(),
                    expires_at: *expires_at,
                    warning: format!("Subscription renews in {} minutes. Ensure box is connected to internet.", diff.num_minutes()),
                };
            }
        }
    }
}
