# 🚀 DSP Audio Recorder — Deployment Guide (`apps/dsp_recorder`)
## Shambhavi AI Labs — Studio DAW Desktop UI & Native Mobile App (Android / iOS)

---

## 🎧 1. फ्रंटएंड व क्लाइंट आर्किटेक्चर (Frontend & Client Architecture)

| घटक | विवरण |
|---|---|
| **कोर ऑडियो इंजन** | 100% शुद्ध रस्ट ऑडियो ड्राइवर (`cpal` + `hound`), सब-5.3ms बफर लेटेंसी, RBJ बाइक्वाड EQ मैथ, स्पेक्ट्रल टोन क्लोनिंग, 24-बिट 48kHz PCM लॉसलेस WAV |
| **डेस्कटॉप स्टूडियो DAW UI** | **क्रिएटर डेस्कटॉप DAW ऐप** (Windows/Mac/Linux) — रियल-टाइम FFT स्पेक्ट्रम एनालाइज़र, 10-बैंड इंटरैक्टिव EQ कर्व्स, और एनालॉग-स्टाइल dB मीटर्स |
| **नेटिव मोबाइल ऐप (Android / iOS)** | **Native Studio Mobile App** — क्योंकि अधिकांश क्रिएटर और पॉडकास्टर्स मोबाइल से बेहतरीन माइक पर रिकॉर्डिंग करते हैं; 1-क्लिक स्टूडियो एनहांसमेंट व क्लाउड सिंक |
| **लाइसेंसिंग एवं मीटरिंग** | `crates/drm_guard` द्वारा 24-घंटे क्रिप्टोग्राफिक लीज + OTA ऑटो-अपडेटर (`DspRecorderMetrics`: रिकॉर्डिंग मिनट्स व सक्रिय सेशंस) |

---

## 🧪 2. टेस्टिंग गाइड (How to Test & Verify — आप खुद कैसे टेस्ट करेंगे)

### Level 1: स्टेटिक कंपाइलेशन व ऑडिट
```powershell
cargo check -p dsp_recorder
cargo clippy -p dsp_recorder -- -D warnings
```

### Level 2: ऑटोमेटेड डीएसपी व फिल्टर टेस्ट सुइट
```powershell
cargo test -p dsp_recorder -- --nocapture
```

### Level 3: हैंड्स-ऑन लाइव ऑडियो रिकॉर्डिंग टेस्ट (Drive the DSP Recorder)
1. **ऑडियो डिवाइस सूची जांचें:**
   ```powershell
   cargo run -p dsp_recorder -- --list-devices
   ```
   - आपकी मशीन से जुड़े हेडफोन, माइक और ऑडियो इंटरफेस की सूची स्क्रीन पर दिखेगी।

2. **स्टूडियो-ग्रेड 24-बिट वार्म टोन रिकॉर्डिंग प्रारंभ करें:**
   ```powershell
   cargo run -p dsp_recorder -- --duration 10 --preset warm --output "output/my_studio_test.wav"
   ```
   - अपने माइक में 10 सेकंड बोलें।
   - स्क्रीन पर देखें:
     - **Buffer Latency:** < 5.3ms
     - **Biquad Filters:** लो-कट रंबल फिल्टर (80 Hz) + वार्म प्रेजेंस बूस्ट (3.5 kHz) रियल-टाइम में अप्लाई।
     - **Peak dB Meter:** 0 dB से नीचे हेडरुम मॉनिटरिंग।
   - रिकॉर्डिंग समाप्त होने पर `output/my_studio_test.wav` को वीएलसी या मीडिया प्लेयर में चलाकर क्रिस्टल-क्लियर 24-बिट स्टूडियो क्वालिटी सुनें।

---

## 📋 3. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist)

- [ ] **1. Zero Compiler Warnings:** `cargo check --release -p dsp_recorder` में 0 त्रुटियां और 0 चेतावनियां।
- [ ] **2. लेटेंसी सत्यापन:** हार्डवेयर ऑडियो बफर लेटेंसी **< 10ms** पर लॉक है।
- [ ] **3. आउटपुट डायरेक्टरी:** `output/` फोल्डर में राइट परमिशन्स उपलब्ध हैं।
- [ ] **4. 24-घंटे DRM लीज:** `crates/drm_guard` सक्रिय है और 24h ऑफलाइन ग्रेस पीरियड चालू है।
- [ ] **5. रिसोर्स बजट:** निरंतर रिकॉर्डिंग के दौरान RAM उपयोग **< 35 MB** और CPU उपयोग **< 2%**।

---

## 🚀 4. फाइनल डिप्लॉयमेंट प्रक्रिया (Final Production Deployment)

### Step 1: डेस्कटॉप प्रोडक्शन बाइनरी संकलन
```powershell
cargo build --release -p dsp_recorder
```
बाइनरी पाथ: `target/release/dsp_recorder.exe` (Windows) या `target/release/dsp_recorder` (Mac/Linux)।

### Step 2: मोबाइल ऐप बंडलिंग (Android NDK & iOS Rust Bridge)
मोबाइल ऐप के लिए कोर रस्ट लाइब्रेरी को C-ABI / FFI या UniFFI द्वारा एक्सपोर्ट किया जाता है:
```powershell
# Android NDK Target
cargo build --target aarch64-linux-android --release -p dsp_recorder

# iOS Target
cargo build --target aarch64-apple-ios --release -p dsp_recorder
```
Android APK और iOS App Store पैकेज सीधे इस हाई-परफॉर्मेंस रस्ट ऑडियो इंजन से लिंक होकर 24-बिट लॉसलेस रिकॉर्डिंग करेंगे।
