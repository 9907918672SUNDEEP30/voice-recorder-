# 🎙️ Voice AI Calling Agent - Beginner & Production Guide (`apps/voice_agent`)

यह दस्तावेज़ आपके **Voice AI Calling Agent** का संपूर्ण ऑपरेशनल और टेक्निकल मैनुअल है। यह 100% लोकल ONNX मॉडल्स पर बिना किसी बाहरी क्लाउड API या पायथन पर निर्भर हुए चलता है।

---

## 🏗️ 1. प्रोजेक्ट संरचना (Workspace Monorepo Structure)

आपका कोड एक **Rust Workspace** में व्यवस्थित है:

```text
ai_core_framework/                     <-- Workspace Root
│
├── .cargo/
│   └── config.toml                    <-- MSVC Dynamic CRT (/MD) सेटिंग्स
│
├── crates/
│   └── core/                          <-- ai_core: Universal AI & ONNX Engine
│       ├── Cargo.toml
│       ├── ARCHITECTURE.md            <-- कोर आर्किटेक्चर दस्तावेज़
│       ├── README.md                  <-- कोर गाइड
│       ├── WORK_PROGRESS.md           <-- कोर स्टेटस ट्रैकर
│       └── src/
│           ├── lib.rs
│           ├── engine/                <-- स्टेट मशीन और स्टेप इंजन
│           └── models/
│               ├── adapters.rs        <-- VADAdapter, SttAdapter, LlmAdapter, TtsAdapter
│               ├── onnx_adapters.rs   <-- Silero, Whisper, Qwen, OpenVoice रियल ONNX
│               ├── dsp.rs             <-- Rubato Sinc-Interpolation रीसैंपलिंग
│               └── phonemizer.rs      <-- नेटिव हिंदी फोनेमाइज़र
│
└── apps/
    └── voice_agent/                   <-- voice_agent: Telephony Calling Bot
        ├── Cargo.toml
        ├── ARCHITECTURE.md            <-- वॉइस एजेंट आर्किटेक्चर
        ├── README.md                  <-- यह गाइड
        ├── WORK_PROGRESS.md           <-- वॉइस एजेंट स्टेटस ट्रैकर
        ├── test_client.html           <-- वेब ब्राउज़र से लाइव टेस्टिंग क्लाइंट
        └── src/
            ├── main.rs                <-- ऐप एंट्री पॉइंट, मॉडल्स इनिशियलाइज़ेशन
            ├── telephony.rs           <-- Axum WebSocket सर्वर (Port 3000)
            ├── pipeline.rs            <-- फुल-डुप्लेक्स आर्केस्ट्रेटर, स्टेट मशीन, बार्ज-इन
            ├── codec.rs               <-- G.711 mu-law <-> 16kHz PCM कोडेक
            └── prompts.rs             <-- सिस्टम प्रॉम्प्ट और व्यक्तित्व (Persona)
```

---

## ⚙️ 2. यह काम कैसे करता है? (Data & Streaming Workflow)

यह एप्लिकेशन **300ms से कम लेटेंसी** में कॉलर की बात का जवाब देता है:

1. **Audio Ingestion (कॉल इनपुट):**
   - Telephony (Twilio, Exotel) या वेब ब्राउज़र से 8kHz G.711 $\mu$-law ऑडियो आता है।
2. **Codec Resampling:**
   - `codec.rs` उस ऑडियो को 16kHz 16-bit लीनियर PCM में परिवर्तित करता है।
3. **VAD Listening Check (`SileroVadLocal`):**
   - 512-सैंपल (32ms) चंक्स पर वास्तविक ONNX इनफेरेंस चलता है। जब तक यूज़र बोलता है, ऑडियो बफर होता है। ~1 सेकंड की खामोशी पर यूज़र का टर्न पूरा माना जाता है।
4. **STT Understanding (`WhisperTinyLocal`):**
   - बफर किए गए ऑडियो का 80-बिन Log-Mel स्पेक्ट्रोग्राम बनता है, ONNX एन्कोडर और डिकोडर लूप से वास्तविक टेक्स्ट में ट्रांसक्रिप्शन होता है।
5. **AMD (Voicemail & IVR Cut-off):**
   - यदि ट्रांसक्रिप्ट में "leave a message", "voicemail", या "beep" जैसे शब्द मिलते हैं, तो कॉल तुरंत हैंगअप हो जाती है।
6. **Reasoning & Multi-Turn Brain (`QwenLocal`):**
   - ChatML फॉर्मेट में सिस्टम प्रॉम्प्ट और कन्वर्सेशन हिस्ट्री के साथ Qwen2 LLM में इनपुट जाता है। LLM टोकन्स को स्ट्रीम करता है।
7. **Sentence-by-Sentence TTS Streaming (`OpenVoiceLocal`):**
   - जैसे ही कोई वाक्य समाप्त होता है (`.`, `?`, `!`, `।`), वह वाक्य तुरंत Piper TTS को दिया जाता है।
   - जनरेटेड ऑडियो को 1024-बाइट PCM चंक्स में स्ट्रीम किया जाता है और $\mu$-law में बदलकर कॉलर को तुरंत सुनाई देता है।
8. **Instant Barge-In (यूज़र इंटरप्शन):**
   - यदि बॉट के बोलते समय यूज़र बोलना शुरू करता है, तो VAD तुरंत पहचानता है, एक्टिव टॉस्क कैंसिल हो जाता है, कैरियर को `"clear"` सिग्नल जाता है, और बॉट तुरंत चुप होकर यूज़र को सुनने लगता है।

---

## 📂 3. आवश्यक लोकल मॉडल्स (`C:\AI_Models`)

सभी मॉडल्स आपके कंप्यूटर के `C:\AI_Models` फोल्डर में मौजूद होने चाहिए:

- **VAD:** `C:\AI_Models\silero_vad.onnx`
- **STT Encoder:** `C:\AI_Models\whisper_tiny_onnx\encoder_model_q4.onnx`
- **STT Decoder:** `C:\AI_Models\whisper_tiny_onnx\decoder_model_q4.onnx`
- **STT Tokenizer:** `C:\AI_Models\whisper_tiny_onnx\tokenizer.json`
- **LLM Brain:** `C:\AI_Models\qwen2-vl-2b\llmmodel.onnx`
- **LLM Tokenizer:** `C:\AI_Models\qwen2-vl-2b\tokenizer.json`
- **TTS Base:** `C:\AI_Models\piper_hi\hi_IN-priyamvada-medium-int8.onnx`
- **Tone Color:** `C:\AI_Models\OpenVoice-ONNX-v2\tone_color.onnx`
- **Tone Extract:** `C:\AI_Models\OpenVoice-ONNX-v2\tone_extract.onnx`

---

## 🚀 4. ऐप को कैसे चलाएं? (How to Run)

टर्मिनल (PowerShell या Command Prompt) में वर्कस्पेस रूट से चलाएं:

```bash
cargo run -p voice_agent
```

### सफल आउटपुट:
```text
Voice AI Core Starting...
Loading 100% Local ONNX Stack...
Initializing Silero VAD (Listening Core)...
 - VAD Model: C:\AI_Models\silero_vad.onnx
Initializing OpenVoice (Neutral Core)...
 - Base TTS (Piper): C:\AI_Models\piper_hi\hi_IN-priyamvada-medium-int8.onnx
Initializing Qwen Multimodal LLM (Brain + Vision + OCR Core)...
 - LLM Brain Model: C:\AI_Models\qwen2-vl-2b\llmmodel.onnx
Initializing Whisper STT (Understanding Core)...
ONNX Runtime Initialized.
Telephony WebSocket server listening on ws://0.0.0.0:3000/ws/telephony
```

---

## 🎤 5. ब्राउज़र से लाइव टेस्टिंग (Browser Testing via `test_client.html`)

बिना टेलीफोनी प्रोवाइडर (Twilio/Exotel) के भी आप अपने कंप्यूटर के माइक्रोफोन से सीधे टेस्ट कर सकते हैं:

1. टर्मिनल में सर्वर चालू रखें (`cargo run -p voice_agent`)।
2. `d:\ai_core_framework\apps\voice_agent\test_client.html` फाइल को Chrome या Edge ब्राउज़र में खोलें।
3. **"Start Call"** बटन पर क्लिक करें।
4. माइक्रोफ़ोन की अनुमति (Permission) दें।
5. माइक में बोलें: *"नमस्ते, क्या आप मेरी मदद कर सकती हैं?"*
6. बॉट का जवाब आपके कंप्यूटर स्पीकर से तुरंत सुनाई देगा।
7. बोलते समय बीच में टोक कर देखें (Barge-In) — बॉट तुरंत बोलना बंद करके आपकी नई बात सुनेगा।

---

## 📞 6. टेलीफोनी प्रोवाइडर (Twilio / Exotel) सेटअप

### Twilio TwiML Stream Configuration:
Twilio इनबाउंड कॉल पर निम्नलिखित TwiML रिटर्न करें:
```xml
<Response>
    <Connect>
        <Stream url="wss://your-domain.ngrok-free.app/ws/telephony" />
    </Connect>
</Response>
```

### Exotel Audio Streaming:
Exotel Applet में WebSocket URL सेट करें:
```text
wss://your-domain.ngrok-free.app/ws/telephony
```

---

## 🔧 7. समस्या निवारण (Troubleshooting)

1. **Port 3000 already in use:**
   - यदि पोर्ट 3000 पहले से व्यस्त है, तो `Get-Process -Id (Get-NetTCPConnection -LocalPort 3000).OwningProcess | Stop-Process` चलाएं।
2. **Missing Model File:**
   - सुनिश्चित करें कि सभी 8+ मॉडल्स फ़ाइलें `C:\AI_Models` में सही नाम से मौजूद हैं।
3. **MSVC Linker Errors (`LNK2038`):**
   - `.cargo/config.toml` में `target-feature=-crt-static` और `CFLAGS=/MD` सेट है। हमेशा साफ बिल्ड के लिए `cargo check -p voice_agent` चलाएं।
