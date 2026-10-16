# 🚀 Minimum Viable Product (MVP) Roadmap
## Proactive AI CCTV Guard Appliance (`apps/cctv_guard`)

---

## 📌 Executive MVP Scope

The **Minimum Viable Product (MVP)** delivers an edge-native, two-tier AI CCTV security appliance built in 100% pure Rust for our turnkey Black-Box units. It operates 24/7 on ultra-low-cost hardware (Minimal Dual-Core/Celeron CPU, 2 GB RAM, SSD) without discrete GPUs or cloud dependencies, continuously filtering multi-channel RTSP streams at <5% CPU, transmitting **10-minute proactive health telemetry pings** (triggering WhatsApp alerts on loose cables or offline cameras), protected by **24-hour cryptographic subscription leases** and automated OTA binary updates.

---

## 🗺️ Multi-Phase Implementation Roadmap

```mermaid
gantt
    title CCTV Guard MVP Implementation Timeline
    dateFormat  YYYY-MM-DD
    section Phase 1: Tier-1 Edge Gatekeeper
    RTSP Stream Ingestion & Luma Grayscale Downscaler (160x120) :active, p1_1, 2026-10-17, 1d
    Adaptive Rolling Background Modeling & DSSIM Dissimilarity  :p1_2, after p1_1, 1d
    section Phase 2: Tier-2 On-Demand Vision
    Quantized VLM Adapter Integration (DirectML/CPU SIMD)      :p2_1, after p1_2, 1d
    Threat Level Classifier (Safe / Suspicious / Critical)     :p2_2, after p2_1, 1d
    section Phase 3: Actuation & Evidence Vault
    Timestamped Incident Snapshot & Append-Only JSONL Audit     :p3_1, after p2_2, 1d
    Emergency Siren / Telegram / Webhook Dispatcher            :p3_2, after p3_1, 1d
    section Phase 4: Proactive Telemetry & 24h DRM
    10-Min Health Telemetry Ping (Loose Wire & Offline Cam)    :p4_1, after p3_2, 1d
    Cloud WhatsApp Alert Webhook & 24-Hour Cryptographic Lease :p4_2, after p4_1, 1d
    OTA Binary Auto-Updater with Atomic Swap                   :p4_3, after p4_2, 1d
```

---

## 🧪 MVP Verification & Test Matrix

| Component | Target Verification | Expected Pass Criteria |
| :--- | :--- | :--- |
| **RTSP Gatekeeper** | Multi-camera 1080p RTSP stream processing | CPU utilization strictly $<5\%$; memory baseline $<80\text{ MB}$; 0 dropped frames. |
| **DSSIM Motion Filter** | Distinguish subtle human motion from rain/shadows | Zero false triggers on shadows or headlights; $>98\%$ detection on physical breaches. |
| **On-Demand VLM** | Awakens local model only when motion gate trips | Cold-to-inference latency $<800\text{ ms}$; outputs structured JSON classification. |
| **10-Min Health Ping** | Disconnected camera or physical wire disconnect | Pings Cloudflare every 10 mins; dispatches WhatsApp notification to owner in $<5\text{s}$. |
| **24-Hour DRM Lease** | Expired lease / non-payment simulation | Session key zeroized in RAM; AI detection halts; prompt for renewal. |
| **OTA Auto-Updater** | Version bump and update deployment | Verifies SHA-256 hash; atomic binary swap completes seamlessly. |

---

## 🏁 MVP Launch Acceptance Criteria
1. Full workspace compilation: `cargo check -p cctv_guard` with **0 errors**.
2. Operates seamlessly on **Minimal CPU, 2 GB RAM, and SSD** with ultra-low power consumption.
3. 10-minute proactive telemetry reliably detects camera drops and dispatches WhatsApp alerts.
4. 24-hour cryptographic subscription leasing fully integrated with `crates/drm_guard`.
