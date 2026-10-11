# 🎥 AI Video Studio: Virtual Cameraman & Gaze Correction Engine (`apps/video_studio`)

> **Production-grade, zero-latency local video processing, auto-framing, teleprompter gaze correction, edge-preserving skin polish, and cinematic 3D LUT grading engine built strictly in 100% pure Rust.**

---

## 🚀 Key Highlights & Capabilities

1. **Auto-Framing & Smart Face Tracking (AI Cameraman)**:
   - Dynamic Pan-Tilt-Zoom (PTZ) keeps the subject centered on stage.
   - **Deadzone Jitter Cancellation:** Suppresses micro-movements caused by breathing or slight head tilts ($D_{\text{thresh}} = 0.04$).
   - **Exponential Moving Average (EMA) Motion Damping:** Yields smooth, cinematic fluid-head camera panning ($\alpha = 0.08$) without robotic jerks.
   - **Sub-Pixel Bilinear Crop & Rescale:** Continuous floating-point sampling for crisp 720p/1080p broadcast frames.

2. **AI Eye Contact Correction (Teleprompter Mode)**:
   - Detects pupil locus within eye sockets using minimum luminance gradient.
   - Redirects downward teleprompter gaze directly into the camera lens.
   - **Radial Cosine Feathering Mask:** Prevents the "uncanny valley" robotic stare by blending smoothly with the sclera and eyelids ($w(r) = 0.5(1 + \cos(\pi r))$).
   - **Blink-Safe Protection:** Automatically decays intensity across 3 frames during natural eyelid blinks to eliminate tearing artifacts.

3. **Edge-Preserving Skin Smoothing & Studio Relighting**:
   - $5 \times 5$ neighborhood variance-adaptive spatial filter softens skin blemishes while keeping eyes, eyelashes, lips, and hair razor sharp.
   - Studio radial face relighting creates soft exposure falloff centered on the presenter.

4. **3D LUT Trilinear Color Grading**:
   - High-performance $17\times 17\times 17$ discrete cube volume color transform.
   - 8-point trilinear interpolation with 4 built-in cinematic presets:
     - `teal-orange`: High-end Hollywood cinematic contrast and skin-tone separation.
     - `warm`: Flattering podcast and educational creator warmth.
     - `minimal`: Clean, neutral, high-clarity tech presentation look.
     - `film`: Rich, saturated vintage Kodachrome film stock aesthetic.

5. **Pure Rust Hardware Independence**:
   - Fully decoupled from voice recording (`apps/dsp_recorder`).
   - Uses `nokhwa` for physical webcams with Windows Media Foundation (WMF) COM thread safety.
   - Built-in `VirtualCamera` studio simulator for 60 FPS offline testing and headless CI.

---

## 📐 System Pipeline Architecture

```
[ Camera Source / Virtual Simulator ]
                 │
                 ▼ (Raw RGB 1080p/720p Frame Stream)
┌────────────────────────────────────────────────────────┐
│ Stage 1: Fast Skin-Chrominance Centroid & Landmarks    │
└──────────────────────────┬─────────────────────────────┘
                           │ FaceBox & Facial Landmarks
                           ▼
┌────────────────────────────────────────────────────────┐
│ Stage 2: AI Cameraman Auto-Framing & Virtual PTZ       │
│ • Deadzone Filter  • EMA Damping  • Bilinear Resampler │
└──────────────────────────┬─────────────────────────────┘
                           │ Centered 16:9 Broadcast Canvas
                           ▼
┌────────────────────────────────────────────────────────┐
│ Stage 3: Teleprompter Eye Contact Correction           │
│ • Pupil Locus  • Vector Shift  • Cosine Mask • Blinks  │
└──────────────────────────┬─────────────────────────────┘
                           │ Natural Eye Optical Alignment
                           ▼
┌────────────────────────────────────────────────────────┐
│ Stage 4: Grading & Studio Relighting                   │
│ • Edge-Preserving Filter  • 3D LUT Trilinear Transform │
└──────────────────────────┬─────────────────────────────┘
                           │ Broadcast-Ready RGB Frame
                           ▼
┌────────────────────────────────────────────────────────┐
│ Stage 5: Output Storage & Real-Time Telemetry          │
│ • Lossless PPM Stream  • PNG Snapshot  • ASCII Monitor │
└────────────────────────────────────────────────────────┘
```

---

## 🛠️ Usage & CLI Reference

### 1. Query System Video Cameras
```bash
cargo run -p video_studio -- --devices
```

### 2. Run Virtual Studio Simulator (Default Test Mode)
```bash
cargo run -p video_studio -- --virtual
```

### 3. Connect to Physical Webcam with 3D LUT Grading
```bash
# Connect to camera index 0 with teal-orange LUT preset
cargo run -p video_studio -- --camera 0 --lut teal-orange
```

### 4. Adjust Eye Contact Teleprompter Intensity
```bash
cargo run -p video_studio -- --camera 0 --gaze 0.90
```

### 5. Capture a High-Resolution PNG Snapshot
```bash
cargo run -p video_studio -- --virtual --snapshot studio_preview.png
```

### 6. Record Lossless Frame Sequence
```bash
cargo run -p video_studio -- --virtual --record-frames 300 --out video_records
```

---

## 📊 CLI Options & Flags Table

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--devices` | | - | List all connected physical video camera devices |
| `--camera <IDX>` | `-c` | - | Select physical camera index (disables virtual camera) |
| `--virtual` | `-v` | active | Run 60 FPS virtual camera studio simulator |
| `--lut <PRESET>` | | `teal-orange` | 3D LUT preset: `teal-orange`, `warm`, `minimal`, `film`, `none` |
| `--gaze <0.0-1.0>` | | `0.85` | Teleprompter eye contact correction intensity |
| `--no-framing` | | - | Disable auto-framing virtual PTZ |
| `--no-gaze` | | - | Disable eye contact correction |
| `--snapshot <FILE>` | | - | Save single processed frame as PNG snapshot |
| `--record-frames <N>`| | - | Record $N$ processed frames as lossless PPM sequence |
| `--out <DIR>` | `-o` | `video_records` | Output directory for snapshots and recordings |
| `--help` | `-h` | - | Display help and usage instructions |

---

## 🖥️ Real-Time ASCII Dashboard

When running in interactive mode, the studio outputs a real-time console status dashboard:

```text
[STATUS] Active | Camera: Virtual Studio | Resolution: 1280x720
[FPS] 59.8 | Latency: 13.6 ms | Frame: #480
[AI TRACK] Face: LOCKED [x: 0.35, y: 0.30, w: 0.28, h: 0.38]
[PTZ CROP] Left: 0.08, Top: 0.06, Zoom: 1.34x (EMA Damped)
[GAZE] Active (Lift: 85%, Blink Safe: OPEN)
[LOOK] LUT: TealAndOrange (3D Cube) | Skin Polish: Active
```

---

## 📁 Crate & Module Organization

```text
apps/video_studio/
├── Cargo.toml                    # Workspace configuration & lean dependencies
├── README.md                     # Comprehensive developer & user guide
├── ARCHITECTURE.md               # Mathematical derivations & architectural blueprints
├── WORK_PROGRESS.md              # Source-of-truth progress & test audit tracker
├── src/
│   ├── lib.rs                    # Public module exports
│   ├── main.rs                   # Interactive CLI application & ASCII dashboard
│   ├── capture/
│   │   ├── mod.rs
│   │   └── camera.rs             # CameraSource trait, NokhwaCamera, VirtualCamera
│   ├── tracking/
│   │   ├── mod.rs
│   │   ├── face_detector.rs      # Skin-chrominance centroid & landmark detector
│   │   └── auto_framing.rs       # Virtual PTZ, Deadzone filter, EMA damper, bilinear scaler
│   ├── gaze/
│   │   ├── mod.rs
│   │   ├── iris_tracker.rs       # Elliptical eye socket & pupil localization
│   │   └── gaze_warp.rs          # Cosine-feathered gaze corrector & blink safety
│   ├── grading/
│   │   ├── mod.rs
│   │   ├── lut3d.rs              # 17x17x17 3D LUT trilinear volume interpolation
│   │   └── skin_smooth.rs        # Variance-adaptive skin filter & studio relighting
│   ├── storage/
│   │   ├── mod.rs
│   │   └── frame_writer.rs       # PNG snapshot encoder & binary PPM sequence streamer
│   └── pipeline/
│       ├── mod.rs
│       └── video_pipeline.rs     # 60 FPS multi-stage coordinator & telemetry
└── tests/
    └── video_tests.rs            # 7 comprehensive unit & integration tests
```

---

## 🧪 Testing & Verification

Run the entire automated test suite:
```bash
cargo test -p video_studio
```

**Test Coverage Summary (7/7 Passing):**
1. `test_video_frame_pixel_access`: Validates RGB24 linear buffer access and contiguous memory layout.
2. `test_face_detector_on_virtual_camera`: Tests spatial chrominance centroid tracking and eye landmarks.
3. `test_auto_framing_ema_damping_and_rescale`: Verifies deadzone jitter dampening, EMA smoothing, and bilinear crop.
4. `test_iris_tracker_and_gaze_warper`: Confirms pupil locus gradient detection, cosine feathering, and blink safety.
5. `test_lut3d_trilinear_interpolation`: Tests $17\times 17\times 17$ trilinear mathematical volume interpolation.
6. `test_skin_smoothing_and_relighting`: Validates edge preservation over high variance areas and facial relighting.
7. `test_end_to_end_virtual_camera_pipeline`: Executes the complete multi-stage pipeline and verifies telemetry.
