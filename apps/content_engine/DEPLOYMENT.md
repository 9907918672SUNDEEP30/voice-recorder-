# 🚀 Content Engine — Deployment Guide (`apps/content_engine`)
## Shambhavi AI Labs — Cloud Rust Engine & Desktop/Chrome Creator Workspace

---

## ✍️ 1. फ्रंटएंड व क्लाइंट आर्किटेक्चर (Frontend & Desktop Workspace Architecture)

| घटक | विवरण |
|---|---|
| **क्लाउड बैकएंड इंजन** | 100% शुद्ध रस्ट ऑटोनॉमस कंटेंट पाइपलाइन, क्लाउड VPS पर 24/7 बैकग्राउंड वर्कर, 0% पेड API, 0% पायथन |
| **ट्रेंड रेकोनिसेंस** | क्रोम DevTools प्रोटोकॉल (CDP) हेडलेस स्क्रैपर, ब्रेकिंग ट्रेंड्स और कॉम्पिटिटर एनालिसिस |
| **ऑम्नीचैनल जेनरेशन** | वायरल शॉर्ट-फॉर्म वीडियो स्क्रिप्ट्स, लॉन्ग-फॉर्म ब्लॉग्स, और सोशल मीडिया पोस्ट्स का ऑटोमैटिक ड्राफ्टिंग |
| **ऑडियो व कराओके सबटाइटल्स** | टीटीएस वॉइसओवर सिंथेसिस + सब-सेकंड वर्ड टाइमस्टैम्प्स के साथ ASS कराओके सबटाइटल्स एक्सपोर्ट |
| **डेस्कटॉप क्रिएटर वर्कस्पेस** | **Desktop/Chrome Creator Studio Workspace** — (नोट: यूज़र निर्देशानुसार अभी इसका मोबाइल ऐप नहीं बनेगा, केवल डेस्कटॉप/क्रोम वर्कस्पेस पर फोकस है) |
| **लाइसेंसिंग एवं मीटरिंग** | `crates/drm_guard` द्वारा 24-घंटे लीज + OTA ऑटो-अपडेटर (`ContentEngineMetrics`: जनरेटेड पोस्ट्स, वीडियो स्क्रिप्ट्स, और शेड्यूल्ड कैंपेन्स) |

---

## 🧪 2. टेस्टिंग गाइड (How to Test & Verify — आप खुद कैसे टेस्ट करेंगे)

### Level 1: स्टेटिक कंपाइलेशन व ऑडिट
```powershell
cargo check -p content_engine
cargo clippy -p content_engine -- -D warnings
```

### Level 2: ऑटोमेटेड पाइपलाइन व टीटीएस टेस्ट सुइट
```powershell
cargo test -p content_engine -- --nocapture
```

### Level 3: हैंड्स-ऑन लाइव कंटेंट पाइपलाइन टेस्ट (Drive Content Engine)
1. **ट्रेंड रेकॉन व कॉम्पिटिटर एनालिसिस टेस्ट:**
   ```powershell
   cargo run -p content_engine -- recon --topic "AI Automation 2026"
   ```
   - स्क्रीन पर देखें:
     - क्रोम CDP द्वारा लाइव ट्रेंड्स, टॉप वायरल कीवर्ड्स और कॉम्पिटिटर व्यूज का JSON डॉसियर।

2. **वॉइसओवर व ASS कराओके सबटाइटल जेनरेशन टेस्ट:**
   ```powershell
   cargo run -p content_engine -- speech --text "Welcome to the future of pure native AI content creation." --output "output/test_voice.wav"
   ```
   - टर्मिनल में देखें:
     - `output/test_voice.wav` (हाई-क्वालिटी वॉइसओवर ऑडियो)
     - `output/test_voice.ass` (वर्ड-लेवल हाइलाइट कराओके सबटाइटल्स)
   - वीएलसी प्लेयर में दोनों को एक साथ प्ले करके सिंक चेक करें।

3. **संपूर्ण ऑटोनॉमस कैंपेन टेस्ट:**
   ```powershell
   cargo run -p content_engine -- pipeline --topic "Indian Tech Revolution"
   ```
   - कुछ ही सेकंडों में रिसर्च से लेकर वीडियो स्क्रिप्ट, वॉइसओवर, सबटाइटल और सोशल मीडिया ड्राफ्ट तैयार।

---

## 📋 3. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist)

- [ ] **1. Zero Compiler Warnings:** `cargo check --release -p content_engine` 0 त्रुटियां और 0 चेतावनियां।
- [ ] **2. Chrome CDP एंडपॉइंट:** हेडलेस क्रोम पोर्ट (9222) उपलब्ध है या डिफॉल्ट स्क्रैपर कॉन्फ़िगर है।
- [ ] **3. आउटपुट डायरेक्टरी:** `output/omnichannel_content/` डायरेक्टरी राइट परमिशन्स के साथ तैयार है।
- [ ] **4. 24-घंटे DRM लीज:** `crates/drm_guard` सक्रिय है और कंटेंट जेनरेशन की मीटरिंग ट्रैक हो रही है।
- [ ] **5. रिसोर्स बजट:** पूरी पाइपलाइन के निष्पादन के दौरान RAM उपयोग **< 160 MB** पर स्थिर।

---

## 🚀 4. फाइनल डिप्लॉयमेंट प्रक्रिया (Cloud Engine & Desktop Workspace Deployment)

### Step 1: प्रोडक्शन रिलीज़ बाइनरी संकलन
```powershell
cargo build --release -p content_engine
```
बाइनरी पाथ: `target/release/content_engine.exe` (Windows) या `target/release/content_engine` (Linux)।

### Step 2: क्लाउड बैकग्राउंड शेड्यूलर सर्विस (Linux Systemd)
`/etc/systemd/system/content_engine.service`:
```ini
[Unit]
Description=Shambhavi Content Engine Cloud Worker
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/shambhavi/content_engine
ExecStart=/opt/shambhavi/content_engine/content_engine pipeline
Restart=always
RestartSec=60
LimitNOFILE=65536
MemoryMax=800M

[Install]
WantedBy=multi-user.target
```
सक्रिय करें:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now content_engine.service
```
