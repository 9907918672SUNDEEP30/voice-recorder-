# 📋 Product Requirements Document (PRD): Proactive AI CCTV Guard (`cctv_guard`)

## 1. Executive Summary & Problem Statement
Traditional CCTV and NVR security systems are **passive forensic tools**: they record crimes on hard drives for post-incident review, but fail to prevent active bank robberies, warehouse break-ins, or perimeter intrusions in real time. Cloud-based video analytics suffer from high bandwidth costs, latency (>5-10s), and privacy violations.

**`cctv_guard`** is a standalone, 100% native Rust, edge-native **Two-Tier Proactive AI Guard Appliance**:
1. **Tier-1 Lightweight Edge Gatekeeper (<5% CPU):** Continuously monitors multiple RTSP video streams at native framerates, downscaling frames to 160x120 grayscale luma, running adaptive rolling background subtraction and 8x8 block Structural Dissimilarity ($DSSIM$).
2. **Tier-2 On-Demand Qwen VLM Reasoning Engine:** Awakens a local 2B-parameter quantized Vision-Language Model on DirectML/CUDA only when physical anomalies occur, classifying threat levels (`Safe`, `Suspicious`, `Critical`) in under 800ms, dispatching siren triggers, Telegram/WhatsApp alerts, and saving encrypted evidence.

---

## 2. Target Market & User Personas

| Target Segment | Typical Deployments | Key Pain Points Solved |
|---|---|---|
| **Banks & ATM Kiosks** | Night ATM vestibules, vault cash-counter shutters. | Shutter forced entry, masked intruders, loitering after hours, real-time siren dispatch. |
| **Jewelry & Retail Stores** | Showrooms, back-office cash safes. | Daylight armed robberies, glass breaking, brandished weapons detection. |
| **Industrial Warehouses** | Perimeter fences, loading bays, forklift lanes. | Night trespassers, theft of raw materials, filtering out stray animals/rain false alarms. |
| **Gated Communities & Estates** | Main entrance gates, remote perimeter walls. | Unauthorized vehicle tailgating, perimeter breach without expensive cloud subscription fees. |

---

## 3. Core Functional Requirements

### 3.1 Tier-1 Motion & Structural Gatekeeper
- **RTSP Ingestion:** Native TCP/UDP RTSP stream demuxing via `ffmpeg`/`nokhwa` without memory leaks.
- **Luma Optimization:** Rec.601 integer conversion ($Y = (R \times 77 + G \times 150 + B \times 29) \gg 8$) downscaled to 160x120.
- **Adaptive Background Modeling:** Rolling pixel mean with learning rate $\alpha = 0.04$ and spatial bounding box extraction.
- **Hysteresis Thresholding:** Dual-threshold triggers ($T_{high} = 0.04$, $T_{low} = 0.015$) to prevent flapping.
- **Cooldown Timer:** Configurable cooldown period (default 15s) per camera to suppress repetitive alerts.

### 3.2 Tier-2 On-Demand Qwen Vision Reasoning
- **Zero Idle Overhead:** VLM weights remain cold or unloaded when no threat is detected.
- **Threat Categorization:**
  - `Critical`: Brandished weapon, masked intruder prying shutter, physical violence.
  - `Suspicious`: Unattended backpack, loitering past curfew, covered camera lens.
  - `Safe`: Legitimate postal courier delivery, pet/animal crossing, wind/shadows.
- **Evidence Archival:** Automated timestamped JPEG snapshots saved under `evidence/` alongside structured append-only audit trail in `evidence/incidents.jsonl`.

### 3.3 Hardware Appliance Licensing & DRM
- Integrated with `crates/drm_guard` for 24-hour rotating cryptographic lease verification, machine UUID binding, and metered RTSP camera limits.

---

## 4. Hardware & Deployment Specifications (Ultra-Low Budget Edge Hardware)

- **Minimum Hardware Specs (Kam se Kam Kharcha / Ultra-Budget Edge Box):**
  - **CPU:** Minimal / Entry-level Low-Power CPU (Intel Celeron / N100 / Dual-Core x86_64, or ARM Cortex-A53/A72 SoC like Orange Pi / Raspberry Pi).
  - **RAM:** **2 GB RAM** (Native Rust engine uses <50MB–120MB baseline; zero garbage collector, zero Python overhead).
  - **Storage:** 16 GB – 64 GB SSD / eMMC (Fast solid-state I/O for OS, local binary, and incident evidence storage).
  - **Target Device Cost:** Ultra-low Capex (~₹3,000 – ₹7,000 / $35 – $80 thin-client / mini-PC box).
- **Operating Systems:** Windows 10/11 IoT / Home (DirectML / CPU SIMD) & Linux (Ubuntu Server 20.04/22.04 LTS, Debian Minimal, or Alpine Linux).
- **Deployment Form Factors:**
  - **Windows Service:** Auto-starts on OS boot via NSSM / Windows Service Manager with auto-restart on crash.
  - **Linux Systemd Service:** Headless edge server daemon with resource cgroups (<15% CPU quota).
