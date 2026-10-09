# 🧭 Master Work Progress & Workspace Directory

> **सख्त हिदायत (Strict Directive):** किसी भी बातचीत या कार्य को शुरू करने से पहले, एजेंट को इस फ़ाइल तथा इसके अंतर्गत दोनों समर्पित प्रोग्रेस फ़ाइलों, आर्किटेक्चर और README को पढ़ना अनिवार्य है।

---

## 📂 वर्कस्पेस की 6 मुख्य दस्तावेज़ फ़ाइलें (Core 6 Documentation Files)

### 1. AI Core Framework (`crates/core`)
- 👉 **[`crates/core/ARCHITECTURE.md`](file:///d:/ai_core_framework/crates/core/ARCHITECTURE.md)** — कोर ML इंजन का आर्किटेक्चर, टेंसर सिग्नेचर और DSP ब्लूप्रिंट।
- 👉 **[`crates/core/README.md`](file:///d:/ai_core_framework/crates/core/README.md)** — कोर फ्रेमवर्क की विस्तृत गाइड, मॉडल डायरेक्टरी लेआउट और MSVC बिल्ड निर्देश।
- 👉 **[`crates/core/WORK_PROGRESS.md`](file:///d:/ai_core_framework/crates/core/WORK_PROGRESS.md)** — कोर इंजन का आधिकारिक सोर्स ऑफ ट्रूथ और ज़ीरो-मॉक ऑडिट।

### 2. Voice AI Calling Agent (`apps/voice_agent`)
- 👉 **[`apps/voice_agent/ARCHITECTURE.md`](file:///d:/ai_core_framework/apps/voice_agent/ARCHITECTURE.md)** — फुल-डुप्लेक्स टेलीफोनी आर्किटेक्चर, स्टेट मशीन, बार्ज-इन और सीक्वेंस डायग्राम।
- 👉 **[`apps/voice_agent/README.md`](file:///d:/ai_core_framework/apps/voice_agent/README.md)** — वॉइस एजेंट का उपयोग, ब्राउज़र टेस्टिंग और टेलीफोनी सेटअप गाइड।
- 👉 **[`apps/voice_agent/WORK_PROGRESS.md`](file:///d:/ai_core_framework/apps/voice_agent/WORK_PROGRESS.md)** — वॉइस एजेंट का आधिकारिक सोर्स ऑफ ट्रूथ और लेटेंसी बेंचमार्क।

---

## 📌 वर्कस्पेस स्थिति सारांश (Workspace Status Summary)

- **लोकल मॉडल्स पाथ:** `C:\AI_Models` (100% वेरिफाइड एवं अपरिवर्तनीय)
- **बिल्ड स्टेटस:** `cargo check --workspace` & `cargo build -p voice_agent` -> **Exit Code 0 (सफलतापूर्वक कंपाइल हुआ)**
- **रन स्टेटस:** `cargo run -p voice_agent` -> **Exit Code 0 (सारे मॉडल्स RAM में लोड होकर सर्वर `ws://0.0.0.0:3000/ws/telephony` पर एक्टिव लिसन कर रहा है)**
- **मॉडल्स इनफेरेंस स्थिति (100% Real ONNX):**
  - **Silero VAD:** ✅ रियल ONNX `session.run()` कनेक्टेड (512 f32 सैंपल्स, LSTM स्टेट `[2, 1, 128]`)।
  - **Whisper STT:** ✅ 80-बिन Log-Mel + रियल ONNX एन्कोडर/डिकोडर लूप कनेक्टेड।
  - **Qwen LLM:** ✅ ChatML + BPE + रियल ONNX 3D टेंसर फॉरवर्ड पास एवं टोकन स्ट्रीमिंग कनेक्टेड।
  - **Piper TTS:** ✅ रियल ONNX फोनीम सिंथेसिस + 1024-बाइट PCM ऑडियो स्ट्रीमिंग चैनल कनेक्टेड।
- **MSVC सी-रनटाइम कॉन्फ़िगरेशन:** `.cargo/config.toml` (`target-feature=-crt-static`, `CFLAGS=/MD`, `CXXFLAGS=/MD`) सक्रिय।
