# 🛡️ AI Proactive CCTV Guard (`apps/cctv_guard`) - Architecture Specification

## 1. High-Level Vision & Philosophy
Conventional CCTV systems are passive recorders ("मूकदर्शक") that merely capture crime after it has occurred.
The **AI CCTV Guard** (`cctv_guard`) is an autonomous, proactive, edge-first sentry system built with Pure Rust.
It utilizes a **Two-Tier Cost-Saving Architecture** designed for resource-constrained edge hardware (ARM, Raspberry Pi, or CPU-only VPS):

* **Tier 1 (Gatekeeper - Edge Trigger):** A continuous, zero-latency (< 5% CPU) background service consuming RTSP streams, downscaling frames to 160x120 grayscale, computing rolling background differencing and Structural Dissimilarity (SSIM), and evaluating hysteresis and cooldown timers.
* **Tier 2 (VLM Analyzer - Heavy Engine):** A 2B-parameter Qwen Vision-Language Model (quantized 4-bit ONNX) executed via ONNX Runtime (`ort`). It remains completely idle or unloaded until Tier 1 explicitly trips an anomaly trigger, eliminating unnecessary cloud or compute costs.

```
+-------------------------------------------------------------------------------------------------+
|                                    CCTV GUARD ENGINE (Pure Rust)                                |
+-------------------------------------------------------------------------------------------------+
                                                 |
                                    (Live RTSP / Network Stream)
                                                 v
+-------------------------------------------------------------------------------------------------+
|                                TIER 1: EDGE GATEKEEPER (< 5% CPU)                               |
|                                                                                                 |
|  +-------------------------+     +--------------------------+     +--------------------------+  |
|  |  Downsampler (Rec.601)  | --> | Rolling Frame Difference | --> | Structural Dissimilarity |  |
|  |   1080p -> 160x120 Grayscale  |   Adaptive Background    |     |      (Block-based SSIM)  |  |
|  +-------------------------+     +--------------------------+     +--------------------------+  |
|                                                |                                                |
|                                                v                                                |
|                                  +--------------------------+                                   |
|                                  |   Hysteresis & Cooldown  |                                   |
|                                  |   Trigger Filter (>0.04) |                                   |
|                                  +--------------------------+                                   |
+-------------------------------------------------------------------------------------------------+
                                                 |
                                  (Trigger Signal + Event Frame Buffer)
                                                 v
+-------------------------------------------------------------------------------------------------+
|                               TIER 2: QWEN VLM ANALYZER (Heavy Engine)                          |
|                                                                                                 |
|  +-------------------------------------------------------------------------------------------+  |
|  |                                VLM Lifecycle Manager                                      |  |
|  |                    (On-Demand Loading & Automatic RAM Reclaim after Idle)                 |  |
|  +-------------------------------------------------------------------------------------------+  |
|                                                |                                                |
|                                                v                                                |
|  +-------------------------------------------------------------------------------------------+  |
|  |                          Qwen2-VL 2B (Quantized 4-bit ONNX via ORT)                       |  |
|  |                  Input: [1, 3, 384, 384] Normalized ImageNet Tensor                       |  |
|  |                  Output: ThreatLevel (Safe / Suspicious / Critical)                       |  |
|  |                          ThreatCategory (Intruder / WeaponThreat / Package / Animal)       |  |
|  +-------------------------------------------------------------------------------------------+  |
+-------------------------------------------------------------------------------------------------+
                                                 |
                                                 v
+-------------------------------------------------------------------------------------------------+
|                             STORAGE & STRUCTURED EVIDENCE ARCHIVAL                              |
|                                                                                                 |
|  - Snapshot JPEG Compression (`image` crate @ quality 85)                                       |
|  - Append-Only JSON Lines Logging (`incidents.jsonl` with ISO timestamps & confidence)          |
|  - Millisecond Real-Time Telemetry Console Output                                               |
+-------------------------------------------------------------------------------------------------+
```

---

## 2. Component Blueprint

### A. RTSP Ingestion & Preprocessing (`src/rtsp/`)
* **`CctvFrame`:** Interleaved 24-bit RGB contiguous buffer with nanosecond/millisecond timestamps.
* **`DownscaledFrame`:** 160x120 single-channel 8-bit grayscale representation computed via integer-arithmetic Rec.601 luma conversion: $Y = (299R + 587G + 114B + 500) / 1000$.
* **`RtspClient`:** Native TCP socket client handling RTSP handshakes (`OPTIONS`, `DESCRIBE`, `SETUP`, `PLAY`) and interleaved RTP transport, plus deterministic bank scenario generator (`SimulatorScenario`).

### B. Tier-1 Edge Gatekeeper (`src/tier1/`)
* **`FrameDifferencer`:** Adaptive background model with continuous learning rate ($\alpha = 0.04$). Thresholds intensity deltas at $\Delta L > 14$ and detects global illumination surges (light switches).
* **`StructuralDissimilarity`:** 8x8 block-based SSIM computing local luminance mean ($\mu$), contrast ($\sigma$), and covariance ($\sigma_{xy}$), yielding structural dissimilarity $DSSIM = (1 - SSIM) / 2$.
* **`Tier1Gatekeeper`:** Dual-threshold hysteresis ($T_{high} = 0.04, T_{low} = 0.015$), pre-event frame ring buffer (10 frames), and cooldown timer (e.g. 15.0 seconds).

### C. Tier-2 Qwen VLM Analyzer (`src/tier2/`)
* **`QwenVlmEngine`:** Quantized 4-bit ONNX runtime execution on ORT (`ort 2.0.0-rc.13` with `GraphOptimizationLevel::Level3`). Normalizes inputs with ImageNet constants and extracts threat classifications.
* **`VlmLifecycleManager`:** Lazy on-demand loader that holds model session in standby and unloads after configurable inactivity timeout (default 60 seconds) to conserve RAM.

### D. Evidence Archival & Logging (`src/storage/`)
* **`EvidenceVault`:** Compresses evidence frames to JPEG on disk (`evidence/incident_<timestamp>_<threat>.jpg`) and appends structured JSON lines to `evidence/incidents.jsonl`.
