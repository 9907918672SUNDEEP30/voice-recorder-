# 🚀 Video Studio — Deployment Guide (`apps/video_studio`)
## Shambhavi AI Labs — OBS-Style Creator Studio Desktop & Mobile Creator App (Android / iOS)

---

## 🎬 1. फ्रंटएंड व क्लाइंट आर्किटेक्चर (Frontend & Client Architecture)

| घटक | विवरण |
|---|---|
| **कोर वीडियो इंजन** | 100% शुद्ध रस्ट वीडियो पाइपलाइन, डायरेक्ट WMF/V4L2 वेबकैम कैप्चर, 60 FPS पर सब-8ms लेटेंसी |
| **वर्चुअल AI कैमरामैन PTZ** | ऑटो-फ्रेमिंग (डेडज़ोन + एक्स्पोनेंशियल मूविंग एवरेज EMA डैम्पिंग) बिना किसी अतिरिक्त फिजिकल कैमरा ऑपरेटर के |
| **आई-कॉन्टैक्ट करेक्शन** | टेलीप्रॉम्प्टर या स्क्रीन पढ़ते समय पुतलियों को सीधा कैमरे में देखने का कोसाइन फेदरिंग मास्क सुधार |
| **3D LUT कलर ग्रेडिंग** | 17x17x17 ट्राई-लीनियर इंटरपोलेशन सिनेमैटिक कलर प्रोफाइल (Teal & Orange, Vintage, Warm Cinema) |
| **OBS-स्टाइल डेस्कटॉप UI** | **Creator Studio Desktop App** (Windows/Mac) — लाइव वीडियो प्रीव्यू, PTZ ट्रैकिंग ग्रिड, आई-कॉन्टैक्ट टॉगल, और रिकॉर्डिंग कंट्रोल्स |
| **नेटिव मोबाइल ऐप (Android / iOS)** | **Mobile Creator Studio App** — क्योंकि अधिकांश यूट्यूबर्स व रील क्रिएटर्स सीधे मोबाइल से शूटिंग करते हैं; ऑटो-फ्रेमिंग व स्किन स्मूथिंग के साथ लाइव वीडियो कैप्चर |
| **लाइसेंसिंग एवं मीटरिंग** | `crates/drm_guard` द्वारा 24-घंटे लीज + OTA ऑटो-अपडेटर (`VideoStudioMetrics`: रेंडर मिनट्स व एक्सपोर्ट्स) |

---

## 🧪 2. टेस्टिंग गाइड (How to Test & Verify — आप खुद कैसे टेस्ट करेंगे)

### Level 1: स्टेटिक कंपाइलेशन व ऑडिट
```powershell
cargo check -p video_studio
cargo clippy -p video_studio -- -D warnings
```

### Level 2: ऑटोमेटेड वीडियो व ग्रेडिंग टेस्ट सुइट
```powershell
cargo test -p video_studio -- --nocapture
```

### Level 3: हैंड्स-ऑन लाइव वीडियो स्टूडियो टेस्ट (Drive Video Studio)
1. **वर्चुअल कैमरा व आई-कॉन्टैक्ट सिमुलेशन टेस्ट:**
   ```powershell
   cargo run -p video_studio -- --virtual --frames 120 --lut TealAndOrange --gaze
   ```
   - स्क्रीन पर देखें:
     - **Frame Processing:** 60 FPS पर निरंतर < 8ms प्रति फ्रेम।
     - **PTZ Smoothing:** वर्चुअल कैमरा स्मूथली चेहरे को सेंटर में ट्रैक करता है।
     - **Eye-Contact:** स्क्रीन पर पढ़ने के बावजूद आंखें कैमरे की ओर संरेखित।
     - **LUT Color:** साधारण वेबकैम फुटेज को रिच हॉलीवुड सिनेमैटिक ग्रेड में बदलना।

---

## 📋 3. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist)

- [ ] **1. Zero Compiler Warnings:** `cargo check --release -p video_studio` 0 एरर और 0 वार्निंग्स।
- [ ] **2. 60 FPS लेटेंसी बजट:** प्रति फ्रेम प्रोसेसिंग समय **< 16.6ms** (60 FPS मानक)।
- [ ] **3. DirectML GPU एक्सेलेरेशन:** DirectX 12 / DirectML सपोर्ट सक्रिय है।
- [ ] **4. 24-घंटे DRM लीज:** `crates/drm_guard` सक्रिय है और मशीन बाइंडिंग लॉक है।
- [ ] **5. रिसोर्स बजट:** 1080p 60 FPS रेंडरिंग के दौरान RAM उपयोग **< 120 MB** पर स्थिर।

---

## 🚀 4. फाइनल डिप्लॉयमेंट प्रक्रिया (Final Production Deployment)

### Step 1: डेस्कटॉप प्रोडक्शन बाइनरी संकलन
```powershell
cargo build --release -p video_studio
```
बाइनरी पाथ: `target/release/video_studio.exe` (Windows) या `target/release/video_studio` (Mac/Linux)।

### Step 2: मोबाइल ऐप बंडलिंग (Android & iOS)
वीडियो स्टूडियो रस्ट कोर को मोबाइल ऐप में नेटिव कैमरा फ्रेमवर्क (CameraX Android / AVFoundation iOS) के साथ रीयल-टाइम फिल्टर के रूप में बंडल किया जाता है:
```powershell
cargo build --target aarch64-linux-android --release -p video_studio
cargo build --target aarch64-apple-ios --release -p video_studio
```
रस्ट का ज़ीरो-कॉस्ट एब्स्ट्रैक्शन मोबाइल को बिना बैटरी गर्म किए 60 FPS पर स्मूथ AI कैमरा एक्सपीरियंस प्रदान करता है।
