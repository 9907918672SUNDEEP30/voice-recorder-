# 🎥 AI Video Studio: Virtual Cameraman & Gaze Correction Engine (`apps/video_studio`)

> **Production-grade, zero-latency local video processing, auto-framing, teleprompter gaze correction, and cinematic 3D LUT grading engine built strictly in 100% pure Rust.**

---

## 🚀 Key Highlights & Capabilities

1. **Auto-Framing & Smart Face Tracking (AI Cameraman)**:
   - Dynamic Pan-Tilt-Zoom (PTZ) keeps the subject centered on stage.
   - **Deadzone Jitter Cancellation:** Suppresses micro-movements from breathing or slight head tilts.
   - **Exponential Moving Average (EMA) Motion Damping:** Yields smooth, cinematic camera panning without robotic jerks.
   - **Sub-Pixel Bilinear Crop & Rescale:** Real-time scaling to crisp 720p/1080p outputs.

2. **AI Eye Contact Correction (Teleprompter Mode)**:
   - Detects pupil locus within eye sockets.
   - Redirects downward teleprompter gaze directly into the camera lens.
   - **Cosine Feathering Mask:** Prevents the "uncanny valley" robotic stare by blending seamlessly with sclera and eyelids.
   - **Blink-Safe:** Automatically disables during natural eyelid blinks.

3. **Edge-Preserving Skin Smoothing & Studio Relighting**:
   - Variance-weighted spatial filter softens skin blemishes while keeping eyes, lips, and hair razor sharp.
   - Studio ring-light relighting centers soft illumination on the subject's face.

4. **3D LUT Trilinear Color Grading**:
   - High-performance $17\times 17\times 17$ trilinear volume color transform.
   - Built-in cinematic presets: `teal-orange`, `warm`, `minimal`, `film`.

5. **Pure Rust Hardware Independence**:
   - Fully decoupled from voice recording.
   - Uses `nokhwa` for physical cameras, with built-in `VirtualCamera` simulator for instant offline verification.

---

## 🛠️ Usage & CLI Reference

### 1. Run Virtual Studio Simulator (Test Mode)
```bash
cargo run -p video_studio -- --virtual
```

### 2. Connect to Physical Webcam
```bash
# First list devices
cargo run -p video_studio -- --devices

# Start studio with physical camera index 0
cargo run -p video_studio -- --camera 0
```

### 3. Apply Cinematic 3D LUT and Custom Eye Contact Strength
```bash
cargo run -p video_studio -- --camera 0 --lut teal-orange --gaze 0.90
```

### 4. Capture a Processed High-Resolution Snapshot
```bash
cargo run -p video_studio -- --virtual --snapshot studio_preview.png
```

### 5. CLI Flags Table

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--devices` | | - | List physical video camera devices |
| `--camera <IDX>` | `-c` | - | Select physical camera index |
| `--virtual` | `-v` | active | Run virtual camera simulator |
| `--lut <PRESET>`| | `teal-orange` | 3D LUT preset: `teal-orange`, `warm`, `minimal`, `film`, `none` |
| `--gaze <0.0-1.0>`| | `0.85` | Teleprompter eye contact correction intensity |
| `--no-framing` | | - | Disable auto-framing virtual PTZ |
| `--no-gaze` | | - | Disable eye contact correction |
| `--snapshot <PNG>`| | - | Save single processed frame as PNG snapshot |
| `--record-frames <N>`| | - | Record N frames as lossless PPM sequence |
| `--out <DIR>` | `-o` | `video_records` | Output directory for recordings |
| `--help` | `-h` | - | Show help message |

---

## 🧪 Testing & Verification

Run all test suites:
```bash
cargo test -p video_studio
```
Verifies pixel layouts, face detection, EMA damping, sub-pixel bilinear scaling, iris tracking, cosine gaze warping, 3D LUT trilinear math, and end-to-end 60 FPS pipeline execution.
