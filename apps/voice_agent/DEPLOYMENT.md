# 🚀 Voice AI Calling Agent — Deployment Guide (`apps/voice_agent`)
## Shambhavi AI Labs — Cloud Telephony Engine & Next.js PWA Dashboard

---

## 🖥️ 1. फ्रंटएंड व क्लाइंट आर्किटेक्चर (Frontend & Client Architecture)

| घटक | विवरण |
|---|---|
| **क्लाउड बैकएंड इंजन** | 100% शुद्ध रस्ट बाइनरी (`voice_agent`), < 150 MB RAM, 0% पायथन, लिनक्स VPS / डॉकर पर 24/7 डिप्लॉयड |
| **कस्टमर डैशबोर्ड** | **Next.js PWA वेब व मोबाइल ऐप** (डेस्कटॉप ब्राउज़र, Android व iOS पर 1-क्लिक इंस्टॉल) |
| **डेटाबेस व नॉलेज बेस** | **Supabase** (PostgreSQL + pgvector + BGE Small 384-dim Embeddings + BGE Reranker) |
| **डैशबोर्ड क्षमताएं** | रियल-टाइम लाइव कॉल मॉनिटर, ऑडियो वेवफॉर्म, सब-300ms लाइव ट्रांसक्रिप्ट स्ट्रीम, पर्सोना/सिस्टम प्रॉम्प्ट एडिटर, कॉल रिकॉर्डिंग्स प्लेयर, और कॉल मिनट्स बिलिंग मीटर |

---

## 🧪 2. टेस्टिंग गाइड (How to Test & Verify — आप खुद कैसे टेस्ट करेंगे)

### Level 1: स्टेटिक कंपाइलेशन व ऑडिट
```powershell
cargo check -p voice_agent
cargo clippy -p voice_agent -- -D warnings
```

### Level 2: ऑटोमेटेड यूनिट व टेलीफोनी टेस्ट सुइट
```powershell
cargo test -p voice_agent -- --nocapture
```

### Level 3: हैंड्स-ऑन लाइव इंटरएक्टिव माइक टेस्ट (Drive the Voice Agent)
1. **वॉयस एजेंट प्रारंभ करें:**
   ```powershell
   cargo run -p voice_agent
   ```
2. **ब्राउज़र टेस्ट क्लाइंट खोलें:**
   - फ़ाइल खोलें: `apps/voice_agent/test_client.html` अपने क्रोम ब्राउज़र में।
   - **Connect** बटन दबाएं (WebSocket `ws://127.0.0.1:3000/ws` कनेक्ट होगा)।
   - अपने हेडफोन माइक से बोलें: *"नमस्ते, क्या आप मेरी कंपनी के लिए अपॉइंटमेंट बुक कर सकते हैं?"*
   - स्क्रीन पर देखें:
     - **Silero VAD:** आवाज़ शुरू होते ही ग्रीन इंडिकेटर (< 5ms)।
     - **Whisper STT:** स्क्रीन पर रियल-टाइम हिंदी/इंग्लिश टेक्स्ट स्ट्रीम (< 150ms)।
     - **Qwen LLM + OpenVoice TTS:** 300ms के भीतर हेडफोन में स्पष्ट आवाज़ में उत्तर।
     - **Barge-in:** जब बॉट बोल रहा हो, आप बीच में बोलें; बॉट तुरंत चुप होकर आपकी नई बात सुनेगा।

---

## 📋 3. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist)

- [ ] **1. Zero Compiler Warnings:** `cargo check --release -p voice_agent` में 0 एरर और 0 चेतावनियां।
- [ ] **2. ONNX मॉडल्स पूर्णता (`C:\AI_Models\`):**
  - `silero_vad.onnx` (< 2 MB)
  - `whisper_tiny_onnx/` (Encoder, Decoder, Tokenizer)
  - `qwen2-vl-2b/` (LLM मॉडल व टोकेनाइज़र)
  - `piper_hi/` या `OpenVoice-ONNX-v2/`
  - `bge_small_embedding/` व `bge_reranker/`
- [ ] **3. Supabase स्कीमा माइग्रेशन:** `supabase_schema.sql` Supabase में रन हो चुका है और `.env` में `SUPABASE_URL` व `SUPABASE_KEY` सेट हैं।
- [ ] **4. 24-घंटे DRM लीज:** `crates/drm_guard` की 24h लीज और 10-मिनट हार्टबीट पिंग `VoiceAgentMetrics` के साथ सक्रिय है।
- [ ] **5. मेमोरी सीलिंग:** 100 लगातार कॉल्स पर RAM उपयोग **< 150 MB** पर स्थिर (Zero Memory Leaks)।

---

## 🚀 4. प्रोडक्शन डिप्लॉयमेंट प्रक्रिया (Final Production Deployment)

### Step 1: प्रोडक्शन रिलीज़ बाइनरी संकलन
```powershell
cargo build --release -p voice_agent
```
बाइनरी पाथ: `target/release/voice_agent.exe` (Windows) या `target/release/voice_agent` (Linux)।

### Step 2: Linux Headless Server / Systemd सर्विस डिप्लॉयमेंट
`/etc/systemd/system/voice_agent.service`:
```ini
[Unit]
Description=Shambhavi Voice AI Telephony Cloud Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/shambhavi/voice_agent
ExecStart=/opt/shambhavi/voice_agent/voice_agent
Restart=always
RestartSec=3
LimitNOFILE=65536
MemoryMax=500M

[Install]
WantedBy=multi-user.target
```
सक्रिय करें:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now voice_agent.service
sudo systemctl status voice_agent.service
```

### Step 3: Next.js PWA कस्टमर डैशबोर्ड डिप्लॉयमेंट
```bash
cd apps/voice_agent/dashboard
npm install
npm run build
pm2 start npm --name "voice-pwa-dashboard" -- start -- -p 3000
```
कस्टमर अब अपने मोबाइल या लैपटॉप पर `https://voice.yourdomain.com` खोलकर PWA इंस्टॉल कर सकते हैं।
