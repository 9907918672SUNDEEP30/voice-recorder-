# 🚀 Minimum Viable Product (MVP) Plan
## Project: Commercial AI CCTV / NVR Hardware Appliance (`nvr_core`)

---

### 1. MVP Objective
To build and validate a functional, end-to-end commercial prototype of the **Rust AI CCTV NVR Core** running within WSL (Windows Subsystem for Linux) and edge hardware, capable of:
1. Ingesting multiple video feeds (simulated via movie/CCTV video files and real RTSP).
2. Writing zero-copy rolling MP4 recordings with disk FIFO cleanup.
3. Rendering a responsive multi-camera monitor grid.
4. Gating anomalies via the Tier-1 Sentinel and classifying threats with a fast 50–100MB vision model.
5. Triggering evidence lock and alerts via the Actuator.
6. Enforcing the 24-hour rotating cryptographic lease token.

---

### 2. Milestone Breakdown

```
  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
  │  Milestone  │ ──► │  Milestone  │ ──► │  Milestone  │ ──► │  Milestone  │
  │      1      │     │      2      │     │      3      │     │      4      │
  │ Core Stream │     │ Dual-Tier AI│     │ Display Grid│     │ Licensing & │
  │ & Recording │     │ & Actuator  │     │  & Preview  │     │ Hardware OS │
  └─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

#### 📍 Milestone 1: Ingestion & Zero-Copy Storage Pipeline (Days 1–5)
- [ ] Scaffold `apps/nvr_core` package and dependencies in Cargo workspace.
- [ ] Implement RTSP / video file stream listener using `ffmpeg-next`.
- [ ] Implement zero-copy MP4 rolling writer (15-minute chunking without re-encoding).
- [ ] Implement FIFO disk space manager (auto-prune oldest chunks when disk > 90%).
- [ ] **Validation Test:** Feed 1 hour of continuous movie/CCTV video footage into the pipeline. Verify zero dropped frames, zero-copy CPU utilization < 2%, and intact MP4 files playable in VLC.

#### 📍 Milestone 2: Dual-Tier Intelligence & Actuator (Days 6–10)
- [ ] Port the lightweight Tier-1 Gatekeeper (160x120 downscale + DSSIM + rolling background diff).
- [ ] Integrate a 50–100 MB quantized INT8 Edge Vision Model (e.g. YOLOv8n / MobileNetV4) via `ort` (ONNX Runtime CPU).
- [ ] Build the Actuator worker:
  - Event metadata packaging (JSON).
  - Evidence clip quarantine (locking the 30s pre-event + 60s post-event MP4).
  - Snapshot extraction (JPEG).
  - Simulated relay trigger (logging GPIO actuation).
- [ ] **Validation Test:** Play a movie clip containing a quiet room that suddenly has an intruder / break-in. Verify that the sentinel remains asleep during quiet frames, fires within 50ms upon breach, and produces a locked incident file.

#### 📍 Milestone 3: Multi-Camera HDMI Display Grid (Days 11–15)
- [ ] Implement multi-camera grid layout compositor (2x2, 3x3, 4x4 matrix).
- [ ] Sub-stream hardware decoding to GPU textures (VAAPI / OpenGL / wgpu).
- [ ] Render dynamic HUD: camera labels, timestamps, and flashing red alert frames during active incidents.
- [ ] **Validation Test:** Run 8 concurrent simulated streams in WSL/Linux. Verify smooth 30–60 FPS display rendering with < 300 MB total RAM usage.

#### 📍 Milestone 4: Anti-Theft Licensing & Turnkey Appliance Lockdown (Days 16–20)
- [ ] Implement Hardware Machine Fingerprint generation (SHA-256 of motherboard UUID, CPU ID, MAC).
- [ ] Implement 24-hour rotating lease client (Ed25519 signature validation).
- [ ] Create Cloudflare Worker + Supabase schema for tenant subscription validation.
- [ ] Graceful lockout state: when unlicensed or expired, recording stops and monitor displays renewal QR code.
- [ ] Produce production build flags (`opt-level = 3`, `lto = "fat"`, `strip = true`).

---

### 3. Simulation & WSL Testing Suite

To ensure development does not depend on physical IP cameras in early stages:
1. **Mock Video Feeds:**
   - Script to spin up a local RTSP server (e.g. via `mediamtx` or FFmpeg loop) streaming movie clips and sample surveillance files.
2. **Scenario Test Matrix:**
   - `quiet_room.mp4`: Tests false alarm suppression and low CPU idling.
   - `shutter_break_in.mp4`: Tests perimeter breach detection and rapid alarm response.
   - `daytime_delivery.mp4`: Tests benign motion classification (no siren false alarm).
   - `night_intruder.mp4`: Tests low-light silhouette detection.
3. **Hardware Constraints Stress Test:**
   - Run tests under `systemd-run --scope -p MemoryMax=1.5G` to rigorously ensure the application never exceeds the 2 GB hardware threshold.
