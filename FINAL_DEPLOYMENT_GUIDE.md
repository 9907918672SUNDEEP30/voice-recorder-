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

### 6. 🧘 मैनिफेस्टेशन ऐप (`apps/manifestation_app`) — [🔜 आगामी / Planned for Next Phase]
> **नोट:** यह ऐप वर्तमान में वर्कस्पेस में निर्मित नहीं है। इसे आगामी चरण में शुद्ध नेटिव रस्ट ऐप के रूप में बनाया जाएगा।
- **डिप्लॉयमेंट रूपरेखा:**
  - डेस्कटॉप और मोबाइल के लिए **Pure Native App** (भविष्य में बनने वाला)।
- **मुख्य क्षमताएं (नियोजित):**
  - 100% लोकल स्टोरेज और ज़ीरो-लेटेंसी माइंडसेट रूटीन।
  - न्यूरल अफर्मेशन शेड्यूलर, सब्लिमिनल ऑडियो जनरेटर और स्ट्रिक ट्रैकर।
- **लाइसेंसिंग:** 24-घंटे की क्रिप्टोग्राफिक लीज + OTA ऑटो-अपडेटर।

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
| **6** | **Manifestation App** (`manifestation_app`) | 🖥️ Native Desktop/Mobile | 24 घंटे | — | ✅ | Active Routine Sessions | 🔜 आगामी (Planned) |
| **7** | **Local RAG Engine** (`local_rag_engine`) | 📱 PWA + Local Rust Vault | 24 घंटे | — | ✅ | Search Queries & Indexed Chunks | ✅ Ready |
| **8** | **Content Automation** (`content_automation`) | 🌐 Cloud Rust + Next.js PWA | 24 घंटे | — | ✅ | Generated Posts & Schedules | 🔜 आगामी (Planned) |


---


## 🛠️ डिप्लॉयमेंट कमांड्स चीट-शीट (Quick Reference)

### 1. प्रोडक्शन बाइनरी संकलन (Release Builds)
```powershell
# संपूर्ण वर्कस्पेस को ऑप्टिमाइज़्ड रिलीज़ मोड में बिल्ड करें
cargo build --release --workspace

# विशिष्ट ऐप बाइनरी बिल्ड करें
cargo build --release -p voice_agent
cargo build --release -p cctv_guard
cargo build --release -p video_studio
cargo build --release -p dsp_recorder
cargo build --release -p financial_engine
cargo build --release -p local_rag_engine
```

### 2. क्लाउडफ्लेयर लाइसेंसिंग वर्कर डिप्लॉयमेंट
```powershell
cd crates/drm_guard/scripts
npx wrangler deploy universal_licensing_worker.js --name "universal-drm-guard"
```

### 3. सुपबेस pgvector स्कीमा माइग्रेशन
```powershell
# Supabase CLI या Supabase वेब SQL एडिटर में निष्पादित करें:
# apps/voice_agent/supabase_schema.sql
```
