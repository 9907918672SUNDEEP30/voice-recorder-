# 🏛️ Universal DRM Guard Architecture (`crates/drm_guard`)

## 1. High-Level System Architecture

```text
+-------------------------------------------------------------------------------+
|                            APPLICATIONS (`apps/*`)                            |
|       cctv_guard   |   voice_agent   |   local_rag_engine   |   financial_engine  |
+-------------------------------------------------------------------------------+
                                      │
                                      ▼
+-------------------------------------------------------------------------------+
|                  UNIVERSAL DRM GUARD CRATE (`crates/drm_guard`)               |
|                                                                               |
|  +----------------------------+             +-------------------------------+  |
|  |     Hardware Binder        |             |   Anti-Tamper Monotonic Clock |  |
|  |  SHA256(MB || CPU || MAC)  |             |      Instant + Drift Check    |  |
|  +----------------------------+             +-------------------------------+  |
|               │                                              │                |
|               └───────────────────────┬──────────────────────┘                |
|                                       ▼                                       |
|  +--------------------------------------------------------------------------+ |
|  |                        DRMGuard Coordinator Facade                       | |
|  |            - Manages InnerDrmState (Active, Expired, Revoked)            | |
|  |            - Pulls Usage Metrics via `UsageMetricProvider` Trait          | |
|  |            - Enforces 24-Hour Rotating Leases & Grace Windows            | |
|  +--------------------------------------------------------------------------+ |
|               │                                              │                |
|               ▼                                              ▼                |
|  +----------------------------+             +-------------------------------+  |
|  |    Volatile Session Key    |             |    Tokio Background Daemon    |  |
|  |  zeroize::ZeroizeOnDrop    |             |    Interval Tick Heartbeat    |  |
|  |  AES-256-GCM Cipher        |             |    Auto-Wipe on Revocation    |  |
|  +----------------------------+             +-------------------------------+  |
+-------------------------------------------------------------------------------+
                                      │
                                      ▼ HTTPS POST /api/heartbeat
+-------------------------------------------------------------------------------+
|                     CLOUDFLARE EDGE LICENSING WORKER                          |
|         - Registry / KV Database verification of Client ID & Key             |
|         - Hardware UUID validation (rejects device spoofing)                 |
|         - Metered tier validation (camera caps, minute quotas)               |
|         - Cryptographic 24-hour lease token generation                       |
+-------------------------------------------------------------------------------+
```

---

## 2. Security Invariants & Principles

1. **Zero Plaintext Key Storage:**
   The AES-256 decryption key is generated dynamically from the valid lease and held in memory via `zeroize::ZeroizeOnDrop`. No key file is stored on disk.
2. **Deterministic Hardware Lock:**
   A client license is bound strictly to physical hardware. Cloning a virtual machine or moving an SSD to another motherboard results in a hash mismatch and immediate locking.
3. **Rollback Resilience:**
   The clock guard compares wall-clock time with monotonic intervals. Rewinding the system clock to trick expiration will trigger instant revocation.
4. **App-Agnostic Extensibility:**
   Any app can implement `UsageMetricProvider` without modifying the core licensing pipeline.
