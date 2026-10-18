# 🚀 Local RAG Engine — Deployment Guide (`apps/local_rag_engine`)
## Shambhavi AI Labs — 100% Air-Gapped Enterprise Document Search Vault (Desktop / Intranet Web UI)

---

## 🗄️ 1. फ्रंटएंड व क्लाइंट आर्किटेक्चर (Frontend & Workstation Architecture)

| घटक | विवरण |
|---|---|
| **कोर RAG इंजन** | 100% शुद्ध रस्ट Local RAG इंजन, DirectML GPU (DirectX 12) द्वारा BGE Small वेक्टर्स, सब-2ms इन-मेमरी कोसाइन सर्च |
| **एंटरप्राइज वॉल्ट सुरक्षा** | **100% एयर-गैप्ड (Air-Gapped):** कोई भी प्राइवेट डेटा (मरीजों के मेडिकल रिकॉर्ड्स, इनवॉइस, लीगल कॉन्ट्रैक्ट्स) इंटरनेट पर नहीं जाता; लोकल SQLite AES-256-GCM एन्क्रिप्टेड स्टोरेज |
| **क्लाइंट इंटरफ़ेस (Desktop / Intranet Web UI)** | **Enterprise Document Search Vault UI** — (अस्पताल, कोर्ट/वकील, अकाउंटेंट, और कॉर्पोरेट ऑफिस वर्कस्टेशन के लिए समर्पित)। <br>*(नोट: यूज़र निर्देशानुसार इसका कोई मोबाइल ऐप नहीं बनेगा, क्योंकि जटिल डॉक्युमेंट सर्च व बिलिंग ऑडिटिंग केवल कंप्यूटर/वर्कस्टेशन पर की जाती है।)* |
| **यूआई क्षमताएं** | बल्क PDF/CSV/Excel ड्रैग-एंड-ड्रॉप इंजेक्शन, सिमेंटिक सर्च बार, डॉक्यूमेंट चंक्स की ओरिजिनल पेज साइटेशन, और CRM क्लाइंट रिकॉर्ड लिंकिंग |
| **लाइसेंसिंग एवं मीटरिंग** | `crates/drm_guard` द्वारा 24-घंटे लीज + OTA ऑटो-अपडेटर (`RagEngineMetrics`: सर्च क्वेरीज़ और इंडेक्स्ड डॉक्युमेंट चंक्स) |

---

## 🧪 2. टेस्टिंग गाइड (How to Test & Verify — आप खुद कैसे टेस्ट करेंगे)

### Level 1: स्टेटिक कंपाइलेशन व ऑडिट
```powershell
cargo check -p local_rag_engine
cargo clippy -p local_rag_engine -- -D warnings
```

### Level 2: ऑटोमेटेड वेक्टर व एन्क्रिप्शन टेस्ट सुइट
```powershell
cargo test -p local_rag_engine -- --nocapture
```

### Level 3: हैंड्स-ऑन लाइव डॉक्यूमेंट सर्च टेस्ट (Drive Local RAG)
1. **लोकल आरएजी वॉल्ट प्रारंभ करें:**
   ```powershell
   cargo run -p local_rag_engine -- --port 8080 --db-path "rag_vault.db"
   ```
2. **ब्राउज़र में वॉल्ट डैशबोर्ड खोलें:**
   - जाएं: `http://localhost:8080`
   - **Upload Document:** अस्पताल का कोई टेस्ट बिल या पॉलिसी PDF ड्रैग-एंड-ड्रॉप करें।
   - स्क्रीन पर देखें:
     - **Text Parser:** सेकंडों में टेक्स्ट एक्सट्रैक्शन व चंकिंग।
     - **DirectML GPU:** BGE Small मॉडल द्वारा 384-डायमेंशनल वेक्टर्स का निर्माण।
     - **AES-256 Vault:** डेटाबेस में एन्क्रिप्टेड सेव।
   - **Search Query:** सर्च बार में टाइप करें: *"Patient discharge billing summary"*
   - **Result:** **< 2ms** में सटीक पैराग्राफ, पेज नंबर और मैचिंग स्कोर स्क्रीन पर हाइलाइट होकर दिखेगा।

---

## 📋 3. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist)

- [ ] **1. Zero Compiler Warnings:** `cargo check --release -p local_rag_engine` 0 त्रुटियां और 0 चेतावनियां।
- [ ] **2. ONNX मॉडल उपलब्धता (`C:\AI_Models\`):**
  - `bge_small_embedding/` (384-dim ONNX मॉडल व टोकेनाइज़र)
- [ ] **3. डेटाबेस वॉल्ट पाथ:** SQLite डेटाबेस पाथ पर पर्याप्त स्टोरेज उपलब्ध है।
- [ ] **4. 24-घंटे लीज व मशीन बाइंडिंग:** `DrmGuard` की 24h क्रिप्टोग्राफिक लीज सक्रिय है और 10-मिनट हार्टबीट बैकग्राउंड लूप चालू है।
- [ ] **5. रिसोर्स बजट:** 10,000+ डॉक्यूमेंट चंक्स के साथ RAM उपयोग **< 180 MB** पर स्थिर।

---

## 🚀 4. फाइनल डिप्लॉयमेंट प्रक्रिया (Workstation / Intranet Server Deployment)

### Step 1: प्रोडक्शन रिलीज़ बाइनरी संकलन
```powershell
cargo build --release -p local_rag_engine
```
बाइनरी पाथ: `target/release/local_rag_engine.exe` (Windows) या `target/release/local_rag_engine` (Linux)।

### Step 2: ऑफिस इंट्रानेट सर्वर ऑटो-स्टार्ट सर्विस (Windows Task Scheduler / Linux Systemd)
अस्पताल या ऑफिस के लोकल सर्वर पर इसे ऑटो-स्टार्ट सर्विस के रूप में डिप्लॉय करें ताकि पूरे लोकल नेटवर्क (LAN) के डॉक्टर्स या क्लर्क अपने कंप्यूटर से `http://192.168.1.50:8080` पर सर्च कर सकें:
```powershell
schtasks /create /tn "ShambhaviRagVault" `
  /tr "'D:\ai_core_framework - Copy (2)\target\release\local_rag_engine.exe' --host 0.0.0.0 --port 8080 --db-path 'D:\vault\office_vault.db'" `
  /sc onstart /ru SYSTEM
```
