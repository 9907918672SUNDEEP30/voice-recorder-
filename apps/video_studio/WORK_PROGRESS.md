# 🎥 AI Video Studio (`apps/video_studio`) - Source of Truth

> **नियम:** यह फ़ाइल `apps/video_studio` (AI वर्चुअल कैमरामैन, आई-कॉन्ट्रैक्ट और सिनेमैटिक 3D LUT इंजन) का आधिकारिक प्रोग्रेस ट्रैकर और सोर्स ऑफ ट्रुथ है।

---

## 📌 उत्पाद विवरण (Product Specification)
- **पैकेज नाम:** `video_studio` (v0.1.0)
- **लोकेशन:** `d:\ai_core_framework - Copy (2)\apps\video_studio`
- **आधारभूत सिद्धांत:** Core Engine First • Pure Rust • 100% Decoupled from Audio/DSP
- **आर्किटेक्चरल ब्लूप्रिंट:** [ARCHITECTURE.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/video_studio/ARCHITECTURE.md)
- **यूजर गाइड:** [README.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/video_studio/README.md)

---

## 🏗️ मॉड्यूल स्थिति (Module Status)
- [x] **`Cargo.toml`:** वर्कस्पेस इंटीग्रेशन एवं लीन डिपेंडेंसीज (`ai_core`, `nokhwa`, `image`, `tokio`, `crossbeam-channel`).
- [x] **`src/capture/camera.rs`:** `CameraSource` ट्रेट, `NokhwaCamera` (हार्डवेयर OS कैप्चर), एवं `VirtualCamera` (सिंथेटिक 1080p स्टूडियो सिमुलेटर)।
- [x] **`src/tracking/face_detector.rs`:** प्योर रस्ट हाई-स्पीड फेस व लैंडमार्क डिटेक्टर।
- [x] **`src/tracking/auto_framing.rs`:** AI कैमरामैन PTZ, Deadzone जिटर डैम्पिंग, EMA स्मूथिंग, एवं सब-पिक्सल बायलीनियर रीस्केलर।
- [x] **`src/gaze/iris_tracker.rs`:** आई-सॉकेट एवं पुतली (Iris/Pupil) लोकलाइजेशन।
- [x] **`src/gaze/gaze_warp.rs`:** टेलीप्रॉम्प्टर आई-कॉन्ट्रैक्ट करेक्शन, कोसाइन फेदरिंग ब्लेंड एवं ब्लिंक सेफ्टी।
- [x] **`src/grading/lut3d.rs`:** 3D क्यूब LUT ट्रिलिनियर वॉल्यूम इंटरपोलेशन एवं सिनेमैटिक प्रीसेट्स (`teal-orange`, `warm`, `minimal`, `film`)।
- [x] **`src/grading/skin_smooth.rs`:** एज-प्रिजर्विंग गाइडेड स्किन फिल्टर एवं स्टूडियो रेडियल री-लाइटिंग।
- [x] **`src/storage/frame_writer.rs`:** लॉसलेस PNG स्नैपशॉट एवं PPM फ्रेम सीक्वेंस रिकॉर्डर।
- [x] **`src/pipeline/video_pipeline.rs`:** 60 FPS कैपेबल कोऑर्डिनेटर एवं रियल-टाइम टेलीमेट्री (FPS, Latency, Face Lock, Gaze Active)।
- [x] **`src/main.rs`:** इंटरएक्टिव सीएलआई स्टूडियो एवं रियल-टाइम कंसोल डैशबोर्ड।
- [x] **`tests/video_tests.rs`:** 7 संपूर्ण यूनिट व इंटीग्रेशन टेस्ट्स।
