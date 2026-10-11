# 🎥 AI Video Studio (`apps/video_studio`) - Source of Truth

> **नियम:** यह फ़ाइल `apps/video_studio` (AI वर्चुअल कैमरामैन, टेलीप्रॉम्प्टर आई-कॉन्ट्रैक्ट और सिनेमैटिक 3D LUT इंजन) का आधिकारिक प्रोग्रेस ट्रैकर और सोर्स ऑफ ट्रुथ है।

---

## 📌 उत्पाद विवरण (Product Specification)
- **पैकेज नाम:** `video_studio` (v0.1.0)
- **लोकेशन:** `d:\ai_core_framework - Copy (2)\apps\video_studio`
- **आधारभूत सिद्धांत:** Core Engine First • Pure Rust • 100% Decoupled from Audio/DSP • Zero Python • Zero OpenCV
- **आर्किटेक्चरल ब्लूप्रिंट:** [ARCHITECTURE.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/video_studio/ARCHITECTURE.md)
- **यूजर गाइड:** [README.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/apps/video_studio/README.md)

---

## 🏗️ मॉड्यूल स्थिति एवं तकनीकी ऑडिट (Module Status & Audit)

- [x] **`Cargo.toml`:** वर्कस्पेस एकीकरण एवं सटीक रस्ट डिपेंडेंसीज (`ai_core`, `nokhwa` 0.10.11, `image` 0.25, `tokio`, `crossbeam-channel`, `clap` 4.6).
- [x] **`src/capture/camera.rs`:** 
  - `CameraSource` डायनामिक ट्रेट इंटरफेस।
  - `VideoFrame` कंटीगुअस इंटरलीव्ड RGB24 बफर प्रबंधन (ज़ीरो-कॉपी पिक्सेल एक्सेस)।
  - `NokhwaCamera` विंडोज मीडिया फाउंडेशन (WMF COM) हार्डवेयर वेबकैम एब्स्ट्रैक्शन (थ्रेड-सेफ एनकैप्सुलेशन)।
  - `VirtualCamera` 60 FPS सिंथेटिक 1080p स्टूडियो सिम्युलेटर (मूविंग प्रेज़ेंटर, आई-सॉकेट्स, प्यूपिल्स, आईलिड्स एवं स्टूडियो बैकग्राउंड)।
- [x] **`src/tracking/face_detector.rs`:** 
  - हाई-स्पीड पिक्सल-क्रोमिनेंस सेंट्रोइड फेस ट्रैकर (`FaceDetector`, `FaceBox`, `FacialLandmarks`)।
  - एंथ्रोपोमॉर्फिक आई-सॉकेट एवं लैंडमार्क एक्सट्रैक्शन।
- [x] **`src/tracking/auto_framing.rs`:** 
  - वर्चुअल AI कैमरामैन PTZ इंजन (`AutoFramer`, `AutoFramingConfig`, `FramingBox`)।
  - **Deadzone Jitter Filter:** सांस लेने या सिर के हल्के कम्पन से कैमरा हिलने से रोकने हेतु $D_{\text{thresh}} = 0.04$ थ्रेशोल्ड।
  - **EMA Motion Damping:** फ्लूइड-हेड सिनेमैटिक पैनिंग हेतु एक्सपोनेंशियल मूविंग एवरेज ($\alpha = 0.08$)।
  - **Sub-Pixel Bilinear Resampler:** फ्लोटिंग-पॉइंट सब-पिक्सल बाइलीनियर क्रॉपिंग व री-स्केलिंग।
- [x] **`src/gaze/iris_tracker.rs`:** 
  - आई-सॉकेट इलिप्टिकल बाउंडिंग बॉक्सिंग (`IrisTracker`, `EyeRegion`)।
  - ल्यूमिनेंस मिनिमम ग्रेडिएंट आधारित डार्क प्यूपिल लोकलाइजेशन।
  - आईलिड क्लोजर / ब्लिंक डिटेक्शन।
- [x] **`src/gaze/gaze_warp.rs`:** 
  - टेलीप्रॉम्प्टर आई-कॉन्ट्रैक्ट करेक्शन इंजन (`GazeCorrector`, `GazeCorrectionConfig`)।
  - बैकवर्ड ऑप्टिकल एक्सिस डिस्प्लेसमेंट वेक्टर शिफ्ट ($\Delta x, \Delta y$)।
  - **Cosine Feathering Mask:** अनकैनी वैली (रोबोटिक आंखें) रोकने हेतु कंटीन्यूअस कोसाइन एनवेलप ($w(r)$) ब्लेंडिंग।
  - **Blink Safety System:** पलक झपकते ही 3-फ्रेम ग्रैजुअल डिके द्वारा टियरिंग व डिस्टॉर्शन से 100% सुरक्षा।
- [x] **`src/grading/lut3d.rs`:** 
  - $17 \times 17 \times 17$ डिस्क्रीट क्यूब वॉल्यूम 3D LUT ग्रेडिंग इंजन (`Lut3D`, `LutPreset`)।
  - 8-कॉर्नर ट्रिलिनियर वॉल्यूम इंटरपोलेशन मैथेमैटिकल फॉर्मूला।
  - 4 सिनेमैटिक प्रीसेट्स: `TealAndOrange`, `WarmStudio`, `CleanMinimal`, `FilmPunch`।
- [x] **`src/grading/skin_smooth.rs`:** 
  - $5 \times 5$ नेबरहुड वेरिएंस-गाइडेड एज-प्रिजर्विंग स्किन स्मूथिंग (`SkinFilter`)।
  - कम वेरिएंस (स्किन) को स्मूथ और अधिक वेरिएंस (आंखें, होंठ, बाल) को 100% शार्प रखने वाला वेटिंग सिस्टम।
  - स्टूडियो रेडियल फेस री-लाइटिंग सॉफ्ट एक्सपोज़र फॉलऑफ मास्क।
- [x] **`src/storage/frame_writer.rs`:** 
  - लॉसलेस PNG सिंगल-फ्रेम स्नैपशॉट राइटर (`FrameWriter`)।
  - हाई-स्पीड PPM (P6 बाइनरी) फ्रेम सीक्वेंस डिस्क रिकॉर्डर।
- [x] **`src/pipeline/video_pipeline.rs`:** 
  - 60 FPS रियल-टाइम 5-स्टेज कोऑर्डिनेटर (`StudioVideoPipeline`, `StudioPipelineConfig`, `PipelineTelemetry`)।
  - बैकप्रेशर प्रोटेक्शन एवं रियल-टाइम टेलीमेट्री (FPS, फ्रेम लेटेंसी ms, फेस लॉक, आई-कॉन्ट्रैक्ट स्टेटस)।
- [x] **`src/main.rs`:** 
  - प्रोडक्शन सीएलआई इंजन (`--devices`, `--camera`, `--virtual`, `--lut`, `--gaze`, `--snapshot`, `--record-frames`)।
  - रियल-टाइम ASCII कंसोल डैशबोर्ड।
- [x] **`tests/video_tests.rs`:** 
  - 7 संपूर्ण यूनिट व इंटीग्रेशन टेस्ट्स (100% टेस्ट पास)।

---

## 🧪 सत्यापन एवं टेस्ट परिणाम (Verification Log)

### 1. यूनिट एवं इंटीग्रेशन टेस्ट्स (7/7 Pass)
```text
$ cargo test -p video_studio
running 7 tests
test test_video_frame_pixel_access            ... ok
test test_lut3d_trilinear_interpolation        ... ok
test test_skin_smoothing_and_relighting       ... ok
test test_face_detector_on_virtual_camera      ... ok
test test_auto_framing_ema_damping_and_rescale ... ok
test test_iris_tracker_and_gaze_warper         ... ok
test test_end_to_end_virtual_camera_pipeline   ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
```

### 2. पूरे वर्कस्पेस का संकलन ऑडिट (0 Errors, 0 Warnings)
```text
$ cargo check --workspace
   Checking ai_core v0.1.0
   Checking dsp_recorder v0.1.0
   Checking video_studio v0.1.0
   Checking runtime_cli v0.1.0
   Checking ai_core_framework v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

---

## 🔒 आर्किटेक्चरल सील व लॉक (Architecture Locked & Sealed)
1. **Core Engine First & Pure Rust:** संपूर्ण विज़न, पीटीजेड और ग्रेडिंग एल्गोरिदम 100% शुद्ध रस्ट में हस्तनिर्मित हैं। कोई पायथन या भारी C++/OpenCV निर्भरता नहीं है।
2. **पूर्ण स्वायत्तता (Decoupled Independence):** `apps/video_studio` और `apps/dsp_recorder` एक-दूसरे से पूर्णतः स्वतंत्र हैं। इन्हें अलग-अलग चलाया व टेस्ट किया जा सकता है।
3. **शून्य अधूरा कोड (No Lazy Code):** प्रत्येक फॉर्मूला, इंटरपोलेशन लूप और बाउंडेड बफर पूर्ण रूप से प्रोडक्शन-ग्रेड रस्ट में लागू है।
