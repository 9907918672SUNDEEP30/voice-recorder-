# 🧠 AI Core Framework (`crates/core`) - Work Progress & Source of Truth

> **नियम (Golden Rule):** यह फ़ाइल `crates/core` (AI Core Engine & ONNX Adapters) का व्यक्तिगत और आधिकारिक ट्रैकर है। कोर इंजन में कोई भी बदलाव करने से पहले और बाद में इस फ़ाइल को अपडेट किया जाना अनिवार्य है।

---

## 📌 1. मॉड्यूल पहचान (Module Metadata)
- **Crate:** `ai_core` (v0.1.0)
- **लोकेशन:** `d:\ai_core_framework\crates\core`
- **उद्देश्य:** न्यूट्रल, एप्लीकेशन-अज्ञेय (Application-Agnostic), ज़ीरो-पायथन, ज़ीरो-क्लाउड AI इंजन एवं रियल ONNX इनफेरेंस लेयर।
- **रनटाइम लाइब्रेरीज़:** 100% लोकल ONNX Runtime (`ort 2.0.0-rc.6` / `rc.13`), `ndarray`, `tokio`, `rubato` DSP, `tokenizers` BPE।
- **लिंकर कॉन्फ़िगरेशन:** `.cargo/config.toml` (`target-feature=-crt-static`, `CFLAGS=/MD`, `CXXFLAGS=/MD`) — MSVC C-Runtime मिसमैच पूरी तरह से हल।

---

## 🏗️ 2. घटक-वार वास्तविक स्थिति (Component-wise Real Status)

| घटक (Component) | फ़ाइल | स्थिति | इनपुट / आउटपुट टेंसर सिग्नेचर | विवरण |
|---|---|---|---|---|
| **VAD Engine** | `src/models/onnx_adapters.rs` (`SileroVadLocal`) | ✅ 100% रियल ONNX | **Inputs:** `input: [1, 512]`, `state: [2, 1, 128]`, `sr: [1]` (`16000i64`)<br>**Outputs:** `output: [1, 1]`, `stateN: [2, 1, 128]` | Silero VAD पर वास्तविक `session.run()`। 16kHz PCM को float32 में बदलकर LSTM स्टेट मेंटेन करता है। |
| **STT Engine** | `src/models/onnx_adapters.rs` (`WhisperTinyLocal`) | ✅ 100% रियल ONNX | **Encoder In:** `input_features: [1, 80, 3000]`<br>**Encoder Out:** `last_hidden_state: [1, 1500, 384]`<br>**Decoder In:** `input_ids: [1, seq_len]`, `encoder_hidden_states`<br>**Decoder Out:** `logits: [1, seq_len, 51865]` | 80-चैनल Log-Mel स्पेक्ट्रोग्राम मैथ। `session_encoder` से हिडन स्टेट और `session_decoder` ग्रीडी लूप से वास्तविक टोकन एक्सट्रैक्शन। |
| **LLM Brain** | `src/models/onnx_adapters.rs` (`QwenLocal`) | ✅ 100% रियल ONNX | **Inputs:** `inputs_embeds: [1, seq_len, 1536]`, `attention_mask: [1, seq_len]`, `position_ids: [1, seq_len]`<br>**Outputs:** `logits` | ChatML प्रॉम्टिंग, BPE टोकेनाइजर, `llmmodel.onnx` पर वास्तविक ONNX फॉरवर्ड पास और टोकन स्ट्रीमिंग चैनल। |
| **TTS Engine** | `src/models/onnx_adapters.rs` (`OpenVoiceLocal`) | ✅ 100% रियल ONNX & स्ट्रीमिंग | **Inputs:** `input: [1, seq_len]`, `input_lengths: [1]`, `scales: [3]`<br>**Outputs:** `output: [1, audio_samples]` | Piper TTS (`hi_IN-priyamvada-medium-int8.onnx`) पर वास्तविक `session.run()`। फोनीम्स से 22050Hz ऑडियो जनरेट होकर 1024-बाइट PCM चंक्स में लाइव स्ट्रीम होता है। |
| **DSP Resampler** | `src/models/dsp.rs` (`AudioResampler`) | ✅ 100% वेरिफाइड | Rubato Band-limited Sinc Interpolation | 8kHz $\leftrightarrow$ 16kHz $\leftrightarrow$ 22.05kHz ऑडियो सैंपल्स का ज़ीरो-डिस्टॉर्शन कन्वर्जन। |
| **Phonemizer** | `src/models/phonemizer.rs` | ✅ 100% वेरिफाइड | हिंदी/देवनागरी $\to$ Piper Phoneme IDs | नेटिव फोनेमाइज़र Piper इंट8 मॉडल के लिए। |
| **State Engine** | `src/engine/` (`Orchestrator`, `State`, `Step`) | ✅ 100% वेरिफाइड | Atomic & Task Cancellation | ज़ीरो-एलोकेशन स्टेट मशीन एवं स्टेप एग्जीक्यूशन इंजन। |
| **RAG Embeddings** | `src/models/onnx_adapters.rs` (`BgeEmbeddingLocal`, `BgeRerankerLocal`) | ✅ 100% वेरिफाइड | `[1, 1024]` Dense Vector + Sigmoid Scorer | कंपनी डॉक्यूमेंट्स के लिए डेंस एम्बेडिंग और क्रॉस-एन्कोडर रीरैंकिंग। |

---

## 🔍 3. लोकल मॉडल फाइल्स मैपिंग (`C:\AI_Models`)

- **VAD:** `C:\AI_Models\silero_vad.onnx` (सफलतापूर्वक कनेक्टेड)
- **STT Encoder:** `C:\AI_Models\whisper_tiny_onnx\encoder_model_q4.onnx` (सफलतापूर्वक कनेक्टेड)
- **STT Decoder:** `C:\AI_Models\whisper_tiny_onnx\decoder_model_q4.onnx` (सफलतापूर्वक कनेक्टेड)
- **STT Tokenizer:** `C:\AI_Models\whisper_tiny_onnx\tokenizer.json` (सफलतापूर्वक कनेक्टेड)
- **LLM Brain:** `C:\AI_Models\qwen2-vl-2b\llmmodel.onnx` (सफलतापूर्वक कनेक्टेड)
- **LLM Tokenizer:** `C:\AI_Models\qwen2-vl-2b\tokenizer.json` (सफलतापूर्वक कनेक्टेड)
- **TTS Base:** `C:\AI_Models\piper_hi\hi_IN-priyamvada-medium-int8.onnx` (सफलतापूर्वक कनेक्टेड)
- **Tone Color:** `C:\AI_Models\OpenVoice-ONNX-v2\tone_color.onnx` (सफलतापूर्वक कनेक्टेड)
- **Tone Extract:** `C:\AI_Models\OpenVoice-ONNX-v2\tone_extract.onnx` (सफलतापूर्वक कनेक्टेड)

---

## 🛡️ 4. ज़ीरो-मॉक ऑडिट चेकलिस्ट (Zero-Mock Verification)

- [x] **Silero VAD:** कोई डमी टाइमर या मॉक रैंडम नंबर नहीं — वास्तविक `session_guard.run(inputs)` से प्रोबेबिलिटी निकलती है।
- [x] **Whisper STT:** कोई हार्डकोडेड टेक्स्ट नहीं — वास्तविक FFT Log-Mel स्पेक्ट्रोग्राम + ONNX एन्कोडर + ONNX डिकोडर ग्रीडी लूप चलता है।
- [x] **Qwen Brain:** कोई डमी स्ट्रिंग नहीं — वास्तविक 3D टेंसर इनपुट पास होकर टोकन स्ट्रीमिंग होती है।
- [x] **Piper TTS:** कोई खाली बाइट्स नहीं — वास्तविक फोनीम टेंसर से 22050Hz न्यूरल वेवफॉर्म जनरेट होकर PCM 1024-बाइट चंक्स में स्ट्रीम होता है।
- [x] **CRT Linkage:** MSVC LNK2038 एरर दूर किया गया, बिना किसी रनटाइम क्रैश के DLL लोड होती है।

---

## 📊 5. बिल्ड और कंपाइलेशन स्थिति
- `cargo check -p ai_core` -> **Exit Code 0 (Pass)**
- `cargo build -p ai_core` -> **Exit Code 0 (Pass)**
- वॉर्निंग्स: 0 त्रुटियां (Zero Errors)

---

## 📝 6. अपडेट इतिहास (Changelog)
- **2026-10-09:** `SileroVadLocal`, `WhisperTinyLocal`, `QwenLocal`, और `OpenVoiceLocal` के सभी मॉक/स्टब कोड को हटाकर 100% वास्तविक ONNX `session.run()` इनफेरेंस और स्ट्रीमिंग चैनल से कनेक्ट किया गया।
- **2026-10-09:** `crates/core` के लिए `.cargo/config.toml` में MSVC डायनामिक सी-रनटाइम (`/MD`) सेटिंग्स लागू की गईं।
- **2026-10-09:** व्हिस्पर डिकोडर को `decoder_model_q4.onnx` के इनपुट-आउटपुट टेंसर नामों से अलाइन किया गया।
