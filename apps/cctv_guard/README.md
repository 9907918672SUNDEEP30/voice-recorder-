# 🛡️ AI Proactive CCTV Guard (`cctv_guard`)

A high-performance, edge-first **Two-Tier Proactive CCTV Guard** built with Pure Rust and ONNX Runtime (`ort`).
Instead of acting as a passive recorder, `cctv_guard` continuously monitors live RTSP feeds at ultra-low CPU overhead (< 5% CPU), gates anomalies via rolling frame differencing and structural similarity (SSIM), and triggers an on-demand 2B-parameter Qwen Vision-Language Model (4-bit ONNX) to assess security threats in real time.

---

## ⚡ Quick Start

### 1. Run Built-In Bank Robbery Simulation
Run a simulated bank shutter break-in scenario where a masked intruder attempts forced entry:
```bash
cargo run -p cctv_guard -- --simulate shutter-robbery --camera-id cam_bank_shutter_01
```

### 2. Run Other Simulation Scenarios
```bash
# Armed daytime robbery with brandished firearm
cargo run -p cctv_guard -- --simulate armed-robbery

# Legitimate package delivery (classified as Safe, no false-alarm panic)
cargo run -p cctv_guard -- --simulate delivery

# Stray animal crossing perimeter (classified as Safe)
cargo run -p cctv_guard -- --simulate animal

# Normal quiet bank lobby
cargo run -p cctv_guard -- --simulate quiet
```

### 3. Connect to a Physical IP Camera (Live RTSP)
```bash
cargo run -p cctv_guard -- \
  --rtsp-url "rtsp://admin:password@192.168.1.100:554/live" \
  --camera-id cam_bank_vault_01 \
  --motion-thresh 0.04 \
  --cooldown-secs 15.0 \
  --evidence-dir evidence/
```

---

## 🏗️ Architecture & Features

1. **Tier-1 Edge Gatekeeper (< 5% CPU):**
   - Ingests high-resolution RTSP frames and immediately downscales them to 160x120 grayscale via fast integer Rec.601 luma conversion.
   - Computes rolling background subtraction with adaptive learning rate ($\alpha = 0.04$) and spatial bounding boxes.
   - Computes 8x8 block-based Structural Dissimilarity ($DSSIM = (1 - SSIM) / 2$).
   - Dual-threshold Hysteresis ($T_{high} = 0.04, T_{low} = 0.015$) and cooldown timer suppression.

2. **Tier-2 Qwen VLM Analyzer (Heavy Engine):**
   - 2B-parameter Qwen Vision-Language Model (quantized 4-bit ONNX) loaded via `ort 2.0.0-rc.13`.
   - On-demand dynamic loading: remains completely idle or unloaded until Tier 1 fires an anomaly trigger.
   - Automatically unloads after 60 seconds of inactivity to conserve RAM on resource-constrained devices.
   - Classifies threat levels (`Safe`, `Suspicious`, `Critical`) and categories (`Intruder`, `WeaponThreat`, `PackageDelivery`, `Animal`, `FalseAlarm`).

3. **Evidence Storage & Logging:**
   - Saves evidence JPEG snapshots under `evidence/incident_<timestamp>_<threat>.jpg`.
   - Appends structured JSON records to `evidence/incidents.jsonl`.

---

## 🧪 Running Tests
```bash
cargo test -p cctv_guard
```
