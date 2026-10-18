# 🚀 Financial Super Engine — Deployment Guide (`apps/financial_engine`)
## Shambhavi AI Labs — Proprietary Wealth Engine & Native Mobile App (Android / iOS)

---

## 📈 1. फ्रंटएंड व क्लाइंट आर्किटेक्चर (Frontend & Client Architecture)

| घटक | विवरण |
|---|---|
| **कोर क्वांट इंजन** | 100% शुद्ध रस्ट इन-मेमरी क्वांट इंजन, 5,000+ NSE/BSE कंपनियों का पूरा डेटाबेस RAM में (< 75 MB RAM), सब-50ms रेश्यो मैथ, 0% पायथन |
| **डेटा पाइपलाइन** | 100% मुफ़्त पब्लिक डेटा पाइपलाइन (CDP + NSE/BSE Bhavcopy + कॉर्पोरेट फाइलिंग्स) |
| **शाम्भवी वेल्थ डैशबोर्ड** | **Shambhavi Proprietary Wealth Dashboard** — क्लीन, मिनिमलिस्ट, डेटा-डेंस, विज्ञापन-मुक्त इंटरफ़ेस: <br>• **क्विक रेश्यो मल्टी-बॉक्स:** P/E, ROCE, ROE, Debt to Equity, Market Cap, Book Value, Dividend Yield<br>• **प्रोज़ एंड कॉन्स कार्ड्स:** ऑटोमेटेड AI स्ट्रेंथ्स और रिस्क चेकलिस्ट<br>• **10-वर्षीय टेबल्स:** क्वार्टरली रिज़ल्ट्स, P&L, बैलेंस शीट, कैश फ्लो हिस्ट्री<br>• **कॉन-कॉल माइनिंग:** मैनेजमेंट कमेंट्री, ग्रोथ गाइडेंस, और अर्निंग्स कॉल हाइलाइट्स<br>• **डिटर्मिनिस्टिक वैल्यूएशन:** DCF, Graham Number, Peter Lynch Fair Value, और Margin of Safety |
| **नेटिव मोबाइल ऐप (Android / iOS)** | **Native Investor Mobile App** — पोर्टफोलियो वॉचलिस्ट, लाइव स्टॉक अलर्ट्स, और 1-क्लिक डीप फंडामेंटल एनालिसिस |
| **लाइसेंसिंग एवं मीटरिंग** | `crates/drm_guard` द्वारा 24-घंटे क्रिप्टोग्राफिक लीज + OTA ऑटो-अपडेटर (`FinancialMetrics`: स्पेशल सर्च एवं डीप वैल्यूएशन स्कैन्स) |

---

## 🧪 2. टेस्टिंग गाइड (How to Test & Verify — आप खुद कैसे टेस्ट करेंगे)

### Level 1: स्टेटिक कंपाइलेशन व ऑडिट
```powershell
cargo check -p financial_engine
cargo clippy -p financial_engine -- -D warnings
```

### Level 2: ऑटोमेटेड रेश्यो मैथ व स्क्रेपर टेस्ट सुइट
```powershell
cargo test -p financial_engine -- --nocapture
```

### Level 3: हैंड्स-ऑन लाइव स्टॉक एनालिसिस टेस्ट (Drive the Financial Engine)
1. **फाइनेंशियल इंजन प्रारंभ करें:**
   ```powershell
   cargo run -p financial_engine
   ```
2. **इन-मेमरी डीप स्कैन व कॉन-कॉल माइनिंग टेस्ट:**
   - टर्मिनल में देखें:
     - 5,000+ स्टॉक्स का इन-मेमरी कैश लोड: **< 45ms**
     - टेस्ट स्टॉक (जैसे `TCS` या `RELIANCE`) का फंडामेंटल स्कोर:
       - **P/E vs Industry P/E:** तत्काल गणना
       - **ROCE & ROE:** 10-वर्षीय ऐतिहासिक CAGR
       - **Debt to Equity:** सॉल्वेंसी स्कोर
       - **Con-Call Sentiment:** मैनेजमेंट कमेंट्री से AI इनसाइट्स
     - स्क्रीन पर Screener.in-स्टाइल सारांश आउटपुट प्रस्तुत होगा।

---

## 📋 3. प्री-डिप्लॉयमेंट चेकलिस्ट (Pre-Deployment Checklist)

- [ ] **1. Zero Compiler Warnings:** `cargo check --release -p financial_engine` में 0 त्रुटियां और 0 चेतावनियां।
- [ ] **2. लेटेंसी सत्यापन:** 5,000+ कंपनियों के सर्च व रेश्यो मैथ का उत्तर समय **< 50ms**।
- [ ] **3. मेमोरी सीलिंग:** 5,000 स्टॉक्स लोड होने के बाद कुल RAM उपयोग **< 150 MB** पर स्थिर।
- [ ] **4. 24-घंटे DRM लीज:** `crates/drm_guard` सक्रिय है और स्पेशल सर्च की मीटरिंग दर्ज हो रही है।
- [ ] **5. डेटा अखंडता:** भावकॉपी व पब्लिक डेटा पार्सिंग में कोई मिसिंग फील्ड एरर नहीं है।

---

## 🚀 4. फाइनल डिप्लॉयमेंट प्रक्रिया (Final Production Deployment)

### Step 1: प्रोडक्शन रिलीज़ बाइनरी संकलन
```powershell
cargo build --release -p financial_engine
```
बाइनरी पाथ: `target/release/financial_engine.exe` (Windows) या `target/release/financial_engine` (Linux)।

### Step 2: शाम्भवी वेल्थ डैशबोर्ड होस्टिंग
```bash
# Axum / Next.js वेब डैशबोर्ड होस्ट करें
cd apps/financial_engine/dashboard
npm install
npm run build
pm2 start npm --name "financial-wealth-dashboard" -- start -- -p 5000
```
ग्राहक ब्राउज़र में `http://localhost:5000` या `https://wealth.yourdomain.com` खोलकर सीधे सर्च कर सकते हैं।

### Step 3: मोबाइल ऐप बंडलिंग (Android & iOS)
रस्ट के हाई-स्पीड क्वांट इंजन को मोबाइल ऐप के लिए एक्सपोर्ट करके Android व iOS ऐप तैयार किए जाते हैं, जिससे इन्वेस्टर के फोन पर बिना किसी सर्वर लेटेंसी के ऑफलाइन भी तुरंत रेश्यो कैलकुलेट होते हैं।
