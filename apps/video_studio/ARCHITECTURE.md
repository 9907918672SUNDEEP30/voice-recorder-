# 🏗️ Architecture Blueprint: AI Video Studio (`apps/video_studio`)

> **Role & Classification:** Real-Time AI Virtual Cameraman, Teleprompter Gaze Correction, and 3D LUT Cinematic Video Engine.  
> **Philosophy:** Pure Rust • Core Engine First • Zero Python • Completely decoupled from Audio/DSP.

---

## 📐 1. System Topology & Data Pipeline

The pipeline processes video streams at sub-20ms frame latency (30-60 FPS) through 5 specialized stages:

```
                      [ Camera Hardware / Virtual Video Source ]
                                          │
                                          ▼ (Raw RGB 1080p/720p Frame Stream)
                      ┌───────────────────────────────────────┐
                      │    Stage 1: Camera Ingestion          │
                      │    (nokhwa / Native WMF / Virtual)    │
                      └───────────────────┬───────────────────┘
                                          │ Raw RGB Buffer
                                          ▼
                      ┌───────────────────────────────────────┐
                      │    Stage 2: AI Face & Landmark Detect │
                      │    (Fast Spatial Chrominance Centroid)│
                      └───────────────────┬───────────────────┘
                                          │ Face Bounding Box & Landmarks
                                          ▼
                      ┌───────────────────────────────────────┐
                      │    Stage 3: Auto-Framing & PTZ        │
                      │    • Deadzone Jitter Dampener         │
                      │    • Exponential Moving Average (EMA) │
                      │    • Sub-Pixel Bilinear Crop & Scale  │
                      └───────────────────┬───────────────────┘
                                          │ Centered 16:9 Canvas
                                          ▼
                      ┌───────────────────────────────────────┐
                      │    Stage 4: AI Eye Contact Correction │
                      │    • Iris/Pupil Locus Localization    │
                      │    • Optical Axis Vector Displacement │
                      │    • Radial Cosine Feathering Mask    │
                      └───────────────────┬───────────────────┘
                                          │ Natural Looking Eye Alignment
                                          ▼
                      ┌───────────────────────────────────────┐
                      │    Stage 5: Grading & Relighting      │
                      │    • Edge-Preserving Guided Smoothing │
                      │    • Radial Studio Face Relighting    │
                      │    • 3D LUT Trilinear Color Grading   │
                      └───────────────────┬───────────────────┘
                                          │ Final Broadcast-Grade Frame
                                          ▼
                      ┌───────────────────────────────────────┐
                      │    Output: Real-Time Display / Stream │
                      └───────────────────────────────────────┘
```

---

## 🎯 2. Mathematical Algorithms (Handcrafted Pure Rust)

### A. Auto-Framing & Virtual PTZ Damping
1. **Deadzone Policy:** If face displacement $||\Delta C|| < D_{\text{thresh}}$, target window remains static:
   $$\text{ActiveTarget} = \begin{cases} C_{\text{new}}, & \text{if } ||C_{\text{new}} - C_{\text{last}}|| > D_{\text{thresh}} \\ C_{\text{current}}, & \text{otherwise} \end{cases}$$
2. **Exponential Moving Average (EMA) Motion Damping:**
   $$\text{Box}[t] = \alpha \cdot \text{Box}_{\text{target}} + (1 - \alpha) \cdot \text{Box}[t-1]$$
   *(Where $\alpha = 0.08$ produces fluid cinematic panning without sudden jerks).*
3. **Sub-Pixel Bilinear Resampling:**
   $$I(x, y) = (1 - \Delta x)(1 - \Delta y) I_{00} + \Delta x (1 - \Delta y) I_{10} + (1 - \Delta x)\Delta y I_{01} + \Delta x \Delta y I_{11}$$

### B. AI Eye Contact (Teleprompter Iris Warping)
1. **Pupil Center Vector Shift:**
   $$\Delta x = \pm R_x \cdot k_{\text{align}} \cdot \beta, \quad \Delta y = -R_y \cdot k_{\text{lift}} \cdot \beta$$
2. **Radial Cosine Feathering Weight:**
   For normalized distance $r = \sqrt{(dx/R_x)^2 + (dy/R_y)^2}$:
   $$w(r) = \begin{cases} 0.5 \left( 1 + \cos(\pi \cdot r) \right), & r \le 1.0 \\ 0.0, & r > 1.0 \end{cases}$$
   Eliminates edge distortion and preserves eye sclera, eyelashes, and skin.

### C. 3D LUT Trilinear Color Grading
Transforms RGB space using a sampled 3D cube matrix via 8-corner trilinear volume interpolation:
$$C(r, g, b) = \sum_{i=0}^1 \sum_{j=0}^1 \sum_{k=0}^1 w_{ijk} \cdot \text{LUT}(r_i, g_j, b_k)$$

### D. Edge-Preserving Skin Smoothing
Local variance $\sigma^2$ test separates skin areas from sharp facial features (eyes/lips):
$$\text{Weight}_{\text{smooth}} = \text{Strength} \cdot \left( 1 - \min\left(1, \frac{\sigma^2}{\sigma^2_{\text{thresh}}}\right) \right)$$
Skin blemishes are softened while sharp edges remain 100% untouched.

---

## ⚡ 3. Real-Time Latency Budget (< 20ms)

| Subsystem | Execution Time (720p / 1080p) |
| :--- | :--- |
| **Camera Ingestion** | ~3.0 ms |
| **Face & Eye Detection** | ~1.5 ms |
| **Auto-Framing PTZ Rescale** | ~3.5 ms |
| **Eye Contact Warping** | ~1.2 ms |
| **Skin Filter & Relighting** | ~3.0 ms |
| **3D LUT Trilinear Grading** | ~2.5 ms |
| **Total Pipeline Frame Latency** | **~14.7 ms (60 FPS Capable!)** |
