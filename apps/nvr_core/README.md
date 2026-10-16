# 🛡️ Shambhavi AI Labs — Pehredar Box (`nvr_core`)
> **Commercial AI CCTV & NVR Hardware Appliance Engine**  
> *Apple-Style Closed Ecosystem • Headless Linux Native • 100% Pure Rust Performance*

---

## 🌟 Executive Overview

**Pehredar Box (पहरेदार बॉक्स)** is a commercial-grade, hardware-locked, turnkey Network Video Recorder (NVR) and Edge AI Security Appliance developed by **Shambhavi AI Labs**.

Built from the ground up in **100% Pure Rust** for **Minimal Headless Linux (Ubuntu Server / Debian / Alpine)** on custom-assembled 2GB RAM industrial black boxes:
1. **Uninterruptible 24/7 CCTV Core:** Ingests 8 to 32 RTSP/ONVIF streams (H.264/H.265) and writes 15-minute rolling MP4 chunks with `< 2%` CPU overhead (0% re-encoding).
2. **Direct HDMI Monitor Grid:** Renders an 8 to 32 camera matrix directly to the local display via native hardware graphics (`minifb` / Linux DRM) with automatic headless auto-detection.
3. **Decoupled Hot-Pluggable AI Layer:** 
   - Powered by centralized **`ai_core::YoloV8Detector`** running the real **YOLOv8m ONNX model** located at `C:\AI_Models\yolov8m.onnx` or `/opt/shambhavi/models/yolov8m.onnx`.
   - **Fault-Tolerant Non-Blocking Architecture:** If the AI layer is turned off, runs out of memory, or crashes, **CCTV recording and HDMI display continue 100% normally without dropping a single frame!**
4. **Physical Actuator Engine ("एक्शन इंजन"):** Clamps 30s pre + 60s post-incident video to NVMe SSD, triggers local 110dB sirens/strobes via GPIO/relays, and dispatches encrypted instant cloud alerts via HTTP Webhook.
5. **Anti-Theft 24-Hour Rotating Cloud Lease:** Hardware machine fingerprinting (Motherboard UUID + CPU ID + MAC) coupled with an Ed25519-signed 24-hour lease token from Cloudflare Workers & Supabase to enforce recurring monthly subscriptions.

---

## 💼 Commercial Business Model & Pricing (Indian Market)

| Service / Hardware Tier | Pricing (INR) | Features & Channels |
| :--- | :--- | :--- |
| **Per-Camera Subscription** | **₹250 / camera / month** | 24/7 AI threat monitoring, zero-cloud storage, hardware warranty |
| **Base Plan Floor** | **₹1,000 / month minimum** | Covers 1 to 4 cameras (ensures high margin baseline) |
| **Mid-Volume Tier (16 Cams)** | **₹250 / camera / month** | 16-channel turnkey setups for showrooms, supermarkets |
| **Enterprise Tier (32 Cams)** | **₹250 / camera / month** | 32-channel high-density setup for factories, warehouses |
| **Complete Turnkey Kit** | **₹29,999 – ₹32,000 one-time** | 4 IP Cameras + Custom Pehredar Box + 1-2TB Surveillance HDD + Full Installation |

---

## ⚙️ Hardware Capacity Tiers (Custom-Assembled Industrial Boxes)

| Tier | Camera Capacity | Target Processor | RAM | Storage Configuration | Deployment Target |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Lite** | **Up to 8 Cameras** | Intel N100 / N200 Quad-Core | 8 GB DDR4/5 | 128 GB NVMe + 1 TB Surveillance SATA | Small stores, homes, clinics, offices |
| **Pro** | **Up to 16 Cameras** | Intel Core i3 / i5 (12th/13th Gen) | 16 GB DDR4/5 | 256 GB NVMe + 2 TB–4 TB SATA | Showrooms, supermarkets, warehouses |
| **Enterprise**| **Up to 32 Cameras** | Intel Core i7 (12th/13th/14th Gen) | 32 GB DDR5 | 512 GB NVMe + 4 TB–8 TB SATA | Large factories, gated societies, malls |
| **Cluster** | **32+ to 64+ Cameras** | 2x Clustered 32-Ch Boxes | Clustered | Redundant Clustered Storage | Zero Single-Point-of-Failure (SPOF) |

---

## 📁 Repository & Codebase Structure

```
ai_core_framework/
├── crates/core/              # Centralized Framework Library (ai_core)
│   ├── src/models/yolo_onnx.rs   # High-speed SIMD YOLOv8 ONNX inference engine
│   └── src/lib.rs            # Re-exports YoloV8Detector & YoloDetection
│
└── apps/nvr_core/            # Commercial CCTV NVR Appliance Engine
    ├── deploy/
    │   ├── pehredar.service  # Systemd daemon auto-start for Linux appliances
    │   └── setup_appliance.sh# Turnkey Ubuntu installation script
    ├── scripts/
    │   └── licensing_worker.js # Cloudflare Worker 24h rotating lease engine
    ├── Cargo.toml            # Depends on ai_core = { path = "../../crates/core" }
    ├── README.md             # Product overview, tiers, and commercial guide (this file)
    ├── PRD.md                # Detailed Product Requirements Document
    ├── ARCHITECTURE.md       # In-depth architectural blueprint & memory budget
    ├── MVP.md                # Minimum Viable Product roadmap & milestones
    ├── WORK_PROGRESS.md      # Real-time task tracker and implementation progress
    └── src/
        ├── main.rs           # Engine coordinator and runtime entry point
        ├── config/           # Appliance & camera configuration models
        ├── ingestion/        # Stream acquisition & H.264 NAL demuxer
        ├── storage/          # Zero-copy rolling MP4 storage & FIFO cleaner
        ├── display/          # Direct HDMI monitor grid & native framebuffer window
        ├── sentinel/         # Tier-1 Edge Gatekeeper ("पहरेदार")
        ├── ai/               # Hot-pluggable AI Security Plugin (calls ai_core)
        ├── actuator/         # Tier-3 Evidence locking, relay siren, and webhooks
        └── security/         # Machine fingerprinting & 24h rotating cloud lease
```

---

## 🚀 Quick Start & Deployment Commands

### 1. Run Complete Pipeline with Real YOLOv8m Model
```bash
cargo run -p nvr_core -- --simulate shutter-break-in --frames 80
```

### 2. Run in Pure CCTV Mode (AI Completely Disabled)
Demonstrates the zero-downtime decoupled architecture. Even if AI is turned off, CCTV recording and HDMI display continue smoothly at 25 FPS without interruption:
```bash
cargo run -p nvr_core -- --simulate shutter-break-in --frames 60 --disable-ai
```

### 3. Deploy to Linux / Ubuntu Server Black Box Appliance
```bash
# 1. Build optimized stripped release binary
cargo build --release -p nvr_core

# 2. Run turnkey appliance setup script
sudo bash ./apps/nvr_core/deploy/setup_appliance.sh

# 3. Verify running service
sudo systemctl status pehredar.service
```

---

## 🔒 Security & Anti-Theft Protection
- **Stripped Pure Rust Binary:** Compiled with `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, and `strip = true`.
- **Machine Fingerprint Lock:** Bound to Motherboard UUID, CPU ID, and primary MAC.
- **24-Hour Rotating Lease:** Mandatory daily cryptographic lease check against Shambhavi AI Labs Cloudflare Worker & Supabase. If unpaid, the system automatically locks and displays a renewal QR code on the HDMI monitor.
