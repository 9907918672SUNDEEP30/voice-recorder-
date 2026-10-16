# 📋 Product Requirements Document (PRD)
## Shambhavi AI Labs — Pehredar Box (Commercial AI CCTV & NVR Appliance)

---

### 1. Executive Summary & Brand Vision
- **Company:** **शांभवी एआई लैब्स (Shambhavi AI Labs)**
- **Product Name:** **Pehredar Box (पहरेदार बॉक्स)**
- **Product Nature:** Apple-style closed-ecosystem, zero-cloud-cost, enterprise-grade edge hardware appliance (Turnkey Black Box).
- **Business Model:** Hardware-as-a-Service (HaaS) + Hardware-Locked Monthly Subscription. It is not an unmanaged desktop or mobile app; it is a dedicated, self-healing, custom-assembled black box appliance.
- **Value Proposition:** 100% Pure Rust NVR Core (`nvr_core`), running on ultra-low resource hardware without GPU requirements, offering direct HDMI multi-camera monitor output, zero-copy 24/7 stream recording, dual-tier edge AI threat gating, physical siren actuation, and 24-hour cryptographic subscription protection.

---

### 2. Commercial Pricing & Business Model (Indian Market Fit)

| Plan / Component | Pricing Model | Commercial Terms |
| :--- | :--- | :--- |
| **Per-Camera Subscription** | **₹250 / camera / month** | Billed monthly via automated e-Mandate / recurring debit. |
| **Minimum Base Floor** | **₹1,000 / month** | Applied for 1 to 4 cameras (ensures high recurring margin). |
| **Mid-Volume Tier (16 Cams)** | **₹250 / camera / month** | 16-channel turnkey setups for showrooms, supermarkets. |
| **Enterprise Tier (32 Cams)** | **₹250 / camera / month** | High-density 32-channel setup for large factories and warehouses. |
| **Turnkey Hardware Kit** | **₹29,999 – ₹32,000 one-time** | Includes: 4 IP cameras, Custom Shambhavi AI Black Box, 1TB-2TB Surveillance HDD, cabling, and full professional installation. |

---

### 3. Hardware Strategy & Capacity Tiers (Custom Assembled Black-Box)

Instead of using non-upgradable ready-made mini-PCs, Shambhavi AI Labs utilizes custom-assembled industrial black-box units to maximize thermal cooling, field repairability, and hardware profit margins:

| Tier | Camera Capacity | Target Processor | RAM | Storage Configuration | Target Customer |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Nano (Ultra-Budget)** | **1 to 4 Cameras** | Minimal CPU (Celeron / Dual-Core / ARM SoC) | **2 GB RAM** | 64 GB – 128 GB SSD | Entry-level shops, home gates, single kiosks (kam se kam kharcha) |
| **Lite** | **Up to 8 Cameras** | Intel N100 / N200 Quad-Core | 4 GB – 8 GB DDR4 | 128 GB SSD + 1 TB Surveillance SATA | Small stores, residences, clinics, small offices |
| **Pro** | **Up to 16 Cameras** | Intel Core i3 / i5 (12th/13th Gen) | 16 GB DDR4/5 | 256 GB NVMe + 2 TB–4 TB Surveillance SATA | Showrooms, supermarkets, schools, mid-size warehouses |
| **Enterprise**| **Up to 32 Cameras** | Intel Core i7 (12th/13th/14th Gen) | 32 GB DDR5 | 512 GB NVMe + 4 TB–8 TB Surveillance SATA | Large factories, gated societies, shopping malls |
| **Multi-Box Cluster** | **32+ to 64+ Cameras** | 2x Clustered 32-Ch Boxes | Clustered | Redundant Clustered Storage | Zero Single-Point-of-Failure (SPOF) enterprise deployments |

---

### 4. Core Software Capabilities (100% Pure Rust Architecture)

#### 4.1 Zero-Copy Stream Ingestion & Rolling MP4 Recording
- **RTSP / ONVIF Multi-Stream Ingestion:** Continuous 24/7 ingestion of H.264 / H.265 streams from up to 8/16/32 IP cameras.
- **0% Re-encoding CPU Overhead:** Video packets are remuxed straight into 15-minute MP4 chunks on the surveillance disk without decoding/re-encoding, keeping CPU usage < 2% total across channels.
- **Automated FIFO Storage Manager:** Automatically purges the oldest non-evidence MP4 segments when disk usage exceeds 90%.

#### 4.2 Direct HDMI Display Grid (No Web/Electron Bloat)
- **Zero-Latency Compositor:** Directly drives the physical monitor over HDMI (1080p/4K 60Hz) using native low-level graphics (`wgpu` / OpenGL / Linux DRM/Wayland) with < 100 MB RAM consumption.
- **Real-Time HUD Overlays:** Renders camera timestamps, stream bitrates, dynamic AI bounding boxes, and high-visibility flashing red emergency borders during active security incidents.

#### 4.3 Two-Tier AI Intelligence Pipeline
- **Tier-1 Sentinel ("पहरेदार"):** Downscaled 160x120 grayscale differencing and DSSIM structural dissimilarity (< 3% CPU). Keeps the system completely relaxed during quiet hours.
- **Tier-2 Edge Vision Model (50–100 MB):** On-demand INT8 quantized ONNX model (YOLOv8/YOLOv11-nano / MobileNetV4) executed via `ort` with AVX2/SIMD instructions, delivering 30–50 ms inference per gated frame on basic CPU without discrete GPU.

#### 4.4 Tier-3 Actuator Engine ("एक्शन इंजन")
- **30s Pre + 60s Post Evidence Locking:** Clamps 30 seconds before and 60 seconds after an incident, quarantining it permanently on the NVMe SSD so FIFO cleanup cannot delete it.
- **Hardware Relays:** Fires local 110dB sirens, strobe lights, or electromagnetic door deadbolts via USB-relay / GPIO.
- **Instant Cloud Alerts:** Dispatches incident snapshots and structured threat alerts over Webhook / WhatsApp / Telegram.

---

### 5. Security, Anti-Theft & 24-Hour Rotating Cloud Lease

#### 5.1 Anti-Tamper & Binary Hardening
- **Stripped Pure Rust Binary:** Compiled with `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, and `strip = true`. No interpretable Python/JS code exists on the device.
- **Machine Hardware Fingerprint:**
  $$\text{MachineKey} = \text{SHA256}(\text{Motherboard UUID} \parallel \text{CPU Serial} \parallel \text{Primary NIC MAC})$$
  Copying the SSD or binary to another computer causes an immediate startup abort.
- **LUKS Storage Encryption:** Hard disk partitions are encrypted. Direct drive removal reveals zero plaintext data.

#### 5.2 24-Hour Rotating Cloud Lease (Cloudflare Worker + Supabase)
- **Cryptographic Heartbeat:** Every 24 hours, the box contacts the Shambhavi AI Labs Cloudflare Worker with its MachineKey.
- **Subscription Validation:** Cloudflare verifies billing status in Supabase.
- **Signed Lease Token:** If the account is in good standing, Cloudflare returns an Ed25519-signed token valid for 24 hours.
- **Automated Graceful Lockout:** If internet is cut permanently or subscription payment fails, after the 24-hour lease expires, recording and AI gracefully lock, and the HDMI screen displays:  
  `"Subscription Expired — Scan QR Code to Renew [Shambhavi AI Labs]"`.
