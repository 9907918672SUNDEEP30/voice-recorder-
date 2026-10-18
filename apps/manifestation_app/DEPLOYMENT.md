# 🚀 Manifestation Super App — Deployment Guide (`apps/manifestation_app`)
## Shambhavi AI Labs — Luxury Glassmorphism SPA & Native Mobile App (Android / iOS)

---

## 🧘 1. फ्रंटएंड व क्लाइंट आर्किटेक्चर (Frontend & Mobile App Architecture)

| घटक | विवरण |
|---|---|
| **कोर AI इंजन** | "Core-Engine First" आर्किटेक्चर (`crates/core` / `ai_core`), SD-Turbo Safetensors द्वारा सिंगल-स्टेप 512x512 विजन बोर्ड जनरेशन, 0% पायथन |
| **एम्बेडेड लक्ज़री वेब पोर्टल** | पोर्ट `9090` पर प्योर रस्ट Axum सर्वर (`http://localhost:9090`), ग्लासमोर्फिज्म थीम, 3-6-9 टेस्ला जर्नल, मिरर अफ़र्मेशन, और 528Hz/432Hz थिटा ऑडियो प्लेयर |
| **नेटिव मोबाइल ऐप (Android / iOS)** | **High-End UI/UX Native Mobile App** — प्रीमियम वाइब्रेशनल रेजोनेंस, डेली मॉर्निंग/इवनिंग नोटिफिकेशन्स, और ऑन-डिवाइस विजुअल मैनिफेस्टेशन |
| **लाइसेंसिंग एवं मीटरिंग** | `crates/drm_guard` द्वारा 24-घंटे लीज + OTA ऑटो-अपडेटर (`ManifestationMetrics`: अफ़र्मेशन्स, मेडिटेशन मिनट्स, और विजन बोर्ड्स) |

---

## 🧪 2. टेस्टिंग गाइड (How to Test & Verify — आप खुद कैसे टेस्ट करेंगे)

### Level 1: स्टेटिक कंपाइलेशन व लिंटिंग ऑडिट
```powershell
cargo check -p manifestation_app
cargo clippy -p manifestation_app -- -D warnings
```

### Level 2: ऑटोमेटेड यूनिट व इंटीग्रेशन टेस्ट सुइट
```powershell
cargo test -p manifestation_app -- --nocapture
```

### Level 3: हैंड्स-ऑन लाइव इंटरएक्टिव वेब पोर्टल टेस्ट (Drive Manifestation App)
1. **ऐप प्रारंभ करें:**
   ```powershell
   cargo run -p manifestation_app -- --port 9090
   ```
2. **ब्राउज़र में खोलें:**
   - जाएं: `http://localhost:9090`
   - **लिविंग विजन बोर्ड:** नया विजन बनाएं (e.g. *"Royal palace on misty mountain lake"*); SD-Turbo द्वारा सिंगल-स्टेप में रियल-टाइम इमेज जनरेट होगी।
   - **3-6-9 जर्नल:** सुबह की 3 इंटेंशन लिखें और वाइब्रेशनल स्कोर देखें।
   - **मिरर अफ़र्मेशन:** स्पीच/टेक्स्ट कंविक्शन स्कोरिंग टेस्ट करें।
   - **528 Hz थिटा मेडिटेशन:** प्ले बटन दबाएं और बाइक्वाड जनरेटेड 528 Hz सोल्फ़ेगियो टोन सुनें।

---

## 📋 3. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist)

- [ ] **1. Zero Compiler Warnings:** `cargo check --release -p manifestation_app` में शुद्ध 0 एरर और 0 वार्निंग्स।
- [ ] **2. टेस्ट्स 100% उत्तीर्ण:** सभी यूनिट व इंटीग्रेशन टेस्ट्स सफल।
- [ ] **3. SD-Turbo Safetensors पूर्णता (`C:\AI_Models\sd_turbo`):**
  - `text_encoder/model.safetensors` (1.36 GB)
  - `unet/diffusion_pytorch_model.safetensors` (3.46 GB)
  - `vae/diffusion_pytorch_model.safetensors` (334 MB)
  - `tokenizer/` व `scheduler/` पूर्ण।
- [ ] **4. "Core-Engine First" आर्किटेक्चर:** ऐप केवल `ai_core` से `ImageGenAdapter` व `SdTurboImageEngine` ले रहा है; कोई अलग मॉडल कोड ऐप के अंदर नहीं है।
- [ ] **5. 24-घंटे DRM लीज व मशीन लॉकिंग:** `crates/drm_guard` की 24h क्रिप्टोग्राफिक लीज सक्रिय है और ऑफलाइन ग्रेस पीरियड तैयार है।
- [ ] **6. पोर्ट्स व रिसोर्स बजट:** पोर्ट `9090` खुला है और RAM उपयोग **< 45 MB** पर स्थिर है (Zero Memory Leaks)।

---

## 🚀 4. फाइनल डिप्लॉयमेंट प्रक्रिया (Final Production Deployment)

### Step 1: प्रोडक्शन रिलीज़ बाइनरी संकलन
```powershell
cargo build --release -p manifestation_app
```
बाइनरी पाथ: `target/release/manifestation_app.exe` (Windows) या `target/release/manifestation_app` (Linux)।

### Step 2: 24x7 ऑटो-स्टार्ट बैकग्राउंड सर्विस डिप्लॉयमेंट

#### A. Windows Task Scheduler (सिस्टम बूट पर ऑटो-स्टार्ट):
```powershell
schtasks /create /tn "ShambhaviManifestation" `
  /tr "'D:\ai_core_framework - Copy (2)\target\release\manifestation_app.exe' --port 9090 --sd-models-dir 'C:\AI_Models\sd_turbo'" `
  /sc onstart /ru SYSTEM
```

#### B. Linux Headless Server Systemd Service (`/etc/systemd/system/manifestation.service`):
```ini
[Unit]
Description=Shambhavi Manifestation Super App Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/shambhavi/manifestation_app
ExecStart=/opt/shambhavi/manifestation_app/manifestation_app --port 9090 --sd-models-dir /opt/ai_models/sd_turbo
Restart=always
RestartSec=3
LimitNOFILE=65536
MemoryMax=250M

[Install]
WantedBy=multi-user.target
```
सक्रिय करें:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now manifestation.service
```

### Step 3: नेटिव मोबाइल ऐप बंडलिंग (Android & iOS)
रस्ट के कोर इंटेंशन व मेडिटेशन लॉजिक को मोबाइल ऐप के लिए कंपाइल किया जाता है:
```powershell
cargo build --target aarch64-linux-android --release -p manifestation_app
cargo build --target aarch64-apple-ios --release -p manifestation_app
```
