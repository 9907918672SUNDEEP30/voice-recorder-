# 🎛️ Audio Recorder & Tone Cloner Studio (`apps/dsp_recorder`)

> **Production-Ready, Zero-Latency Embedded/Desktop Audio Processing and Acoustic Tone Cloning Engine.**  
> Built strictly in 100% pure Rust, inheriting the sealed and neutral **AI Core Framework** (`crates/core`).

---

## 🚀 Key Highlights & Architecture

1. **Zero-Latency Audio Capture (`cpal`)**:
   - Hardware stream interface operating with ultra-low buffer frames (128 / 256 samples, `< 5.3ms` hardware buffer latency).
   - Real-time priority execution: Zero heap allocation and lock-free thread transfer in the callback thread.
   - Channel-averaging to normalized mono 32-bit floating point (`[-1.0, 1.0]`).

2. **Studio IIR Biquad Filters (Acoustic Styling)**:
   - Direct Form II Transposed implementation ensuring numerical stability on ARM, Raspberry Pi, and x86_64.
   - **Rumble HPF (35 Hz):** Removes sub-audible microphone handling noise and floor thump.
   - **Low-Shelf Bass Warmth (110 Hz):** Enhances chest resonance and vocal body.
   - **Low-Mid Peaking (400 Hz):** Sculptural control over vocal boxiness.
   - **High-Mid Presence (3400 Hz):** Boosts consonant intelligibility and vocal projection.
   - **High-Shelf Studio Air (9500 Hz):** Adds high-end sheen and clarity.
   - **Warm Analog Soft-Saturator:** Prevents digital clipping on extreme dynamics.

3. **Fast Fourier Transform Spectral Matching (`rustfft`)**:
   - 2048-point forward FFT with precomputed Hann windowing.
   - 7-band acoustic energy envelope extraction: Sub-Bass, Bass, Low-Mids, Mids, High-Mids, Presence, and Air.
   - Reference tone cloning: Extracts spectral delta curve against any reference WAV file and automatically tunes DSP filter gains.

4. **Lossless High-Fidelity WAV Serialization (`hound`)**:
   - Asynchronous, decoupled background writer thread with 128KB buffered disk I/O.
   - Supports 16-bit, 24-bit PCM, and 32-bit float at 44.1kHz or 48kHz.
   - Safe header finalization with zero corrupted WAV files on sudden stop (SIGINT / Ctrl+C).

5. **AI Core Framework Reuse (`crates/core`)**:
   - Inherits `ai_core::models::onnx_adapters::SileroVadLocal` and `VADAdapter` for optional speech activity gating.
   - Inherits `ai_core::models::dsp::AudioResampler` (Rubato Sinc resampler) for multi-rate stream alignment.

---

## 🛠️ CLI Reference & Usage

### 1. Enumerate Audio Input Devices
```bash
cargo run -p dsp_recorder -- --devices
```

### 2. Spectral Analysis on a Reference Recording
```bash
cargo run -p dsp_recorder -- --analyze path/to/reference_vocal.wav
```
*Output gives 7-band energy distribution, spectral centroid (brightness in Hz), and RMS level.*

### 3. Record with Live Tone Cloning from Reference WAV
```bash
cargo run -p dsp_recorder -- --clone path/to/podcast_mic.wav --out my_styled_recording.wav
```

### 4. Live Recording with Manual Studio DSP Sliders
```bash
cargo run -p dsp_recorder -- --out studio_take.wav --bass 4.0 --presence 3.0 --air 2.5 --depth 24
```

### 5. CLI Arguments & Flags Table

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--devices` | | - | Lists all available system audio input devices |
| `--analyze <FILE>` | | - | Performs 2048-pt FFT analysis on a WAV file |
| `--clone <WAV>` | `-c` | - | Extracts tone signature from reference WAV and applies it live |
| `--out <FILE>` | `-o` | `recording_master.wav` | Destination output WAV file path |
| `--bass <dB>` | | `+3.5` | Low-shelf bass boost in decibels (-18 to +18 dB) |
| `--presence <dB>` | | `+2.5` | Vocal presence boost in decibels (-18 to +18 dB) |
| `--air <dB>` | | `+2.0` | High-shelf air/sheen boost in decibels (-18 to +18 dB) |
| `--rate <Hz>` | | `48000` | Audio sampling rate (44100 or 48000) |
| `--buffer <N>` | | `256` | Hardware buffer frame size (128 or 256 for low latency) |
| `--depth <BITS>`| | `16` | Bit depth: `16` (16-bit PCM), `24` (24-bit PCM), `32` (Float32) |
| `--duration <S>`| `-d` | - | Auto-stops recording after specified seconds |
| `--help` | `-h` | - | Shows command-line help message |

---

## 🎚️ Live Terminal VU Meter

During recording, the terminal renders a real-time responsive VU meter:
```text
🎙️ [Time: 00:08] VU: [████████████████▒▒░░░░] -14.2 dB | Peak: 0.68
```
- **Green Zone (`█`):** Healthy signal level (-60 dB to -18 dB).
- **Yellow Zone (`▒`):** Studio broadcast target level (-18 dB to -6 dB).
- **Red Zone (`!`):** Approaching maximum ceiling (-6 dB to 0 dB).

---

## 🧪 Testing & Verification

Run the complete test suite:
```bash
cargo test -p dsp_recorder
```
All 6 unit and integration tests verify:
- Biquad coefficient mathematical stability and finiteness
- Direct Form II Transposed sine wave filtering
- Cascaded multi-stage DSP styling
- Zero-click exponential parameter smoother convergence
- Fast Fourier Transform 7-band spectral profiling and matching
- Lossless 16-bit WAV writing and reading roundtrip via `hound`

---

## 🍓 ARM & Raspberry Pi Deployment

To cross-compile for Raspberry Pi 4/5 (ARM64 `aarch64-unknown-linux-gnu`):
```bash
cargo build -p dsp_recorder --target aarch64-unknown-linux-gnu --release
```
The application uses pure software DSP routines without requiring external C/C++ audio frameworks.
