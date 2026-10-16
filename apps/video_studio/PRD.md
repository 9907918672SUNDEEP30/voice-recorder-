# 📋 Product Requirements Document (PRD): AI Video Studio (`video_studio`)

## 1. Executive Summary & Problem Statement
Solo video creators, educators, corporate presenters, and livestreamers face high production barriers: manual camerawork requires an operator or costly robotic motorized PTZ gimbals; teleprompters cause speakers to look off-camera (breaking viewer eye contact); and post-production color grading and skin retouching require expensive GPU-heavy suites like DaVinci Resolve or Adobe Premiere. Existing AI video apps require hefty cloud GPUs, subscription fees ($30–$50/month), or heavy Python stacks that consume multiple gigabytes of memory.

**`video_studio`** is a standalone, 100% native Rust, real-time desktop video enhancement appliance:
1. **Virtual AI Cameraman & Auto-Framing:** Real-time skin-chrominance face tracking with deadzone jitter suppression and Exponential Moving Average (EMA) damping, rendering cinematic Pan-Tilt-Zoom (PTZ) crops with sub-pixel bilinear interpolation.
2. **AI Teleprompter Eye Contact Correction:** Detects pupil locus and warps gaze back into the camera lens with radial cosine feathering masks and blink-safe protection to eliminate the "uncanny valley" robotic stare.
3. **Edge-Preserving Skin Polishing & Studio Relighting:** $5\times 5$ variance-adaptive spatial filter that smooths skin blemishes while retaining sharp eyes, eyelashes, and hair, coupled with radial face relighting.
4. **Cinematic 3D LUT Grading:** Real-time $17\times 17\times 17$ color grading volume with trilinear interpolation (`teal-orange`, `warm`, `minimal`, `film`).
5. **Zero Cloud, Ultra-Low Resource Execution:** Runs entirely locally on consumer hardware without requiring external cloud GPUs or heavy wrappers.

---

## 2. Target Market & User Personas

| Target Segment | Typical Deployments | Key Pain Points Solved |
|---|---|---|
| **Online Educators & Course Creators** | Home desktop studios, lecture recording. | Reading teleprompter scripts while maintaining direct eye contact with students. |
| **Corporate Presenters & Remote Executives** | Zoom / Teams / Google Meet calls. | Professional studio appearance without manual camera adjustments or expensive gimbals. |
| **Livestreamers & Tech Broadcasters** | OBS virtual camera, Twitch, YouTube Live. | Real-time cinematic color grading and automatic framing with < 5% CPU overhead. |
| **Telehealth & Remote Consultation** | Doctor-patient consultation desks. | Natural eye contact and clear, high-fidelity video transmission on budget PCs. |

---

## 3. Core Functional Requirements

### 3.1 Fast Face & Skin-Chrominance Centroid Extraction
- Color space conversion to YCbCr to isolate skin tones ($77 \le Cb \le 127$, $133 \le Cr \le 173$).
- Connected-component bounding box estimation running in $<2\text{ms}$ per frame.
- Landmark extraction for eyes, pupils, and mouth orientation.

### 3.2 AI Cameraman Auto-Framing & Virtual PTZ
- **Deadzone Jitter Cancellation:** Deadzone threshold ($D_{\text{thresh}} = 0.04$) to filter out natural micro-breathing movements.
- **Exponential Moving Average Damping:** Smooth camera tracking ($\alpha = 0.08$) simulating a professional fluid-head camera operator.
- **Sub-Pixel Resampling:** High-precision bilinear interpolation generating 720p/1080p output streams from camera inputs.

### 3.3 Teleprompter Gaze Correction
- Pupil locus extraction using minimum luminance gradient within eye socket boundaries.
- Vectorized pupil shift towards target optical center.
- **Cosine Feathering:** Smooth mask boundary ($w(r) = 0.5(1 + \cos(\pi r))$) to blend seamlessly with surrounding sclera.
- **Blink-Safe Protection:** Dynamic 3-frame decay preventing eyelid distortion during natural blinking.

### 3.4 3D LUT Color Transform & Skin Smoothing
- $17\times 17\times 17$ RGB cube volume grading with 8-point trilinear interpolation.
- Built-in cinematic LUT presets (`teal-orange`, `warm`, `minimal`, `film`).
- Variance-adaptive $5\times 5$ spatial bilateral skin smoothing.

---

## 4. Platform & Target Environments (Desktop Creator Application)

- **Application Nature:** Native Desktop Creator Application (Runs directly on the user's existing PC or laptop — no dedicated hardware purchase or edge appliance required).
- **Supported Platforms:**
  - **Windows:** Windows 10 / 11 (Windows Media Foundation backend).
  - **macOS:** macOS 12+ (AVFoundation camera capture).
  - **Linux:** Ubuntu / Fedora / Arch (V4L2 / PipeWire video).
- **Resource Footprint:** Extremely lightweight native binary allocating $<80\text{MB} - 120\text{MB}$ RAM during 1080p 60 FPS processing with $<8\text{ms}$ latency. Runs on any standard consumer desktop or laptop.
- **Camera Inputs:** Built-in laptop webcam or any standard plug-and-play USB UVC webcam (720p/1080p @ 30/60 FPS).
- **Delivery Form Factor:** Standalone desktop GUI / CLI executable (`video_studio.exe` / `video_studio`) and virtual webcam output for OBS, Zoom, Teams, and Google Meet.

