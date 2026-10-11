# 🏗️ Architecture Blueprint: Pure Rust Audio Recorder & Tone Cloner (`apps/dsp_recorder`)

> **Role & Classification:** Production-Ready, Zero-Latency Embedded/Desktop Audio Processing and Acoustic Tone Cloning Engine.  
> **Host Architecture:** High-performance Rust application built strictly on top of the neutral, sealed **AI Core Framework** (`crates/core`).

---

## 📐 1. System Topology & Data Flow

The engine enforces a strict decoupling between the **Real-Time Audio Hardware Thread**, the **Synchronous DSP Processing Pipeline**, the **AI Inference Core (Silero VAD)**, and the **Asynchronous File I/O Subsystem**.

```
                           ┌──────────────────────────────────────────────┐
                           │   Audio Hardware Input (Mic / USB Audio)     │
                           └──────────────────────┬───────────────────────┘
                                                  │ Low-Latency Stream (128/256 frames)
                                                  ▼
                         ┌──────────────────────────────────────────────────┐
                         │      cpal Audio Hardware Callback Thread         │
                         │   • Real-Time Thread Priority                    │
                         │   • ZERO Heap Allocation in steady state         │
                         │   • Channel Averaging to Mono f32 [-1.0, 1.0]    │
                         └────────────────────────┬─────────────────────────┘
                                                  │ Non-blocking try_send()
                                                  ▼
                         ┌──────────────────────────────────────────────────┐
                         │   Bounded SPSC Channel Buffer (crossbeam: 512)   │
                         └────────────────────────┬─────────────────────────┘
                                                  │ recv_timeout()
                                                  ▼
     ┌────────────────────────────────────────────────────────────────────────────────────────┐
     │                       Central DSP & Audio Pipeline Engine                              │
     │                                                                                        │
     │  ┌───────────────────────────┐                ┌─────────────────────────────────────┐  │
     │  │  AI Core Silero VAD       │                │  Cascaded IIR Biquad Filters        │  │
     │  │  (crates/core in-memory)  │                │  (Direct Form II Transposed)        │  │
     │  │                           │                │                                     │  │
     │  │  1. rubato Sinc Resample  │                │  1. 35 Hz Rumble High-Pass Filter   │  │
     │  │     (48kHz -> 16kHz)      │                │  2. 110 Hz Low-Shelf (Bass Warmth)  │  │
     │  │  2. LSTM State Tensor     │                │  3. 400 Hz Low-Mid (Body Sculpting) │  │
     │  │  3. Speech Prob Gating    │                │  4. 3400 Hz Peaking (Vocal Clarity) │  │
     │  └─────────────┬─────────────┘                │  5. 9500 Hz High-Shelf (Studio Air) │  │
     │                │ Speech Prob                  │  6. Analog-Style Soft Saturator     │  │
     │                └──────────────┬───────────────┴──────────────────┬──────────────────┘  │
     └───────────────────────────────┼──────────────────────────────────┼─────────────────────┘
                                     │                                  │ Processed PCM f32
                                     ▼                                  ▼
                         ┌──────────────────────────────────────────────────┐
                         │    rustfft 2048-pt Spectral Analyzer Engine      │
                         │   • Hann Windowing & Magnitude Spectrum |X[k]|   │
                         │   • 7-Band Acoustic Envelope Extraction          │
                         │   • Dynamic Delta EQ Matching against Reference  │
                         └────────────────────────┬─────────────────────────┘
                                                  │ Streamed PCM
                                                  ▼
                         ┌──────────────────────────────────────────────────┐
                         │       hound Async WAV Disk Writer Thread         │
                         │   • 128 KB Buffered OS File I/O                  │
                         │   • 16-bit / 24-bit / 32-bit Lossless PCM        │
                         │   • Atomic Header Finalization on SIGINT/Stop    │
                         └──────────────────────────────────────────────────┘
```

---

## ⚡ 2. Real-Time Safety & Latency Budget

Audio glitches (buffer under-runs / xruns) occur when the audio callback thread is stalled by memory allocation, OS scheduling, or synchronization locks. This engine guarantees:

1. **Zero Heap Allocation on Audio Thread:**
   - Pre-allocated chunk transfer via fixed-capacity slices.
   - Channel backpressure drops metrics via atomic counters (`metrics.buffer_overflow_drops`) rather than blocking the hardware.
2. **Deterministic Latency Budget:**
   - At $F_s = 48\,000\text{ Hz}$ with $N = 128\text{ samples}$, hardware input latency is:
     $$\tau_{\text{hw}} = \frac{128}{48\,000} \approx 2.67\text{ ms}$$
   - With $N = 256\text{ samples}$:
     $$\tau_{\text{hw}} = \frac{256}{48\,000} \approx 5.33\text{ ms}$$
   - Combined IIR Biquad processing latency is sub-microsecond ($< 0.15\text{ ms}$ on ARM Cortex-A72 / Raspberry Pi 4).

---

## 🎛️ 3. IIR Biquad Mathematical Specifications

Every filter is implemented in **Direct Form II Transposed** representation. This structure exhibits optimal numerical stability in single-precision floating-point arithmetic and prevents limit-cycle oscillations.

### A. Difference Equations
For input $x[n]$, output $y[n]$, and state registers $s_1[n], s_2[n]$:

$$y[n] = b_0 x[n] + s_1[n-1]$$
$$s_1[n] = b_1 x[n] - a_1 y[n] + s_2[n-1]$$
$$s_2[n] = b_2 x[n] - a_2 y[n]$$

### B. Robert Bristow-Johnson Coefficient Formulas
Given corner frequency $f_0$, sample rate $F_s$, quality factor $Q$, and gain in decibels $G_{dB}$:

$$A = 10^{\frac{G_{dB}}{40}}, \quad \omega_0 = 2\pi \frac{f_0}{F_s}, \quad \alpha = \frac{\sin(\omega_0)}{2Q}$$

#### 1. Low-Shelf Filter (110 Hz Deep Vocal Warmth)
$$b_0 = A \left[ (A+1) - (A-1)\cos(\omega_0) + 2\sqrt{A}\alpha \right]$$
$$b_1 = 2A \left[ (A-1) - (A+1)\cos(\omega_0) \right]$$
$$b_2 = A \left[ (A+1) - (A-1)\cos(\omega_0) - 2\sqrt{A}\alpha \right]$$
$$a_0 = (A+1) + (A-1)\cos(\omega_0) + 2\sqrt{A}\alpha$$
$$a_1 = -2 \left[ (A-1) + (A+1)\cos(\omega_0) \right]$$
$$a_2 = (A+1) + (A-1)\cos(\omega_0) - 2\sqrt{A}\alpha$$

*(All coefficients normalized by dividing by $a_0$)*

#### 2. High-Shelf Filter (9500 Hz Vocal Air & Sheen)
$$b_0 = A \left[ (A+1) + (A-1)\cos(\omega_0) + 2\sqrt{A}\alpha \right]$$
$$b_1 = -2A \left[ (A-1) + (A+1)\cos(\omega_0) \right]$$
$$b_2 = A \left[ (A+1) + (A-1)\cos(\omega_0) - 2\sqrt{A}\alpha \right]$$
$$a_0 = (A+1) - (A-1)\cos(\omega_0) + 2\sqrt{A}\alpha$$
$$a_1 = 2 \left[ (A-1) - (A+1)\cos(\omega_0) \right]$$
$$a_2 = (A+1) - (A-1)\cos(\omega_0) - 2\sqrt{A}\alpha$$

#### 3. Peaking Filter (3400 Hz Intelligibility & Clarity)
$$b_0 = 1 + \alpha A, \quad b_1 = -2\cos(\omega_0), \quad b_2 = 1 - \alpha A$$
$$a_0 = 1 + \frac{\alpha}{A}, \quad a_1 = -2\cos(\omega_0), \quad a_2 = 1 - \frac{\alpha}{A}$$

#### 4. Analog Soft Saturator
To prevent harsh digital overs ($|y[n]| \ge 1.0$) during heavy acoustic boosts, samples exceeding $\pm 0.85$ pass through a warm cubic saturation curve:
$$y_{\text{sat}}[n] = \text{sgn}(y) \cdot \left( 0.85 + \frac{|y| - 0.85}{1.0 + (|y| - 0.85)^2} \right)$$

---

## 🧠 4. Spectral Frequency Tone Cloning Engine (`rustfft`)

The Tone Cloner extracts an acoustic fingerprint from any high-fidelity reference audio (e.g., studio microphone recording) and dynamically matches the input voice to that tone.

### A. Windowed Fast Fourier Transform
1. **Window Function:** 2048-point Hann window:
   $$w[i] = 0.5 \left( 1 - \cos\left(\frac{2\pi i}{N - 1}\right) \right), \quad i \in [0, N-1]$$
2. **Frequency Bin Resolution:**
   $$\Delta f = \frac{F_s}{N} = \frac{48\,000}{2048} \approx 23.44\text{ Hz/bin}$$
3. **Magnitude Spectrum:**
   $$|X[k]| = \frac{2}{N} \sqrt{\text{Re}(X[k])^2 + \text{Im}(X[k])^2}, \quad k \in [0, N/2]$$

### B. 7-Band Acoustic Sub-Band Energy Integration
Energy is integrated across standard psychoacoustic intervals:
- **Band 0 (Sub-Bass):** 20 Hz – 60 Hz
- **Band 1 (Bass):** 60 Hz – 250 Hz (Vocal warmth & resonance)
- **Band 2 (Low-Mids):** 250 Hz – 500 Hz (Vocal body & fullness)
- **Band 3 (Mids):** 500 Hz – 2000 Hz (Vowel formant core)
- **Band 4 (High-Mids):** 2000 Hz – 4000 Hz (Consonant articulation)
- **Band 5 (Presence):** 4000 Hz – 8000 Hz (Vocal brightness)
- **Band 6 (Air/Brilliance):** 8000 Hz – 20000 Hz (High-frequency sheen)

### C. Dynamic Delta EQ Matching Algorithm
Given the normalized energy vector of the reference recording $\vec{E}_{\text{target}}$ and the live recording $\vec{E}_{\text{input}}$:

$$\Delta G_k = 10 \log_{10}\left( \frac{E_{\text{target}, k} + \epsilon}{E_{\text{input}, k} + \epsilon} \right)$$

The computed delta values are clamped to $[-9.0\text{ dB}, +9.0\text{ dB}]$ for musical safety and mapped to the active Biquad Cascade sliders in real-time.

---

## 🔒 5. AI Core Framework Integration (`crates/core`)

The application consumes the foundation layer without modifying core internals:
1. **Silero VAD (`ai_core::models::onnx_adapters::SileroVadLocal`):** Real ONNX runtime inference providing real-time voice probability $[0.0, 1.0]$.
2. **Audio Resampler (`ai_core::models::dsp::AudioResampler`):** Rubato sinc interpolation handling sample rate conversion between hardware (44.1/48kHz) and neural models (16kHz).
3. **Telemetry & Lifecycle:** Seamless integration with execution contexts and state machines.
