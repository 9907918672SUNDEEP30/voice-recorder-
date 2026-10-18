# 🚀 NVR Core — Deployment Guide (`apps/nvr_core`)
## Shambhavi AI Labs — 24/7 RTSP NVR Core & TP-Link Style Web Portal (Port 8080)

---

## 📦 1. सिस्टम व फ्रंटएंड आर्किटेक्चर (System & Web Portal Architecture)

| घटक | विवरण |
|---|---|
| **कोर इंजन** | 100% शुद्ध रस्ट मल्टी-थ्रेडेड RTSP कैप्चर, सर्कुलर डिस्क बफर, AES-256 एनक्रिप्टेड NVR स्टोरेज |
| **एम्बेडेड वेब पोर्टल** | पोर्ट `8080` पर प्योर रस्ट Axum सर्वर (`http://localhost:8080` / `http://0.0.0.0:8080`) |
| **TP-Link 4-स्टेप विज़ार्ड** | 1. हार्डवेयर बाइंडिंग, 2. 24h DRM लीज, 3. RTSP कैमरा मैपिंग (4/8/16 चैनल्स), 4. स्टोरेज व अलर्टिंग |
| **Jio-TV स्टाइल OTA अपडेटर** | बैकग्राउंड में स्वचालित नए वर्ज़न की जांच, SHA-256 हैश सत्यापन और बिना रिकॉर्डिंग रोके लाइव बाइनरी स्वैप |
| **10-मिनट प्रोएक्टिव टेलीमेट्री** | `CctvGuardMetrics` (सक्रिय कैमरे, वायर फॉल्ट्स, डिस्क स्पेस पर्सेंटेज) |

---

## 🧪 2. टेस्टिंग गाइड (How to Test & Verify — आप खुद कैसे टेस्ट करेंगे)

### Level 1: स्टेटिक कंपाइलेशन व ऑडिट
```powershell
cargo check -p nvr_core
cargo clippy -p nvr_core -- -D warnings
```

### Level 2: ऑटोमेटेड NVR व स्टोरेज टेस्ट सुइट
```powershell
cargo test -p nvr_core -- --nocapture
```

### Level 3: हैंड्स-ऑन लाइव TP-Link वेब पोर्टल टेस्ट (Drive NVR Core)
1. **NVR कोर सिमुलेटर प्रारंभ करें:**
   ```powershell
   cargo run -p nvr_core -- --simulate --cameras 4 --port 8080
   ```
2. **ब्राउज़र में खोलें:**
   - जाएं: `http://localhost:8080`
   - स्क्रीन पर आपको **TP-Link स्टाइल लक्ज़री 4-चरणीय क्विक सेटअप विज़ार्ड** दिखाई देगा:
     - **Step 1:** हार्डवेयर फिंगरप्रिंट (Motherboard UUID, CPU ID, MAC Address Lock) की पुष्टि करें।
     - **Step 2:** 24-घंटे क्लाउड लीज टोकन स्टेटस जांचें।
     - **Step 3:** 4 कैमरों के लाइव RTSP स्लॉट्स (मल्टी-कैमरा ग्रिड) देखें।
     - **Step 4:** स्टोरेज क्षमता (SSD बफर) और व्हाट्सएप नंबर डालकर **Save & Protect** पर क्लिक करें।
3. **OTA ऑटो-अपडेटर टेस्ट:**
   - सेटिंग्स में जाकर **Check for Updates** पर क्लिक करें; यह क्लाउडफ्लेयर वर्कर से नया वर्ज़न चेक करेगा।

---

## 📋 3. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist)

- [ ] **1. Zero Compiler Warnings:** `cargo check --release -p nvr_core` 0 एरर और 0 वार्निंग्स।
- [ ] **2. पोर्ट उपलब्धता:** पोर्ट `8080` अन्य किसी सर्विस द्वारा अधिकृत नहीं है।
- [ ] **3. सर्कुलर डिस्क स्टोरेज:** रिकॉर्डिंग डायरेक्टरी (e.g., `D:\nvr_recordings` या `/mnt/nvr_storage`) में कम से कम 20 GB खाली जगह।
- [ ] **4. 24-घंटे लीज व सुरक्षा:** `crates/drm_guard` की 24h क्रिप्टोग्राफिक लीज सक्रिय है और मशीन बाइंडिंग लॉक है।
- [ ] **5. रिसोर्स बजट:** 4 कैमरों पर निरंतर रिकॉर्डिंग के साथ RAM उपयोग **< 450 MB** पर स्थिर।

---

## 🚀 4. टर्नकी एप्लायंस डिप्लॉयमेंट प्रक्रिया (Appliance Deployment)

### Step 1: प्रोडक्शन रिलीज़ बाइनरी संकलन
```powershell
cargo build --release -p nvr_core
```

### Step 2: ऑटो-स्टार्ट बैकग्राउंड सर्विस डिप्लॉयमेंट (Windows Task Scheduler / Linux Systemd)

#### A. Windows Task Scheduler (सिस्टम बूट पर ऑटो-स्टार्ट):
```powershell
schtasks /create /tn "ShambhaviNvrCore" `
  /tr "'D:\ai_core_framework - Copy (2)\target\release\nvr_core.exe' --port 8080 --storage-dir 'D:\recordings'" `
  /sc onstart /ru SYSTEM
```

#### B. Linux Headless Black-Box Appliance (`/etc/systemd/system/nvr_core.service`):
```ini
[Unit]
Description=Shambhavi NVR Core Appliance Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/shambhavi/nvr_core
ExecStart=/opt/shambhavi/nvr_core/nvr_core --port 8080 --storage-dir /var/shambhavi/recordings
Restart=always
RestartSec=3
LimitNOFILE=65536
MemoryMax=1200M

[Install]
WantedBy=multi-user.target
```
सक्रिय करें:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now nvr_core.service
```
