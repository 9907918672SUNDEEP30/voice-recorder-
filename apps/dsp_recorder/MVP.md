# 🚀 Minimum Viable Product (MVP) Roadmap
## Audio DSP Recorder & Tone Cloner Studio (`apps/dsp_recorder`)

---

## 📌 Executive MVP Scope

The **Minimum Viable Product (MVP)** provides a zero-latency, professional-grade audio processing, recording, and acoustic tone cloning engine built in 100% pure Rust. Designed for podcasters, voice artists, and content creators, it runs as a native cross-platform application on standard desktop computers, laptops, and mobile devices (Windows, macOS, Linux, Android, iOS) with sub-5.3ms latency, < 30 MB RAM usage, and zero dedicated hardware purchase required.

---

## 🗺️ Multi-Phase Implementation Roadmap

```mermaid
gantt
    title DSP Recorder MVP Implementation Timeline
    dateFormat  YYYY-MM-DD
    section Phase 1: Hardware Capture Engine
    cpal Audio Host & Hardware Device Enumeration               :active, p1_1, 2026-10-17, 1d
    Lock-Free Ring Buffer & Normalized 32-bit Float Pipeline    :p1_2, after p1_1, 1d
    section Phase 2: Studio DSP Filter Cascade
    Direct Form II Transposed Biquad Filter Engine (RBJ EQ)    :p2_1, after p1_2, 1d
    Single-Pole Exponential Parameter Smoother & Soft Clipper  :p2_2, after p2_1, 1d
    section Phase 3: FFT Spectral Tone Cloner
    rustfft 2048-pt Hann Window Spectral Analyzer             :p3_1, after p2_2, 1d
    7-Band Acoustic Envelope Extractor & Target EQ Calculator  :p3_2, after p3_1, 1d
    section Phase 4: Storage & Appliance Packaging
    hound Async WAV Serialization (16/24-bit PCM, 32-bit float):p4_1, after p3_2, 1d
    Live ASCII VU Meter & Production CLI App                  :p4_2, after p4_1, 1d
```

---

## 🧪 MVP Verification & Test Matrix

| Component | Target Verification | Expected Pass Criteria |
| :--- | :--- | :--- |
| **Biquad Filters** | RBJ Audio EQ math stability test across sweeps | Finite coefficients; zero numerical blowup or NaN outputs. |
| **Filter Processing** | 1 kHz sine wave filtering & harmonic shaping | Attenuation and boost precisely match filter curve; zero phase tearing. |
| **Parameter Smoother** | Step gain change from 0 dB to +12 dB | Exponential convergence across buffer windows; zero audible zipper noise. |
| **FFT Tone Cloner** | 7-band spectral analysis on reference vocal | Frequency bounds cleanly separated; delta gains computed accurately. |
| **WAV Serialization** | 24-bit PCM lossless write & readback test | Byte-for-byte fidelity preserved; valid WAV header generated. |
| **Hardware Envelope** | Live recording on minimal 2 GB RAM system | Memory usage strictly $<30\text{ MB}$; CPU load $<2\%$. |

---

## 🏁 MVP Launch Acceptance Criteria
1. All 6 automated tests in `tests/dsp_tests.rs` pass with **100% green status**.
2. Zero dropped audio samples or buffer underruns at 128/256 sample buffer frames.
3. Clean compilation: `cargo check -p dsp_recorder` with **0 errors**.
4. Single static binary execution requiring no external DLLs or Python runtime.
