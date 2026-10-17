# 🏛️ शांभवी AI लैब्स — मास्टर डिप्लॉयमेंट आर्किटेक्चर (FINAL_DEPLOYMENT_GUIDE)

> **आर्किटेक्चरल सिद्धांत:** वेब की फ्लेक्सिबिलिटी (PWA/Next.js) और रस्ट की नेटिव परफॉर्मेंस (Rust Binary / DirectML GPU / Zero Python) का एक अभेद्य और व्यावसायिक संयोजन।

---

## 🗺️ समग्र डिप्लॉयमेंट वर्गीकरण (Master Ecosystem Map)

```
                                [ Shambhavi AI Ecosystem ]
                                            |
    +-----------------------+---------------+-----------------------+
    |                       |               |                       |
[ 🌐 Cloud & Web ]     [ 📦 Appliance ]  [ 🖥️ Native Apps ]   [ 📱 PWA + Rust Engine ]
- Voice AI Agent       - CCTV Guard     - DSP Recorder        - Financial Engine
- Next.js Dashboard      (Black-Box)    - Video Studio        - Local RAG Engine
- Supabase Sync                         - Manifestation App*
- Content Automation*
```
*(चिह्नित `*` ऐप्स आगामी नियोजित प्रोजेक्ट्स हैं जिन पर काम बाद में शुरू होगा)*

---

## 🚀 श्रेणी 1: [ 🌐 Cloud & Web ] — वॉयस AI कॉलिंग एजेंट, कंटेंट ऑटोमेशन एवं Next.js PWA पोर्टल

> **समान ट्विन आर्किटेक्चर (Twin Architecture Pattern):**  
> `वॉयस AI कॉलिंग एजेंट` और `कंटेंट ऑटोमेशन ऐप` दोनों बिल्कुल एक समान 3-स्तरीय आधुनिक क्लाउड आर्किटेक्चर का पालन करते हैं:  
> `[ 100% Rust Cloud Binary ] <---> [ Supabase (PostgreSQL + pgvector) ] <---> [ Next.js PWA Dashboard ]`

### 1. 🎙️ वॉयस AI कॉलिंग एजेंट (`apps/voice_agent`)
- **कोर रस्ट बाइनरी (Cloud Backend Engine):**
  - क्लाउड VPS / Linux Server (Ubuntu 22.04 LTS / Docker container) पर 100% शुद्ध **Rust Binary** चलेगी।
  - मेमोरी फुटप्रिंट: < 150 MB RAM (0% पायथन, सिलारो VAD, व्हिस्पर STT, क्वेन LLM, पाइपर TTS).
- **डेटाबेस व नॉलेज सिंक (Supabase Integration):**
  - रस्ट इंजन सीधे Supabase डेटाबेस से जुड़ा रहता है (`apps/voice_agent/supabase_schema.sql`)।
  - कॉल लॉग्स, टाइमस्टैम्प, ऑडियो रिकॉर्डिंग लिंक्स और फुल डायलॉग ट्रांसक्रिप्ट्स रियल-टाइम में Supabase में सेव होते हैं।
  - कंपनी की नॉलेज बेस (FAQs, रेट लिस्ट) **BGE Small Embeddings** (384-dim) और **BGE Small Reranker** द्वारा सब-15ms में रिट्रीव होकर कॉल में उपयोग होती है।
- **कस्टमर डैशबोर्ड (Next.js PWA Dashboard):**
  - ग्राहक के लिए आधुनिक वेब पोर्टल जो **PWA (Progressive Web App)** के रूप में चलेगा।
  - ग्राहक इसे अपने मोबाइल (Android/iOS), टैबलेट या कंप्यूटर पर बिना किसी ऐप स्टोर के 1-क्लिक में नेटिव ऐप की तरह इंस्टॉल कर सकते हैं।
  - Supabase Realtime से लाइव कॉल्स, ट्रांसक्रिप्ट्स, एजेंट पर्सोना और कॉल मिनट्स एनालिटिक्स दिखाता है।
- **लाइसेंसिंग एवं मीटरिंग:**
  - `crates/drm_guard` द्वारा 24-घंटे का रोटेटिंग क्रिप्टोग्राफिक लीज टोकन।
  - मीटरिंग टेलीमेट्री: **कुल पूर्ण कॉल्स और कॉल मिनट्स**।

### 2. 🤖 कंटेंट ऑटोमेशन ऐप (`apps/content_automation`) — [🔜 आगामी / Planned for Next Phase]
> **नोट (सख्त निर्देश):** यह ऐप केवल आर्किटेक्चर एवं फाइनल डिप्लॉयमेंट रोडमैप में नोट किया गया है। इस ऐप पर अभी काम शुरू नहीं करना है। इसे आगामी चरण में बनाया जाएगा।
- **कोर रस्ट बाइनरी (Engine):** 
  - वॉयस एजेंट की तरह ही यह भी एक 100% शुद्ध **Rust Binary** बनेगी जो बैकग्राउंड में हाई-परफॉर्मेंस कंटेंट जेनरेशन, वेब स्क्रैपिंग, और सोशल मीडिया API पब्लिशिंग करेगी (0% पायथन)।
- **डेटाबेस व स्टोरेज (Supabase Integration):**
  - रस्ट बाइनरी सीधे Supabase (PostgreSQL + pgvector) से जुड़ी होगी, जहाँ शेड्यूल्ड आर्टिकल्स, सोशल मीडिया पोस्ट्स, रील्स स्क्रिप्ट्स, और मीडिया मेटाडेटा सुरक्षित रूप से स्टोर व सिंक होंगे।
- **कस्टमर डैशबोर्ड (Next.js PWA Dashboard):**
  - ग्राहक के लिए आधुनिक **Next.js PWA ऐप** बनेगा जो मोबाइल और डेस्कटॉप पर नेटिव ऐप की तरह इंस्टॉल होकर चलेगा।
  - यह Supabase से जुड़कर विजुअल कंटेंट कैलेंडर, बल्क पोस्टिंग, AI प्रॉम्प्ट कस्टमाइज़र, और परफॉर्मेंस एनालिटिक्स दिखाएगा।
- **लाइसेंसिंग एवं क्लाउड लीज:** `crates/drm_guard` द्वारा 24-घंटे की क्रिप्टोग्राफिक लीज + OTA ऑटो-अपडेटर (कंटेंट पोस्ट्स / जेनरेशन मीटरिंग)।

---

## 📦 श्रेणी 2: [ 📦 Appliance ] — ऑन-प्रिमाइसेस ब्लैक-बॉक्स एप्लायंस

### 3. 🛡️ सीसीटीवी गार्ड (`apps/cctv_guard` / `apps/nvr_core`) — पहरेदार ब्लैक-बॉक्स
- **डिप्लॉयमेंट रूपरेखा:**
  - ग्राहक के यहाँ (दुकान, बैंक, शोरूम, वेयरहाउस) लगने वाला **कंपनी का अपना समर्पित हार्डवेयर + सॉफ्टवेयर ब्लैक-बॉक्स** (Minimal Dual-Core/Celeron CPU, 2 GB RAM, SSD)।
  - ज़ीरो-क्लाउड 24/7 रिकॉर्डिंग और HDMI डायरेक्ट डिस्प्ले।
- **10-मिनट प्रोएक्टिव हेल्थ पिंग (Health Telemetry):**
  - बॉक्स हर 10 मिनट में क्लाउडफ्लेयर वर्कर (`/api/telemetry/heartbeat-10m`) को पिंग करता है।
  - यदि तार ढीला हुआ (Loose Wire), कैमरा डिस्कनेक्ट हुआ या डिस्क 90% भरी, तो क्लाउड तुरंत ग्राहक के मोबाइल पर **WhatsApp / वर्चुअल नंबर SMS** अलर्ट भेजता है।
- **24-घंटे क्रिप्टोग्राफिक लीज एवं सिक्योरिटी लॉक:**
  - पूरी तरह एनक्रिप्टेड और हार्डवेयर बाउंड (`SHA256(MB UUID + CPU ID + MAC)`).
  - हर 24 घंटे में क्लाउड से नई AES-256 सेशन की लेता है।
  - ग्राहक द्वारा पेमेंट रोकने पर volatile RAM की तुरंत `ZeroizeOnDrop` द्वारा मिटा दी जाती है और AI गार्ड लॉक हो जाता है।
- **OTA ऑटो-अपडेटर:**
  - बैकग्राउंड में SHA-256 वैलिडेटेड बाइनरी डाउनलोड और एटॉमिक स्वैप (Zero Downtime)।
- **मीटरिंग:** सक्रिय कैमरों की संख्या (Active Camera Count)।

---

## 🖥️ श्रेणी 3: [ 🖥️ Native Apps ] — हाई-परफॉर्मेंस नेटिव रस्ट ऐप्स

### 4. 🎬 वीडियो स्टूडियो (`apps/video_studio`)
- **डिप्लॉयमेंट रूपरेखा:**
  - विंडोज़ / मैक / लिनक्स डेस्कटॉप के लिए पूरी तरह **Native Rust Executable** (`video_studio.exe`)।
  - किसी बाहरी हार्डवेयर बॉक्स या महंगे GPU की कोई ज़रूरत नहीं; मौजूदा वेबकैम और पीसी/लैपटॉप पर चलता है।
- **मुख्य क्षमताएं:**
  - डायरेक्ट WMF/V4L2 कैमरा ड्राइवर एक्सेस, वर्चुअल AI कैमरामैन PTZ (Deadzone + EMA डैम्पिंग)।
  - टेलीप्रॉम्प्टर आई-कॉन्टैक्ट करेक्शन (Cosine feathering mask) और 17x17x17 3D LUT ग्रेडिंग।
- **लाइसेंसिंग:** 24-घंटे की क्रिप्टोग्राफिक लीज + OTA ऑटो-अपडेटर (रेंडर मिनट्स मीटरिंग)।

### 5. 🎛️ वॉयस स्टूडियो / डीएसपी रिकॉर्डर (`apps/dsp_recorder`)
- **डिप्लॉयमेंट रूपरेखा:**
  - पीसी (Windows/Mac/Linux) और मोबाइल के लिए **Native Rust Audio Driver App** (`dsp_recorder.exe`)।
  - यूज़र के मौजूदा हेडफोन या यूएसबी माइक पर चलता है।
- **मुख्य क्षमताएं:**
  - ज़ीरो-लेटेंसी ऑडियो कैप्चर (< 5.3ms बफर लेटेंसी)।
  - RBJ बाइक्वाड ईक्यू, स्पेक्ट्रल टोन क्लोनिंग, और 24-बिट PCM लॉसलेस वेव रिकॉर्डिंग।
- **लाइसेंसिंग:** 24-घंटे की क्रिप्टोग्राफिक लीज + OTA ऑटो-अपडेटर (रिकॉर्डिंग आवर्स मीटरिंग)।

### 6. 🧘 मैनिफेस्टेशन सुपर ऐप (`apps/manifestation_app`)
- **डिप्लॉयमेंट रूपरेखा:**
  - 100% शुद्ध **Native Rust Binary** + एम्बेडेड लक्ज़री ग्लासमोर्फिज्म वेब पोर्टल (`http://localhost:9090`)।
  - किसी बाहरी Node.js, NPM या CDN की आवश्यकता नहीं; संपूर्ण HTML5/CSS3/JS बाइनरी में एम्बेडेड है।
- **मुख्य क्षमताएं:**
  - **"Core-Engine First" AI इमेज जनरेशन:** `crates/core` (`ai_core`) के `ImageGenAdapter` व SD-Turbo Safetensors इंजन द्वारा सिंगल-स्टेप 512x512 लिविंग विजन बोर्ड सिंथेसिस।
  - **निकोला टेस्ला 3-6-9 मैनिफेस्टेशन जर्नल:** सुबह (3x), दोपहर (6x), रात (9x) इंटेंशन लॉगर व वाइब्रेशनल रेजोनेंस ट्रैकर।
  - **मिरर अफ़र्मेशन चैंबर:** रियल-टाइम स्पीच/टेक्स्ट कंविक्शन स्कोरिंग व सबकॉन्शियस इंप्रिंटिंग फीडबैक।
  - **थिटा स्लीप मेडिटेशन इंजन:** 528 Hz (ट्रांसफॉर्मेशन व मिरेकल्स) व 432 Hz सोल्फ़ेगियो फ़्रीक्वेंसी स्क्रिप्ट्स + 6.0 Hz थिटा वेव ब्रेनवेव एंट्रेनमेंट।
- **लाइसेंसिंग एवं मीटरिंग:**
  - `crates/drm_guard` द्वारा 24-घंटे की क्रिप्टोग्राफिक लीज + 24h ऑफलाइन ग्रेस।
  - `ManifestationMetrics`: अफ़र्मेशन काउंट, मेडिटेशन मिनट्स, और जनरेटेड विजन बोर्ड्स की ऑटोमैटिक मीटरिंग।


---

## 📱 श्रेणी 4: [ 📱 PWA + Rust Engine ] — वेब रीच + नेटिव रस्ट स्पीड

### 7. 📈 फाइनेंशियल सुपर इंजन (`apps/financial_engine`)
- **डिप्लॉयमेंट रूपरेखा:**
  - यूजर इंटरफ़ेस: **Progressive Web App (PWA)** — किसी भी मोबाइल, टैबलेट या कंप्यूटर ब्राउज़र में 1-क्लिक इंस्टॉल।
  - बैकएंड इंजन: इसके पीछे सुपर-फास्ट **Pure Rust Binary** काम करती है।
- **मुख्य क्षमताएं:**
  - 5,000+ NSE/BSE कंपनियों का पूरा फाइनेंशियल यूनिवर्स इन-मेमरी कैश (`DashMap`) में < 75 MB RAM में।
  - 100% फ्री पब्लिक डेटा स्क्रेपिंग (CDP + Bhavcopy), डिटरमिनिस्टिक रेश्यो मैथ (< 50ms रिस्पॉन्स)।
- **लाइसेंसिंग एवं मीटरिंग:**
  - 24-घंटे क्रिप्टोग्राफिक लीज + OTA ऑटो-अपडेटर।
  - **मीटरिंग लॉजिक:** स्पेशल सर्च एवं डीप वैल्यूएशन स्कैन्स (`special_searches_count`).

### 8. 🗄️ लोकल आरएजी इंजन (`apps/local_rag_engine`)
- **डिप्लॉयमेंट रूपरेखा:**
  - यूजर इंटरफ़ेस: **PWA / डेस्कटॉप ऐप** (सरल, तेज़, रिस्पॉन्सिव)।
  - बैकएंड इंजन: लोकल रस्ट Axum सर्वर + SQLite AES-256-GCM एन्क्रिप्टेड वॉल्ट।
- **मुख्य क्षमताएं:**
  - यूज़र की अपनी लोकल मशीन पर ही DirectML GPU (DirectX 12) द्वारा BGE वेक्टर्स और इन-मेमरी कोसाइन सर्च (< 2ms लेटेंसी)।
  - 100% एयर-गैप्ड डेटा सुरक्षा; कोई भी प्राइवेट इनवॉइस या कस्टमर डेटा क्लाउड पर नहीं जाता।
- **लाइसेंसिंग एवं मीटरिंग:**
  - 24-घंटे क्रिप्टोग्राफिक लीज + OTA ऑटो-अपडेटर।
  - **मीटरिंग लॉजिक:** कुल सर्च क्वेरीज़ और डॉक्युमेंट चंक्स।

---

## 🔒 यूनिवर्सल सुरक्षा एवं लाइसेंसिंग सारांश (`crates/drm_guard`)

| क्र. सं. | ऍप्लिकेशन नाम | डिप्लॉयमेंट प्रकार | क्लाउड लीज आवृति | 10-Min हेल्थ पिंग | OTA ऑटो-अपडेट | मीटरिंग मेट्रिक्स | स्थिति |
|:---:|---|---|---|:---:|:---:|---|:---:|
| **1** | **Voice AI Calling Agent** (`voice_agent`) | 🌐 Cloud Rust + Next.js PWA | 24 घंटे | — | ✅ | Calls Count & Duration Minutes | ✅ Ready |
| **2** | **CCTV Guard** (`cctv_guard` / `nvr_core`) | 📦 Black-Box Appliance | 24 घंटे | ✅ (WhatsApp) | ✅ | Active Camera Count & Wire Status | ✅ Ready |
| **3** | **Voice Studio** (`dsp_recorder`) | 🖥️ Native Desktop/Mobile | 24 घंटे | — | ✅ | Recording Minutes & Sessions | ✅ Ready |
| **4** | **Video Studio** (`video_studio`) | 🖥️ Native Desktop | 24 घंटे | — | ✅ | Render Minutes & Export Count | ✅ Ready |
| **5** | **Financial App** (`financial_engine`) | 📱 PWA + Rust Backend | 24 घंटे | — | ✅ | Special Searches & Deep Scans | ✅ Ready |
| **6** | **Manifestation App** (`manifestation_app`) | 🖥️ Native Desktop / Axum SPA | 24 घंटे | — | ✅ | Affirmations, Meditations, Visions | ✅ Ready |
| **7** | **Local RAG Engine** (`local_rag_engine`) | 📱 PWA + Local Rust Vault | 24 घंटे | — | ✅ | Search Queries & Indexed Chunks | ✅ Ready |
| **8** | **Content Automation** (`content_automation`) | 🌐 Cloud Rust + Next.js PWA | 24 घंटे | — | ✅ | Generated Posts & Schedules | 🔜 आगामी (Planned) |

---

## 🧪 1. यूनिवर्सल टेस्टिंग प्रोटोकॉल (How to Test & Verify — टेस्ट कैसे करनी है)

डिप्लॉयमेंट से पूर्व प्रत्येक ऐप को 5-स्तरीय परीक्षण से गुजरना अनिवार्य है:

### Level 1: कंपाइलेशन व स्टेटिक एनालिसिस ऑडिट (Static Analysis)
```powershell
# संपूर्ण वर्कस्पेस में 0 एरर और 0 वार्निंग्स सुनिश्चित करें
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

### Level 2: ऑटोमेटेड यूनिट व इंटीग्रेशन टेस्ट सुइट (Automated Test Suite)
```powershell
# संपूर्ण वर्कस्पेस के सभी यूनिट टेस्ट्स रन करें
cargo test --workspace -- --nocapture

# विशिष्ट ऐप्स के टेस्ट्स:
cargo test -p voice_agent
cargo test -p cctv_guard
cargo test -p nvr_core
cargo test -p dsp_recorder
cargo test -p video_studio
cargo test -p financial_engine
cargo test -p manifestation_app
```

### Level 3: एंड-टू-एंड फंक्शनल व सिम्युलेशन टेस्ट (E2E Verification)
- **Voice Agent:** `apps/voice_agent/test_client.html` खोलकर माइक से बोलें और < 300ms में ऑडियो उत्तर व बार्ज-इन जांचें।
- **CCTV Guard:** `cargo run -p cctv_guard -- --simulate "NightIntruderShutter"` चलाकर टियर-1 मोशन डिटेक्शन और टियर-2 Qwen VLM अलर्ट जांचें।
- **NVR Core:** `cargo run -p nvr_core -- --simulate --cameras 4 --port 8080` चलाकर TP-Link वेब पोर्टल (`http://localhost:8080`) पर 4-स्टेप विज़ार्ड देखें।
- **DSP Recorder:** `cargo run -p dsp_recorder -- --list-devices` और `--preset warm` से 24-बिट/48kHz लॉसलेस WAV रिकॉर्डिंग टेस्ट करें।
- **Video Studio:** `cargo run -p video_studio -- --virtual --frames 100 --lut TealAndOrange --gaze` चलाकर 60 FPS पर < 8ms लेटेंसी पुष्टि करें।
- **Financial Engine:** `cargo run -p financial_engine` चलाकर < 50ms में 5000+ स्टॉक्स इन-मेमरी एनालिसिस और कॉन-कॉल Regex माइनिंग जांचें।
- **Manifestation App:** `cargo run -p manifestation_app -- --port 9090` चलाकर `http://localhost:9090` पर सिंगल-स्टेप SD-Turbo विजन बोर्ड जनरेशन, 3-6-9 जर्नल और 528 Hz थिटा मेडिटेशन टेस्ट करें।

### Level 4: 24-घंटे DRM क्रिप्टोग्राफिक लीज व मशीन फिंगरप्रिंट टेस्ट
- जांचें कि मशीन का SHA-256 डिजिटल फिंगरप्रिंट सही ढंग से जनरेट हो रहा है (`DRMGuard::machine_uuid()`)।
- इंटरनेट बंद करके ऐप चलाएं; 24 घंटे का ऑफलाइन ग्रेस पीरियड बिना किसी क्रैश के चालू रहना चाहिए।

### Level 5: लेटेंसी बजट व मेमोरी सीलिंग स्ट्रेस टेस्ट (Memory Leak Check)
- ऐप को 1 घंटे तक निरंतर स्ट्रेस में चलाएं और टास्क मैनेजर या `Get-Process` से पुष्टि करें कि RAM बजट के भीतर ही स्थिर है:
  - `manifestation_app`: < 45 MB RAM
  - `voice_agent`: < 150 MB RAM
  - `cctv_guard` / `nvr_core`: < 900 MB RAM (2 GB Black Box बजट)
  - `financial_engine`: < 150 MB RAM (5000+ स्टॉक्स)

---

## 📋 2. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist — क्या-क्या ओके करना है)

डिप्लॉयमेंट से ठीक पहले इन 7 बिंदुओं पर टिक (OK) लगाना अनिवार्य है:

- [ ] **1. Zero Compiler Warnings:** `cargo check --release --workspace` में शुद्ध **0 त्रुटियां (Zero Errors)** और **0 वार्निंग्स (Zero Warnings)**।
- [ ] **2. Test Suite 100% Passed:** वर्कस्पेस के सभी यूनिट व इंटीग्रेशन टेस्ट्स 100% सफल।
- [ ] **3. Model Files Integrity (`C:\AI_Models\...`):** आवश्यक सभी मॉडल्स अपनी पूर्ण बाइट साइज़ के साथ डिस्क पर मौजूद हैं:
  - `C:\AI_Models\sd_turbo` (कुल 5.16 GB — UNet 3.46 GB, Text Encoder 1.36 GB, VAE 334 MB, Tokenizer, Scheduler)
  - `C:\AI_Models\yolov8m.onnx` (103.7 MB)
  - `C:\AI_Models\silero_vad.onnx` (< 2 MB)
  - `C:\AI_Models\whisper_tiny_onnx` (Encoder, Decoder, Tokenizer)
  - `C:\AI_Models\qwen2-vl-2b` या `qwen2.5_0.5b.onnx`
  - `C:\AI_Models\piper_hi` (Hindi TTS) व `OpenVoice-ONNX-v2`
  - `C:\AI_Models\bge_small_embedding` व `bge_reranker`
- [ ] **4. "Core-Engine First" आर्किटेक्चरल अखंडता:** सभी AI मॉडल्स केंद्रीय रूप से `crates/core` (`ai_core`) में स्थित हैं; किसी भी ऐप में डुप्लीकेट मॉडल कोड नहीं है।
- [ ] **5. 24-घंटे लीज व सुरक्षा सत्यापन:** `crates/drm_guard` की 24h क्रिप्टोग्राफिक लीज सक्रिय है और मशीन बाइंडिंग लॉक है।
- [ ] **6. नेटवर्क पोर्ट्स व फ़ायरवॉल उपलब्धता:** आवश्यक पोर्ट्स (3000, 8080, 9090) खाली हैं और विंडोज/लिनक्स फ़ायरवॉल में स्वीकृत हैं।
- [ ] **7. रिसोर्स बजट व मेमोरी सीलिंग:** किसी भी ऐप में मेमोरी लीक नहीं है और RAM निर्धारित सीलिंग के अंदर है।

---

## 🚀 3. फाइनल डिप्लॉयमेंट प्रक्रिया (Final Deployment Procedure — फाइनल डिप्लॉय कैसे करना है)

### Step 1: प्रोडक्शन बाइनरी संकलन (Release Builds)
```powershell
# संपूर्ण वर्कस्पेस को हाई-ऑप्टिमाइज़ेशन LTO रिलीज़ मोड में संकलित करें
cargo build --release --workspace

# विशिष्ट ऐप बाइनरी बिल्ड:
cargo build --release -p voice_agent
cargo build --release -p cctv_guard
cargo build --release -p nvr_core
cargo build --release -p video_studio
cargo build --release -p dsp_recorder
cargo build --release -p financial_engine
cargo build --release -p manifestation_app
cargo build --release -p local_rag_engine
```
उत्पादित बाइनरी पाथ: `target/release/<app_name>.exe` (Windows) या `target/release/<app_name>` (Linux)।

### Step 2: क्लाउड लाइसेंसिंग वर्कर व Supabase माइग्रेशन
```powershell
# 1. Cloudflare DRM Licensing Worker डिप्लॉय करें
cd crates/drm_guard/scripts
npx wrangler deploy universal_licensing_worker.js --name "universal-drm-guard"

# 2. Supabase pgvector स्कीमा माइग्रेट करें
# apps/voice_agent/supabase_schema.sql को Supabase SQL एडिटर में रन करें
```

### Step 3: 24x7 ऑटो-स्टार्ट प्रोडक्शन सर्विस डिप्लॉयमेंट

#### A. Windows Task Scheduler / NSSM (सिस्टम बूट पर ऑटो-स्टार्ट बिना यूजर लॉगिन):
```powershell
# 1. Manifestation Super App सर्विस
schtasks /create /tn "ShambhaviManifestation" `
  /tr "'D:\ai_core_framework - Copy (2)\target\release\manifestation_app.exe' --port 9090 --sd-models-dir 'C:\AI_Models\sd_turbo'" `
  /sc onstart /ru SYSTEM

# 2. CCTV Guard Sentry सर्विस
schtasks /create /tn "CctvGuardSentry" `
  /tr "'D:\ai_core_framework - Copy (2)\target\release\cctv_guard.exe' --rtsp-url 'rtsp://admin:pass@192.168.1.50:554/h264' --camera-id vault_01" `
  /sc onstart /ru SYSTEM

# 3. Voice Agent NSSM सर्विस
nssm install VoiceAgent "D:\ai_core_framework - Copy (2)\target\release\voice_agent.exe"
nssm set VoiceAgent Start SERVICE_AUTO_START
nssm start VoiceAgent
```

#### B. Linux Headless Server / Black-Box Systemd Service (`/etc/systemd/system/`):
```ini
[Unit]
Description=Shambhavi AI Core Engine Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/shambhavi/app
ExecStart=/opt/shambhavi/app/app_binary
Restart=always
RestartSec=3
LimitNOFILE=65536
MemoryMax=1500M

[Install]
WantedBy=multi-user.target
```
सक्षम व सक्रिय करें:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now app_name.service
sudo systemctl status app_name.service
```

### Step 4: साइलेंट Jio TV स्टाइल OTA ऑटो-अपडेटर सत्यापन
- पुष्टि करें कि बाइनरी बैकग्राउंड में बिना लाइव रिकॉर्डिंग या एक्टिव कॉल रोके नए अपडेट को वैलिडेट करके एटॉमिकली स्वैप (`replace_executable`) कर सकती है।

