# 🚀 Manifestation Super App — Deployment Guide (`apps/manifestation_app`)

---

## 🧪 1. टेस्टिंग गाइड (How to Test & Verify — टेस्ट कैसे करनी है)

### Level 1: स्टेटिक कंपाइलेशन व लिंटिंग ऑडिट
```powershell
cargo check -p manifestation_app
cargo clippy -p manifestation_app -- -D warnings
```

### Level 2: ऑटोमेटेड यूनिट व इंटीग्रेशन टेस्ट सुइट
```powershell
cargo test -p manifestation_app -- --nocapture
```

### Level 3: REST API एंडपॉइंट्स फंक्शनल टेस्टिंग
```powershell
# 1. ऐप प्रारंभ करें
cargo run -p manifestation_app -- --port 9090

# 2. सिस्टम स्टेटस व DRM लीज जांच
Invoke-RestMethod -Uri "http://localhost:9090/api/status" -Method Get | ConvertTo-Json

# 3. विजन बोर्ड निर्माण व SD-Turbo विजुअल एंकर रेंडरिंग
Invoke-RestMethod -Uri "http://localhost:9090/api/vision/create" -Method Post -Body '{"title":"Palace Retreat","description":"Royal palace overlooking golden sunset lake","feeling":"Deep Serenity & Abundance","category":"Lifestyle"}' -ContentType "application/json"
Invoke-RestMethod -Uri "http://localhost:9090/api/vision/vis_0001/render" -Method Post | Select-Object success

# 4. 3-6-9 टेस्ला जर्नल व थिटा स्लीप मेडिटेशन
Invoke-RestMethod -Uri "http://localhost:9090/api/gratitude/add" -Method Post -Body '{"entry_type":"Morning_3","text":"Infinite abundance flows through my consciousness."}' -ContentType "application/json"
Invoke-RestMethod -Uri "http://localhost:9090/api/meditation/generate" -Method Post -Body '{"topic":"Peace and Joy"}' -ContentType "application/json"
```

### Level 4: लक्ज़री ग्लासमोर्फिज्म यूआई सत्यापन
- ब्राउज़र में खोलें: `http://localhost:9090`
- लिविंग विजन बोर्ड, 3-6-9 जर्नल, मिरर अफ़र्मेशन और 528 Hz सोल्फ़ेगियो स्लीप मेडिटेशन को इंटरएक्टिव रूप से टेस्ट करें।

---

## 📋 2. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist — क्या-क्या ओके करना है)

डिप्लॉयमेंट से ठीक पहले इन 6 बिंदुओं पर ओके (OK) टिक लगाना अनिवार्य है:

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

## 🚀 3. फाइनल डिप्लॉयमेंट प्रक्रिया (Final Deployment Procedure — फाइनल डिप्लॉय कैसे करना है)

### Step 1: प्रोडक्शन रिलीज़ बाइनरी संकलन
```powershell
cargo build --release -p manifestation_app
```
उत्पादित बाइनरी: `target/release/manifestation_app.exe` (Windows) या `target/release/manifestation_app` (Linux)।

### Step 2: 24x7 ऑटो-स्टार्ट प्रोडक्शन सर्विस डिप्लॉयमेंट

#### A. Windows Task Scheduler (सिस्टम बूट पर ऑटो-स्टार्ट बिना यूजर लॉगिन):
```powershell
schtasks /create /tn "ShambhaviManifestation" `
  /tr "'D:\ai_core_framework - Copy (2)\target\release\manifestation_app.exe' --port 9090 --sd-models-dir 'C:\AI_Models\sd_turbo'" `
  /sc onstart /ru SYSTEM
```

#### B. Linux Systemd Service (`/etc/systemd/system/manifestation.service`):
```ini
[Unit]
Description=Shambhavi Manifestation Super App Engine
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
सक्षम व सक्रिय करें:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now manifestation.service
sudo systemctl status manifestation.service
```
