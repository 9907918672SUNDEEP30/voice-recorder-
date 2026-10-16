# 🏛️ System Architecture Document
## Shambhavi AI Labs — Pehredar Box (`nvr_core`)

---

### 1. High-Level Architecture Overview

Pehredar Box is architected as an **Apple-style closed-ecosystem, zero-cloud-cost, 100% Pure Rust NVR Appliance** designed for high thermal efficiency, rock-solid stability, and zero discrete GPU overhead.

```
                           [ 8 to 32 IP Cameras (RTSP/ONVIF) ]
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│             SHAMBHAVI AI LABS — PEHREDAR BOX ENGINE (nvr_core)                         │
│                                                                                        │
│  ┌──────────────────────────────────────────────────────────────────────────────────┐  │
│  │ 1. STREAM INGESTION & DEMUXER (FFmpeg / libavformat bindings)                   │  │
│  │    • Multi-threaded RTSP Stream Ingest (Channels 1 .. N)                         │  │
│  │    • Network Jitter Buffer & Exponential Backoff Reconnection                    │  │
│  │    • Demuxes raw H.264/H.265 NAL Units & Audio Packets (G.711 / AAC)             │  │
│  └──────────┬───────────────────────────────┬───────────────────────────────┬───────┘  │
│             │ (Raw NAL Packets)             │ (Raw NAL Packets)             │          │
│             ▼                               ▼                               ▼          │
│  ┌──────────────────────┐     ┌───────────────────────────┐   ┌─────────────────────┐  │
│  │ 2. STORAGE ENGINE    │     │ 3. DISPLAY ENGINE         │   │ 4. SENTINEL ENGINE  │  │
│  │    (Zero-Copy)       │     │    (Direct HDMI Out)      │   │    ("पहरेदार")      │  │
│  │ • Direct MP4 Muxer   │     │ • HW Video Decoder (VAAPI)│   │ • Keyframe Decoder  │  │
│  │   (No re-encoding)   │     │ • 8/16/32 Matrix Grid     │   │ • 160x120 Luma Grid │  │
│  │ • 15-Min Rolling Segs│     │ • wgpu / DRM Framebuffer  │   │ • DSSIM Differencing│  │
│  │ • FIFO Disk Pruning  │     │ • Threat Alert Red Border │   │ • Gating Filter     │  │
│  └──────────────────────┘     └─────────────┬─────────────┘   └──────────┬──────────┘  │
│                                             │                            │             │
│                                             │ Direct HDMI                │ Trigger     │
│                                             ▼                            ▼             │
│                                    [ Physical Monitor ]       ┌─────────────────────┐  │
│                                                               │ 5. EDGE AI MODEL    │  │
│                                                               │    (50–100MB INT8)  │  │
│                                                               │ • ONNX Runtime CPU  │  │
│                                                               │ • YOLOv8/11 SIMD    │  │
│                                                               │ • Structured JSON   │  │
│                                                               └──────────┬──────────┘  │
│                                                                          │ Verdict     │
│                                                                          ▼             │
│  ┌──────────────────────────────────────────────────────────────────────────────────┐  │
│  │ 6. ACTUATOR ENGINE ("एक्शन इंजन")                                                │  │
│  │    • Evidence Preservation (Locks Pre/Post Event MP4 Clamps to NVMe SSD)        │  │
│  │    • Physical Hardware Triggers (USB-Relay / GPIO Siren / Strobe)                │  │
│  │    • Async Alert Dispatch (Webhook / Telegram / WhatsApp / Cloud API)            │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                        │
│  ┌──────────────────────────────────────────────────────────────────────────────────┐  │
│  │ 7. LICENSING & INTEGRITY GUARDIAN                                                │  │
│  │    • Hardware Machine Fingerprint (CPU ID + Motherboard UUID + MAC)              │  │
│  │    • 24-Hour Rotating Ed25519 Token Lease (Cloudflare Worker + Supabase)         │  │
│  │    • Anti-Tamper / Graceful Lockout if Unlicensed or Subscription Expired        │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 2. Multi-Box Enterprise Cluster Architecture (32+ to 64+ Cameras)

For large industrial plants, malls, and campuses requiring 32 to 64+ cameras, Shambhavi AI Labs deploys a **Multi-Box Clustered Architecture** eliminating any Single Point of Failure (SPOF):

```
┌─────────────────────────────────┐           ┌─────────────────────────────────┐
│     PEHREDAR BOX ALPHA          │           │     PEHREDAR BOX BETA           │
│  (Cameras 1 .. 32 Ingestion)    │◄─────────►│  (Cameras 33 .. 64 Ingestion)   │
│  Dedicated Display 1 (Monitor 1)│   gRPC /  │  Dedicated Display 2 (Monitor 2)│
│  Independent Storage Node 1     │   Shared  │  Independent Storage Node 2     │
│  Local Siren / Actuator 1       │  Lease Bus│  Local Siren / Actuator 2       │
└─────────────────────────────────┘           └─────────────────────────────────┘
```
- If Box Beta experiences a hardware issue, Box Alpha continues recording and monitoring its 32 channels without interruption.
- Displays can mirror or dedicate individual 32-channel matrices across multi-screen control rooms.

---

### 3. Module Hierarchy & Rust Code Structure

The `nvr_core` codebase is organized into isolated, decoupled Rust modules:

```
apps/nvr_core/src/
├── main.rs                   # Engine entry point, CLI parsing, runtime coordinator
├── config/                   # Appliance & camera configuration models
│   ├── mod.rs
│   ├── appliance.rs          # Box identity, tier settings, disk quotas
│   └── camera.rs             # RTSP URLs, credentials, FPS, zones
├── ingestion/                # Stream acquisition & demuxing
│   ├── mod.rs
│   ├── rtsp_client.rs        # Persistent connection loop with exponential backoff
│   └── demuxer.rs            # H.264/H.265 NAL packet extractor
├── storage/                  # Zero-copy disk persistence
│   ├── mod.rs
│   ├── mp4_writer.rs         # 15-minute chunked MP4 container muxer
│   ├── ring_buffer.rs        # In-memory rolling circular buffer for pre-roll clamps
│   └── cleaner.rs            # FIFO disk quota cleaner (>90% trigger)
├── display/                  # Direct HDMI monitor grid
│   ├── mod.rs
│   ├── grid.rs               # 2x2, 3x3, 4x4, 6x6 camera matrix layout
│   └── hud.rs                # On-screen bounding boxes, red flashing borders
├── sentinel/                 # Tier-1 Edge Gatekeeper ("पहरेदार")
│   ├── mod.rs
│   ├── luma.rs               # Ultra-fast integer Rec.601 160x120 downscaler
│   ├── dssim.rs              # 8x8 block structural dissimilarity calculation
│   └── gatekeeper.rs         # Hysteresis trigger & cooldown controller
├── ai/                       # Tier-2 Edge Vision Model
│   ├── mod.rs
│   ├── model.rs              # 50-100MB INT8 ONNX inference engine (YOLOv8/11n)
│   └── parser.rs             # Threat classification & bounding box extractor
├── actuator/                 # Tier-3 Action Engine ("एक्शन इंजन")
│   ├── mod.rs
│   ├── evidence.rs           # Quarantines 30s pre + 60s post MP4 & JPEG
│   ├── relay.rs              # USB/GPIO relay triggers (110dB siren/strobe)
│   └── notifier.rs           # Async Webhook/WhatsApp/Telegram alerts
└── security/                 # Licensing & Anti-Theft Guardian
    ├── mod.rs
    ├── fingerprint.rs        # SHA-256 machine hardware hash generator
    └── lease.rs              # 24-hour rotating Ed25519 token verifier
```

---

### 4. Memory & Hardware Resource Envelope

| Component | RAM Budget (Max) | CPU Usage (8 Ch) | CPU Usage (16 Ch) |
| :--- | :--- | :--- | :--- |
| **Linux Kernel & Base OS (Headless)** | 250 MB | 1–2% | 1–2% |
| **Ingestion & Demuxer (Zero-Copy)** | 120 MB | 2–3% | 4–6% |
| **Zero-Copy MP4 Storage Buffers** | 160 MB | 1% | 2% |
| **HDMI Display Renderer (60 FPS)** | 180 MB | 4–6% | 7–10% |
| **Tier-1 Sentinel Gatekeeper** | 60 MB | 2–3% | 4–5% |
| **Tier-2 AI Model (INT8 ONNX in RAM)**| 110 MB | 15–20% (Burst) | 20–30% (Burst) |
| **Actuator & Licensing Guardian** | 30 MB | < 0.5% | < 0.5% |
| **System Headroom (Free Buffer)** | **1,090 MB Free**| — | — |
| **TOTAL (Lite / Pro Envelope)** | **~910 MB used / 2 GB** | **~10–14% Idle** | **~18–25% Idle** |
