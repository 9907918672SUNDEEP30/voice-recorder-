# 🛡️ Universal DRM Guard & Metered Usage Framework (`crates/drm_guard`)

Enterprise-grade, cross-platform, zero-Python shared licensing and cryptographic protection framework for the `ai_core_framework` workspace.

---

## 🌟 Key Capabilities

1. **Hardware Machine Binding:**
   - Deterministically generates a unique 64-character SHA-256 fingerprint from physical hardware:
     $$\text{Machine UUID} = \text{SHA256}(\text{Motherboard UUID} \parallel \text{CPU Processor ID} \parallel \text{Primary MAC})$$
   - Prevents unauthorized cloning, VM duplication, or transferring disk images between machines.

2. **24-Hour Rotating Cryptographic Leasing:**
   - Background Tokio asynchronous daemon pings Cloudflare Edge Worker (`/api/heartbeat`).
   - Supports 24-hour lease renewals with an automatic offline grace-period buffer during internet outages.

3. **Metered Telemetry via `UsageMetricProvider`:**
   - Dynamic Rust trait allowing any app to inject custom usage metrics (CCTV cameras, Voice AI minutes, vector documents, stock scans) into the signed heartbeat payload.

4. **Volatile AES-256-GCM Key Management & Zeroization:**
   - The master decryption key is stored **exclusively in volatile RAM** wrapped in `zeroize::ZeroizeOnDrop`.
   - Never written to disk or swap space.
   - Upon lease expiration, subscription lapse, or tampering, the key is instantly overwritten with zeros (`0x00`), locking SQLite vaults and in-memory caches.

5. **Anti-Clock-Tampering Guard:**
   - Monotonic time tracking (`std::time::Instant`) paired with state checkpoints detects system clock rollbacks and backdating attacks.

---

## 🚀 Quick Start Example

```rust
use drm_guard::{DRMGuard, UsageMetricProvider};
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct CctvMetrics {
    camera_count: Arc<AtomicUsize>,
}

#[async_trait]
impl UsageMetricProvider for CctvMetrics {
    fn app_name(&self) -> &'static str {
        "cctv_guard"
    }

    async fn collect_metrics(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert("camera_count".to_string(), json!(self.camera_count.load(Ordering::Relaxed)));
        map
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let metrics = Arc::new(CctvMetrics {
        camera_count: Arc::new(AtomicUsize::new(8)),
    });

    let drm = Arc::new(DRMGuard::builder()
        .client_id("sme_client_001")
        .license_key("KEY-ENTERPRISE-2026")
        .heartbeat_url(Some("https://licensing.example.com/api/heartbeat".to_string()))
        .metrics_provider(metrics)
        .grace_period_hours(24)
        .build()?);

    // Initial lease check & activate
    drm.perform_heartbeat().await?;

    // Start background lease daemon
    drm.clone().start_background_daemon();

    // Verify lease is active before accessing sensitive resources
    if drm.is_active().await {
        println!("Application licensed and operational!");
    }

    Ok(())
}
```
