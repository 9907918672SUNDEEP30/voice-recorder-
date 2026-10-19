# AI Core Framework और सभी Apps का Code Audit

**Audit की तारीख:** 19 अक्टूबर 2026  
**भाषा:** हिंदी  
**Audit का प्रकार:** Static source-code review और Rust workspace test run  
**बदलाव:** Audit के दौरान source code या manifests में कोई बदलाव नहीं किया गया।

> यह रिपोर्ट उपलब्ध source code और tests पर आधारित है। यह live production, वास्तविक cameras/carriers, deployment network, GPU/model output या Cloudflare/Supabase configuration का operational security test नहीं है। Findings को code में देखे गए व्यवहार के अनुसार लिखा गया है; environment के अनुसार exploitability बदल सकती है।

## 1. संक्षिप्त नतीजा

- सभी **6 core crates** और सभी **8 applications** की समीक्षा की गई।
- Workspace test command `cargo test --workspace --no-fail-fast` पूरा चला, लेकिन सफल नहीं हुआ।
- कुल **101 tests पास**, **4 tests fail** हुए; Cargo ने exit code `101` लौटाया।
- नीचे **29 actionable findings** सूचीबद्ध हैं: **11 उच्च** और **18 मध्यम**। कोई नया critical finding नहीं मिला।
- पिछले audit के कुछ सुधार code में मौजूद हैं, लेकिन कई संबंधित क्षेत्रों में अलग या शेष समस्याएँ मिलीं।

## 2. Test suite का परिणाम

चलाया गया command:

```text
cargo test --workspace --no-fail-fast
```

| परिणाम | संख्या |
|---|---:|
| पास | 101 |
| फेल | 4 |
| अनदेखे | 0 |
| Command exit code | 101 |

### असफल tests

1. **CCTV Guard — `test_end_to_end_bank_shutter_robbery_simulation`**  
   स्थान: [`apps/cctv_guard/tests/guard_tests.rs`](apps/cctv_guard/tests/guard_tests.rs#L249)  
   विफलता: `assertion failed: rec.description.contains("roller shutter")`

2. **Local RAG — `test_drm_guard_autonomous_lease_and_zeroization`**  
   स्थान: [`apps/local_rag_engine/tests/integration_tests.rs`](apps/local_rag_engine/tests/integration_tests.rs#L32)  
   विफलता: `assertion failed: res.is_ok()`

3. **Local RAG — `test_local_rag_engine_ingest_and_query`**  
   स्थान: [`apps/local_rag_engine/tests/integration_tests.rs`](apps/local_rag_engine/tests/integration_tests.rs#L85)  
   विफलता: ingestion के समय वैध server-signed Ed25519 lease न मिलने पर DRM ने fail-closed error लौटाया।

Local RAG failures में test setup और fail-closed licensing behavior के बीच अंतर दिखता है। समाधान के लिए यह तय करना होगा कि tests में वैध signed fixture/worker देना है या runtime configuration बदलनी है; fail-closed सुरक्षा को कमजोर करके test पास कराना उचित नहीं होगा।

## 3. गंभीरता का अर्थ

- **उच्च:** अनधिकृत पहुँच, सुरक्षा नियंत्रण bypass, गलत/नकली production output, या महत्वपूर्ण सेवा failure का ठोस जोखिम।
- **मध्यम:** डेटा/नतीजों की विश्वसनीयता, उपलब्धता, गोपनीयता या workflow पर उल्लेखनीय असर; अक्सर कुछ deployment/configuration शर्तों पर निर्भर।

---

## 4. उच्च गंभीरता की findings

### H-01 — OTA binary swap में signature verification bypass

**फाइल:** [`crates/drm_guard/src/updater.rs`](crates/drm_guard/src/updater.rs#L215)  
**Confidence:** 10/10

`download_and_verify` signature की जाँच करता है, लेकिन public `apply_binary_swap` को सीधे दिए गए bytes पर वही जाँच नहीं दिखती। कोई caller verified download path छो़ड़कर सीधे file replacement तक पहुँच सकता है।

**सुझाव:** swap method को private/verified-artifact-only बनाएँ या install boundary पर signature और digest फिर से सत्यापित करें।

### H-02 — Server denial के बाद भी cached DRM lease स्वीकार हो सकती है

**फाइल:** [`crates/drm_guard/src/guard.rs`](crates/drm_guard/src/guard.rs#L398)  
**Confidence:** 9/10

HTTP `401`/`403` या अन्य explicit denial के बाद code offline cache जाँचता है। अभी-अभी revoke/deny हुई lease यदि locally unexpired है, तो app उसे स्वीकार कर सकती है।

**सुझाव:** authorization denial/revocation को केवल connectivity failure से अलग रखें; denial पर cached lease को भी अस्वीकार/invalidate करें।

### H-03 — Voice WebSocket authentication secret न होने पर fail-open

**फाइल:** [`apps/voice_agent/src/telephony.rs`](apps/voice_agent/src/telephony.rs#L101)  
**Confidence:** 10/10

यदि `VOICE_GATEWAY_SECRET` और `TWILIO_AUTH_TOKEN` दोनों अनुपस्थित हों, तो connection बिना credential जाँच के upgrade हो सकता है। Exposed gateway पर कोई भी reachable client model/TTS संसाधन इस्तेमाल कर सकता है।

**सुझाव:** आवश्यक secret न होने पर startup रोकें; authentication को वैकल्पिक न रखें।

### H-04 — NVR का सक्रिय recording loop वास्तविक playable MP4 नहीं बनाता

**फाइलें:** [`apps/nvr_core/src/main.rs`](apps/nvr_core/src/main.rs#L160), [`apps/nvr_core/src/storage/mp4_writer.rs`](apps/nvr_core/src/storage/mp4_writer.rs#L44)  
**Confidence:** 10/10

सामान्य loop `SimulatedCameraFeed` और fixed zero-byte payload उपयोग करता है। Segment writer bytes को `.mp4` नाम की file में सीधे लिखता है; MP4 muxing नहीं करता। FFmpeg recorder का implementation मौजूद है, लेकिन सामान्य run path में call नहीं होता।

**सुझाव:** production recording को वास्तविक RTSP/FFmpeg path से जोड़ें; generated output को वास्तविक media decoder से खोलकर test करें।

### H-05 — CCTV threat labels heuristic feature statistics से बनते हैं

**फाइल:** [`apps/cctv_guard/src/tier2/qwen_vlm.rs`](apps/cctv_guard/src/tier2/qwen_vlm.rs#L188)  
**Confidence:** 10/10

ONNX output को token energy/variance जैसे आँकड़ों में घटाकर fixed thresholds से “intruder” आदि labels और hard-coded confidence values दिए जाते हैं। इससे semantic threat classification प्रमाणित नहीं होती, लेकिन वही labels alerts चला सकते हैं।

**सुझाव:** trained/validated classifier output दें या इस signal को स्पष्ट रूप से uncalibrated anomaly score कहें; इसे threat probability/confidence की तरह न दिखाएँ।

### H-06 — Continuous NVR run में quota cleanup नहीं चलता

**फाइल:** [`apps/nvr_core/src/main.rs`](apps/nvr_core/src/main.rs#L190)  
**Confidence:** 10/10

जब `target_frames == 0` हो, recording loop अनंत चलता है और quota cleanup loop के बाद ही है। इसलिए continuous mode में quota पार होने पर cleanup trigger नहीं होगा। Cleaner की fixed 50-GB assumption भी actual storage से अलग हो सकती है।

**सुझाव:** recording के दौरान periodic quota check/cleanup करें और वास्तविक volume/quota पढ़ें।

### H-07 — NVR license activation route admin authentication से बाहर है

**फाइलें:** [`apps/nvr_core/src/web_portal/routes.rs`](apps/nvr_core/src/web_portal/routes.rs#L88), [`apps/nvr_core/src/web_portal/mod.rs`](apps/nvr_core/src/web_portal/mod.rs#L38)  
**Confidence:** 9/10

Activation handler DRM credentials बदलता है, पर route public router में है। Portal all-interface bind करता है; reachable client activation/credentials में हस्तक्षेप कर सकता है।

**सुझाव:** route को authenticate करें या one-time local-only setup flow तक सीमित करें।

### H-08 — Manifestation App में persisted input से stored XSS

**फाइल:** [`apps/manifestation_app/src/main.rs`](apps/manifestation_app/src/main.rs#L616)  
**Confidence:** 10/10

App network interfaces पर उपलब्ध है; user-provided vision/journal text store होता है और कुछ dashboard fields `innerHTML` में बिना escaping डाले जाते हैं। Crafted text dashboard देखने वाले browser में script चला सकता है।

**सुझाव:** untrusted values `textContent` से render करें या vetted sanitizer लगाएँ; साथ ही API access नियंत्रित करें।

### H-09 — Local RAG external bind पर API key के बिना खुल सकता है

**फाइलें:** [`apps/local_rag_engine/src/server/routes.rs`](apps/local_rag_engine/src/server/routes.rs#L23), [`apps/local_rag_engine/src/main.rs`](apps/local_rag_engine/src/main.rs#L30)  
**Confidence:** 10/10

`RAG_API_KEY` optional है; external bind/`--allow-external` के साथ key अनिवार्य नहीं दिखती। ऐसी configuration में reachable clients document ingest/query और CRM operations कर सकते हैं।

**सुझाव:** external bind के लिए authentication अनिवार्य करें; key न हो तो loopback-only binding रखें।

### H-10 — Local RAG online DRM grant signature सत्यापित नहीं दिखती

**फाइल:** [`apps/local_rag_engine/src/drm/heartbeat.rs`](apps/local_rag_engine/src/drm/heartbeat.rs#L182)  
**Confidence:** 9/10

Heartbeat response में signature field है, लेकिन online path `granted` status पर expiry/session key लागू करने से पहले उसका trusted public key से सत्यापन नहीं करता।

**सुझाव:** canonical lease payload और signature को trusted key से verify किए बिना grant स्वीकार न करें।

### H-11 — Local RAG में clean deployment के लिए activation path अनुपस्थित हो सकता है

**फाइलें:** [`apps/local_rag_engine/src/drm/heartbeat.rs`](apps/local_rag_engine/src/drm/heartbeat.rs#L205), [`apps/local_rag_engine/src/main.rs`](apps/local_rag_engine/src/main.rs#L55)  
**Confidence:** 10/10

Worker URL optional है, लेकिन worker और पहले से provisioned valid signed cache दोनों न हों तो heartbeat fail-closed होता है। Fresh/default deployment सक्रिय नहीं हो सकता; संबंधित integration test भी इसी कारण fail हुआ।

**सुझाव:** सुरक्षित, documented initial provisioning/activation flow बनाएँ और tests में वास्तविक signed fixture या नियंत्रित test worker दें।

---

## 5. मध्यम गंभीरता की findings

### M-01 — Offline DRM cache का signature सामान्य production lease से मेल नहीं खा सकता

**फाइल:** [`crates/drm_guard/src/guard.rs`](crates/drm_guard/src/guard.rs#L366)  
**Confidence:** 10/10

Production cache session key को disk पर न रखने के लिए `None` लिखता है, जबकि signature reconstruction में session key payload का हिस्सा है। परिणामस्वरूप सामान्य keyed lease offline verify न हो सके; मौजूदा test manually key वाली cache बनाकर production path से अलग व्यवहार जाँचता है।

**सुझाव:** session key persist किए बिना verify होने वाला अलग signed lease claim बनाएं और production cache path का integration test लिखें।

### M-02 — Vault blind index unkeyed hash है और chunk IDs plaintext हैं

**फाइल:** [`crates/vector_store/src/vault.rs`](crates/vector_store/src/vault.rs#L20)  
**Confidence:** 9/10

Document ID का unkeyed SHA-256 hash dictionary guessing से पहचाना जा सकता है; chunk IDs database में plaintext हैं। Database reader अनुमानित IDs और chunk relationships जान सकता है।

**सुझाव:** keyed HMAC blind index और non-predictable identifiers उपयोग करें।

### M-03 — IPC input line की अधिकतम सीमा नहीं

**फाइल:** [`crates/core/src/ipc/bridge.rs`](crates/core/src/ipc/bridge.rs#L23)  
**Confidence:** 9/10

`read_line` newline तक input buffer बढ़ाता है। IPC peer बहुत बड़ा frame भेजकर process memory समाप्त कर सकता है।

**सुझाव:** frame-size cap लागू करें और अधिकतम सीमा से बड़ा input अस्वीकार करें।

### M-04 — Image generation weights मौजूद होने पर भी procedural image लौटाता है

**फाइल:** [`crates/core/src/models/image_gen.rs`](crates/core/src/models/image_gen.rs#L106)  
**Confidence:** 10/10

Weights की मौजूदगी जाँचने के बाद भी execution procedural canvas renderer पर जाता है; उपलब्ध weights से वास्तविक inference नहीं होता। Caller को prompt-संबंधित model image के बजाय unrelated procedural output मिलता है।

**सुझाव:** वास्तविक inference लागू करें, अन्यथा स्पष्ट typed error लौटाएँ।

### M-05 — Core Engine model failure पर semantic-जैसे fallback देता है

**फाइल:** [`crates/core_engine/src/lib.rs`](crates/core_engine/src/lib.rs#L55)  
**Confidence:** 9/10

Embedding model न मिलने/विफल होने पर hashed bag-of-words vector और LLM failure पर पहला context लौटाया जा सकता है। Caller fallback को model result समझ सकता है।

**सुझाव:** fallback को opt-in एवं स्पष्ट बनाएं, या typed error लौटाएँ।

### M-06 — HNSW duplicate ID update पर graph links पुराने vector के रहते हैं

**फाइल:** [`crates/vector_store/src/hnsw.rs`](crates/vector_store/src/hnsw.rs#L134)  
**Confidence:** 9/10

Existing node का vector बदला जाता है, लेकिन पड़ोसी links पुराने vector पर आधारित रहते हैं। Updated item का approximate search recall बिगड़ सकता है।

**सुझाव:** update पर node को reconnect/reinsert करें या graph rebuild करें; दूरस्थ vector update test जोड़ें।

### M-07 — Index memory zeroization अधूरी है

**फाइलें:** [`crates/vector_store/src/hnsw.rs`](crates/vector_store/src/hnsw.rs#L341), [`crates/vector_store/src/index.rs`](crates/vector_store/src/index.rs#L99)  
**Confidence:** 9/10

Explicit `clear()` के कुछ vector scrubbing के बावजूद HNSW map keys और सामान्य object destruction/drop पर सभी buffers की zeroization सुनिश्चित नहीं दिखती।

**सुझाव:** vector तथा identifiers के लिए zeroize-aware storage/Drop लागू करें और cleanup lifecycle test करें।

### M-08 — PDF partial extraction failures सामान्य API से छिपती हैं

**फाइलें:** [`crates/text_parser/src/pdf.rs`](crates/text_parser/src/pdf.rs#L86), [`crates/text_parser/src/lib.rs`](crates/text_parser/src/lib.rs#L49)  
**Confidence:** 9/10

Detailed parser failed pages एकत्र कर सकता है, पर सामान्य public API caller को केवल chunks देती है। अधूरा PDF पूरा मानकर ingest हो सकता है।

**सुझाव:** public result में failed page numbers/status दें या partial extraction को error बनाएँ।

### M-09 — NVR admin UI अपनी admin API calls authenticate नहीं कर पाती

**फाइलें:** [`apps/nvr_core/src/web_portal/routes.rs`](apps/nvr_core/src/web_portal/routes.rs#L36), [`apps/nvr_core/src/web_portal/html.rs`](apps/nvr_core/src/web_portal/html.rs#L777)  
**Confidence:** 10/10

Token generate होता है, पर UI उसे bootstrap या admin `fetch` requests में भेजता नहीं दिखता; protected requests 401 पा सकती हैं। Token generator भी CSPRNG के बजाय PID/time पर निर्भर बताया गया है।

**सुझाव:** OS CSPRNG से token बनाकर सुरक्षित session/bootstrap flow दें और UI requests में auth शामिल करें।

### M-10 — Supabase URL HTTPS तक सीमित नहीं

**फाइल:** [`apps/voice_agent/src/supabase.rs`](apps/voice_agent/src/supabase.rs#L46)  
**Confidence:** 9/10

URL validation में `http://` रोकने की शर्त नहीं दिखती। ऐसे endpoint पर API key और call records/transcripts plaintext में जा सकते हैं।

**सुझाव:** HTTPS अनिवार्य करें और गलत scheme पर configuration error दें।

### M-11 — DSP में VAD model अनुपस्थित होने पर log वास्तविक व्यवहार से उलटा है

**फाइल:** [`apps/dsp_recorder/src/pipeline/audio_pipeline.rs`](apps/dsp_recorder/src/pipeline/audio_pipeline.rs#L97)  
**Confidence:** 10/10

Log “un-gated mode” कहता है, जबकि `VadGated` में default speech state false होने से chunks लिखे नहीं जा सकते। Operator को empty recording का कारण गलत समझ आएगा।

**सुझाव:** log/UI में gated recording बंद होने की स्थिति स्पष्ट करें।

### M-12 — Negative P/E undervaluation scoring को गलत bonus दे सकता है

**फाइल:** [`apps/financial_engine/src/fundamentals.rs`](apps/financial_engine/src/fundamentals.rs#L200)  
**Confidence:** 9/10

Loss-making stock का negative P/E बहुत कम z-score देकर undervaluation bonus पा सकता है, जबकि P/E ऐसी स्थिति में उपयोगी multiple नहीं है।

**सुझाव:** EPS/net income non-positive हो तो P/E reversion score लागू न करें।

### M-13 — Missing financial statements से positive EPS अनुमानित हो सकता है

**फाइल:** [`apps/financial_engine/src/engine.rs`](apps/financial_engine/src/engine.rs#L249)  
**Confidence:** 9/10

Income statement न होने पर P/E से EPS derive करके positive floor लगाया जाता है। Missing/negative input positive earnings के रूप में valuation में जा सकता है।

**सुझाव:** अपर्याप्त data पर valuation रोकें या output को unavailable/insufficient data चिह्नित करें।

### M-14 — Conviction scorer के jitter/shimmer में cycle selection अस्थिर हो सकती है

**फाइल:** [`apps/manifestation_app/src/conviction.rs`](apps/manifestation_app/src/conviction.rs#L123)  
**Confidence:** 8/10

Absolute amplitude peaks को pitch periods मानने से asymmetric waveform में peak polarity बदल सकती है। इससे period/amplitude jitter वास्तव में मौजूद न होने पर भी मापा जा सकता है; वही diagnostics score में उपयोग होते हैं।

**सुझाव:** phase-consistent cycle landmarks अपनाएँ और asymmetric/perturbed voiced signal tests जोड़ें।

### M-15 — Video auto-framing crop aspect ratio बिगाड़ सकता है

**फाइल:** [`apps/video_studio/src/tracking/auto_framing.rs`](apps/video_studio/src/tracking/auto_framing.rs#L105)  
**Confidence:** 9/10

Crop normalized coordinates में बनता है, source aspect ratio ध्यान में नहीं आता और X/Y scaling अलग होती है। Output में दृश्य/चेहरा stretch हो सकता है।

**सुझाव:** source और target pixel aspect ratio ध्यान में रखकर crop निकालें; shape preservation test करें।

### M-16 — Gaze warp default pitch अपनी clamp सीमा के बाहर है

**फाइल:** [`apps/video_studio/src/gaze/gaze_warp.rs`](apps/video_studio/src/gaze/gaze_warp.rs#L25)  
**Confidence:** 9/10

Default pitch `12°` है, लेकिन implementation की upper clamp `+5°` है। इसलिए default व्यवहार असल में `+5°` होगा।

**सुझाव:** intended default/range मिलाएँ और effective default warp test करें।

### M-17 — Local renderer render failure के बाद भी success लौटाता है

**फाइल:** [`apps/content_engine/src/assembly/local_render.rs`](apps/content_engine/src/assembly/local_render.rs#L24)  
**Confidence:** 9/10

FFmpeg missing, execution error या nonzero exit होने पर log के बाद भी `Ok(())` मिल सकता है। Orchestrator video बने बिना success report कर सकता है।

**सुझाव:** वास्तविक error लौटाएँ या explicit `Failed/Skipped` outcome दें।

### M-18 — RAG dashboard API key सेट होने पर अपनी protected calls में key नहीं भेजता

**फाइल:** [`apps/local_rag_engine/src/server/routes.rs`](apps/local_rag_engine/src/server/routes.rs#L645)  
**Confidence:** 9/10

Query/upload/CRM dashboard requests में configured key शामिल नहीं है; key-enabled middleware उन्हें reject कर सकता है।

**सुझाव:** सुरक्षित authenticated dashboard session दें या UI और API auth flow स्पष्ट रूप से अलग करें।

---

## 6. पुराने audit points की पुनः-जाँच

### सुधरे हुए/अब खुले finding के रूप में न रखे गए

- Voice `streamSid` traversal के लिए allowlist/length sanitization दिखाई दी।
- CCTV RTSP में FFmpeg से वास्तविक frame decode path मौजूद है; यह live deployment में सफल है, इसका operational प्रमाण इस audit में नहीं मिला।
- Local RAG dashboard के जाँचे paths में escaping/`innerText` का उपयोग दिखा; वहीं Manifestation App के अलग dashboard path में stored XSS मिला।
- VAD लंबे audio input को 512-sample chunks में बाँटने का code मौजूद है।
- IPC में JSON framing, quoted tokenizer और outbound CR/LF handling मौजूद हैं; input size cap फिर भी नहीं है।
- Local RAG pitch analysis में zero sample rate के लिए lower-bound validation मौजूद है। **इसलिए zero-rate infinite-loop को current finding नहीं माना गया।**
- Financial valuation में सीधे nonpositive EPS के लिए loss-making branch मौजूद है; लेकिन negative P/E scoring और missing-statement EPS fallback की अलग समस्याएँ बनी हैं।
- DSP VAD/resampling/inference errors gated mode में false speech state रखते हैं; पर missing-model log misleading है।

### अभी भी unresolved/अपूर्ण

- OTA download verification मौजूद है, लेकिन public swap method verification bypass कर सकता है।
- DRM app binding और key को disk पर न रखने के बदलाव दिखे, लेकिन offline cache signature mismatch और explicit-denial fallback की समस्याएँ हैं।
- Vault payload metadata encryption हुई, लेकिन unkeyed hash और plaintext chunk IDs की सूचना leakage बाकी है।
- PDF detailed API failed pages जानती है, लेकिन सामान्य public API वह जानकारी caller से छिपाती है।
- Image generator में वास्तविक weights उपलब्ध होने पर भी procedural output की समस्या बनी हुई है।
- HNSW में duplicate ID update topology stale कर सकता है; clear/drop zeroization भी पूर्ण नहीं दिखती।
- General NVR admin routes में auth middleware के कुछ बदलाव दिखे, लेकिन license activation route अलग से public रहना और UI token flow टूटना जारी findings हैं।
- Local RAG API key validation जोड़ी गई लगती है, लेकिन key optional है; external binding के साथ उसे अनिवार्य बनाना जरूरी है।

## 7. Audit coverage और सीमाएँ

- **Core crates:** `core`/`ai_core`, `core_engine`, `drm_guard`, `runtime_cli`, `text_parser`, `vector_store`।
- **Applications:** `voice_agent`, `cctv_guard`, `nvr_core`, `dsp_recorder`, `video_studio`, `financial_engine`, `local_rag_engine`, `manifestation_app`, `content_engine` directory trees। CCTV और NVR अलग directories हैं; इस तरह नौ app folders, पर ledger के अनुसार संयुक्त CCTV Guard & NVR समेत आठ product apps।
- Source और उपलब्ध tests का static निरीक्षण किया गया। Hardware, live camera/RTSP, carrier/SIP, deployment firewall, GPU model quality, OTA server और external services का live test नहीं किया गया।
- Test suite के 4 failures दर्ज हैं; कोई code परिवर्तन इस audit के दौरान नहीं किया गया।

