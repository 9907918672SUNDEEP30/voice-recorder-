# 📋 Product Requirements Document (PRD): Audio DSP Recorder & Tone Cloner (`dsp_recorder`)

## 1. Executive Summary & Problem Statement
In professional audio production, podcasting, voiceover recording, and broadcast media, creators frequently struggle with substandard microphone acoustics, room flutter echo, HVAC low-frequency hum, and complex, latency-heavy digital audio workstation (DAW) plugins. Existing consumer recording software introduces high buffer latencies (>50ms), proprietary cloud subscriptions, and erratic phase distortion.

**`dsp_recorder`** is a standalone, 100% native Rust, zero-latency desktop and embedded audio processing appliance:
1. **Zero-Latency Real-Time Audio Capture (`cpal`):** Direct hardware capture with micro-buffer sizes (128/256 frames, $<5.3\text{ms}$ buffer latency) in a lock-free, zero-allocation audio callback thread.
2. **Professional IIR Biquad Filter Studio:** Direct Form II Transposed filter cascade (Robert Bristow-Johnson Audio EQ Cookbook) featuring 35Hz Rumble HPF, 110Hz Warmth Shelf, 400Hz Boxiness Peaking, 3400Hz Intelligibility Presence, 9500Hz Studio Air Shelf, and analog soft-saturation.
3. **Automated FFT Spectral Tone Matching (`rustfft`):** 2048-point Hann-windowed FFT extracting 7-band spectral energy profiles from reference audio files to automatically compute and apply matching target EQ curves.
4. **Lossless Broadcast WAV Serialization (`hound`):** Decoupled background disk I/O supporting 16-bit, 24-bit PCM, and 32-bit float at 44.1kHz/48kHz with guaranteed atomic header finalization on interrupt.

---

## 2. Target Market & User Personas

| Target Segment | Typical Deployments | Key Pain Points Solved |
|---|---|---|
| **Podcasters & Content Creators** | Home studios, untreated rooms, remote interviews. | Sub-audible room rumble, lack of vocal warmth, digital clipping distortion. |
| **Broadcast & Voiceover Studios** | Dedicated voice booths, field reporting rigs. | Consistency across different microphones via spectral tone cloning without DAW bloat. |
| **Embedded Speech Appliances & Kiosks** | Call center stations, smart intercoms, IoT audio hubs. | Minimal CPU footprint (<2%), ultra-low latency audio conditioning prior to STT/VAD. |
| **Acoustic Engineers & Audio Archivists** | Field recording, archival digitization stations. | Uncompressed, bit-perfect 24-bit/32-bit WAV serialization with instant spectral analysis. |

---

## 3. Core Functional Requirements

### 3.1 Hardware Stream Capture & Buffering
- **Device Enumeration:** Cross-platform audio device discovery via WASAPI (Windows), ALSA/PulseAudio/PipeWire (Linux), and CoreAudio (macOS).
- **Format Flexibility:** Auto-negotiation of sample rates (44.1kHz / 48kHz / 96kHz) with automatic channel averaging to normalized 32-bit floating-point mono ($[-1.0, 1.0]$).
- **Lock-Free Pipeline:** Decoupled audio callback communicating with processing pipeline via `crossbeam-channel` bounded rings.

### 3.2 Real-Time DSP Audio Conditioning
- **Direct Form II Transposed Biquad Cascade:**
  - 35 Hz High-Pass Filter ($Q=0.707$) eliminating handling vibrations and AC hum.
  - 110 Hz Low-Shelf Filter for vocal chest resonance and proximity effect compensation.
  - 400 Hz Peaking Filter ($Q=1.2$) for resolving nasal or muddy boxiness.
  - 3400 Hz High-Mid Peaking Filter for crisp consonant clarity.
  - 9500 Hz High-Shelf Filter for velvety high-frequency studio air.
- **Analog Soft-Saturator:** Tanh-style continuous soft clipper preventing harsh digital square-wave clipping when transients exceed 0 dBFS.
- **Parameter Smoother:** Single-pole exponential coefficient smoother eliminating zipper noise and audio clicks during live adjustment.

### 3.3 Fast Fourier Transform (FFT) Tone Cloner
- **Forward FFT:** 2048-point radix-2 transform with Hann windowing for optimal frequency resolution.
- **7-Band Acoustic Envelope:** Sub-Bass (<60Hz), Bass (60-250Hz), Low-Mids (250-500Hz), Mids (500-2kHz), High-Mids (2-4kHz), Presence (4-8kHz), and Air (>8kHz).
- **Acoustic Delta Engine:** Computes spectral variance against reference WAV files and generates optimal Biquad filter compensation gains.

### 3.4 Speech Activity Gating & Serialization
- **VAD Integration:** Integrated with `crates/core` Silero VAD local ONNX model for optional voice-activity gating.
- **Decoupled WAV Writer:** Background disk writer with 128KB chunk buffers ensuring zero dropped frames during disk stalls.

### 3.5 24-Hour Cryptographic Lease, OTA Auto-Updater & Metered Recording
- **24-Hour Cryptographic Lease:** Integrated with `crates/drm_guard` for 24-hour cryptographic subscription lease verification.
- **Volatile RAM Protection:** If subscription payment is stopped, ephemeral session keys are immediately zeroized, locking studio recording.
- **OTA Binary Auto-Updater:** Checks for updates during heartbeats, downloading and atomically swapping binaries.
- **Metered Usage Telemetry:** Reports `DspRecorderMetrics` (`recording_seconds`, `session_count`) during 24-hour check-ins.

---


## 4. Platform & Target Environments (Desktop & Mobile App)

- **Application Nature:** Native Cross-Platform Desktop & Mobile Application (Runs directly on the user's existing desktop, laptop, or smartphone — no dedicated hardware purchase required).
- **Supported Platforms:**
  - **Desktop:** Windows 10/11 (WASAPI / DirectSound), macOS (CoreAudio), Linux (ALSA / PipeWire).
  - **Mobile:** Android & iOS (via cross-compiled Rust core library bindings).
- **System Footprint:** Ultra-lightweight native Rust binary (< 30 MB RAM, < 2% CPU overhead). Runs smoothly on any standard PC, laptop, or phone.
- **Audio Inputs:** Uses the user's existing device microphone, 3.5mm headset jack, or USB audio interface.
- **Delivery Form Factor:** Standalone desktop application (`dsp_recorder.exe` / `dsp_recorder`) and mobile app package.


