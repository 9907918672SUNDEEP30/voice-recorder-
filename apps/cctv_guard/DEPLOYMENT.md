# 🚀 CCTV Guard — Deployment Guide (`apps/cctv_guard`)
## Shambhavi AI Labs — On-Premises Turnkey Black-Box Appliance & TP-Link Style Web Portal

---

## 📦 1. हार्डवेयर व फ्रंटएंड आर्किटेक्चर (Appliance & Web Portal Architecture)

| घटक | विवरण |
|---|---|
| **डिप्लॉयमेंट रूपरेखा** | **कंपनी का समर्पित ऑन-प्रिमाइसेस ब्लैक-बॉक्स एप्लायंस** (TP-Link / Jio सेट-टॉप बॉक्स स्टाइल टर्नकी हार्डवेयर: Celeron/Dual-Core CPU, 2 GB RAM, SSD) |
| **एम्बेडेड फ्रंटएंड** | **TP-Link स्टाइल एम्बेडेड वेब सर्वर** (`http://localhost:8080` / `http://<box-ip>:8080`) — किसी बाहरी Node.js/NPM की आवश्यकता नहीं |
| **4-चरणीय सेटअप विज़ार्ड** | 1. हार्डवेयर फिंगरप्रिंट लॉक, 2. 24h क्रिप्टोग्राफिक लीज, 3. RTSP कैमरा डिस्कवरी, 4. AI संवेदनशीलता व WhatsApp नंबर |
| **प्रोएक्टिव हेल्थ टेलीमेट्री** | हर 10 मिनट में वायर-स्टेटस व कैमरा हेल्थ पिंग; तार ढीला होने या कैमरा डाउन होने पर तत्काल **WhatsApp / SMS अलर्ट** |
| **Jio-TV स्टाइल OTA अपडेटर** | बैकग्राउंड में बिना लाइव रिकॉर्डिंग या प्रोटेक्शन रोके SHA-256 सत्यापित एटॉमिक बाइनरी स्वैप (`replace_executable`) |

---

## 🧪 2. टेस्टिंग गाइड (How to Test & Verify — आप खुद कैसे टेस्ट करेंगे)

### Level 1: स्टेटिक कंपाइलेशन व ऑडिट
```powershell
cargo check -p cctv_guard
cargo clippy -p cctv_guard -- -D warnings
```

### Level 2: ऑटोमेटेड डिटेक्शन व सायरन टेस्ट सुइट
```powershell
cargo test -p cctv_guard -- --nocapture
```

### Level 3: हैंड्स-ऑन लाइव सिमुलेशन टेस्ट (Drive the CCTV Guard)
1. **इंट्रूडर सिमुलेशन टेस्ट:**
   ```powershell
   cargo run -p cctv_guard -- --simulate "NightIntruderShutter" --siren-test
   ```
   - स्क्रीन पर देखें:
     - **Tier-1 YOLOv8m:** मोशन व ह्यूमन बाउंडिंग बॉक्स डिटेक्ट (< 15ms)।
     - **Tier-2 Qwen VLM:** रात के समय शटर तोड़ने का संदर्भ विश्लेषण (< 400ms)।
     - **Siren Relay Trigger:** GPIO / USB रिले पर 110dB सायरन का सिग्नल आउटपुट।
     - **WhatsApp Webhook:** फोन पर इमेज स्नैपशॉट के साथ सुरक्षा अलर्ट।

2. **10-मिनट लूज़-वायर व कैमरा फॉल्ट टेस्ट:**
   ```powershell
   cargo run -p cctv_guard -- --simulate-wire-fault
   ```
   - सिम्युलेट करें कि कैमरा 2 का तार कट गया या ढीला हो गया।
   - 10-मिनट का टेलीमेट्री पिंग तुरंत क्लाउड को `{ wire_faults: 1, offline_cameras: ["cam_02"] }` भेजेगा और आपके WhatsApp पर चेतावनी संदेश आएगा।

---

## 📋 3. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist)

- [ ] **1. Zero Compiler Warnings:** `cargo check --release -p cctv_guard` 0 त्रुटियां और 0 चेतावनियां।
- [ ] **2. ONNX मॉडल्स पूर्णता (`C:\AI_Models\`):**
  - `yolov8m.onnx` (103.7 MB)
  - `qwen2-vl-2b/` (VLM विजन मॉडल व टोकेनाइज़र)
- [ ] **3. हार्डवेयर मशीन फिंगरप्रिंट लॉक:** मदरबोर्ड UUID + CPU ID + MAC एड्रेस सही ढंग से बंधा हुआ है।
- [ ] **4. 24-घंटे DRM लीज:** `crates/drm_guard` सक्रिय है और 24h ऑफलाइन ग्रेस पीरियड बिना क्रैश के काम कर रहा है।
- [ ] **5. रिसोर्स बजट:** 4 कैमरों के साथ निरंतर चलने पर RAM उपयोग **< 900 MB** (2 GB ब्लैक-बॉक्स बजट के अंदर)।

---

## 🚀 4. टर्नकी ब्लैक-बॉक्स डिप्लॉयमेंट प्रक्रिया (Appliance Deployment)

### Step 1: प्रोडक्शन रिलीज़ बाइनरी संकलन
```powershell
cargo build --release -p cctv_guard
```

### Step 2: ब्लैक-बॉक्स ऑटो-स्टार्ट बूट सर्विस (Linux Appliance Systemd)
`/etc/systemd/system/cctv_guard.service`:
```ini
[Unit]
Description=Shambhavi CCTV Guard Black-Box Sentry
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/shambhavi/cctv_guard
ExecStart=/opt/shambhavi/cctv_guard/cctv_guard --config /etc/shambhavi/cctv_config.json
Restart=always
RestartSec=2
LimitNOFILE=65536
MemoryMax=1500M

[Install]
WantedBy=multi-user.target
```
सक्रिय करें:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now cctv_guard.service
```

### Step 3: ग्राहक के सेटअप के लिए TP-Link वेब विज़ार्ड खोलें
ग्राहक या तकनीशियन अपने मोबाइल/लैपटॉप पर ब्लैक-बॉक्स के वाई-फाई या लैन से कनेक्ट करके ब्राउज़र में खोलें:
```
http://192.168.1.100:8080
```
और 4 आसान चरणों में कैमरे व WhatsApp नंबर सेव कर दें।
