# 📋 Product Requirements Document (PRD)
## Pure Rust Financial Super Engine (`apps/financial_engine`)

---

## 1. Product Vision & Mission

### Problem Statement
Traditional financial analysis platforms suffer from three major roadblocks:
1. **Excessive Recurring Costs:** Broker APIs (Zerodha Kite Connect ₹2,000/month, Bloomberg $25,000/year, FactSet, Refinitiv) lock traders into expensive subscriptions.
2. **Heavy, Fragile Python Runtimes:** Typical quantitative pipelines rely on Python, Pandas, Selenium, and LangChain, resulting in slow execution (>3–10 seconds per report), heavy RAM consumption (>2 GB), and brittle driver dependency breakages.
3. **Black-Box Hallucinations:** Generic GenAI tools hallucinate numbers, hallucinate historical PE ratios, and lack deterministic mathematical validation.

### Our Solution
A **Pure Rust Native Binary Application** that:
- Runs **100% Free Public Data Architectures** (NSE/BSE public endpoints, RSS feeds, and headless Chrome DevTools Protocol via native WebSockets).
- Delivers **Institutional-Grade Deterministic Math (90%)** combined with **Local ONNX AI Models (10%)** from `C:\AI_Models`.
- Operates at **sub-50ms latency** across all **5,000+ NSE and BSE listed companies** inside **< 150 MB of RAM**.

---

## 2. Target Persona & User Stories

### Target Personas:
1. **Quantitative & Systematic Traders:** Requires real-time 24x7 volume outlier detection, Bollinger Band squeeze breakouts, and golden cross momentum alerts.
2. **Fundamental Long-Term Investors:** Evaluates ROCE (>20%), clean balance sheets (Debt/Equity < 0.2), earnings quality (CFO/PAT > 1.0), and 5-year TAM expansion runway.
3. **Family Offices & Prop Desks:** Demands private, self-hosted, air-gapped financial intelligence with zero data leaks to third-party cloud brokers.

---

## 3. Detailed Functional Requirements

### FR-1: 5,000+ Stock In-Memory Database
- The system MUST maintain a thread-safe in-memory cache of all ~5,000 NSE/BSE listed equities using `DashMap`.
- Must support concurrent non-blocking reads and batch upserts.
- Lookup latency for any single stock profile MUST be $< 5\text{ microseconds}$.

### FR-2: Deterministic Financial Ratio Engine
- **ROCE:** Compute Return on Capital Employed using exact formula $\frac{\text{EBIT}}{\text{Capital Employed}} \times 100$.
- **Debt-to-Equity:** Compute $\frac{\text{Total Debt}}{\text{Equity} + \text{Reserves}}$.
- **OPM Expansion Delta:** Compare latest quarter OPM% against historical 3–5 year median OPM%.
- **PE Mean Reversion Z-Score:** Statistically measure whether the stock is undervalued or stretched against its 5-year historical median PE.
- **Earnings Quality Test:** Flag accrual risks when $\frac{\text{CFO}}{\text{Net Profit}} < 0.60$.
- **Fundamental Health Score:** Assign a deterministic score on a 1.0 to 10.0 scale.

### FR-3: Technical Indicators & Candlestick Pattern Engine
- **Indicators:** RSI-14, MACD (12, 26, 9), Bollinger Bands (20, 2σ), EMAs (20, 50, 200).
- **Candlestick Pattern Recognizers:** Detect Doji, Hammer, Inverted Hammer, Bullish/Bearish Engulfing, and Morning Star with confidence ratings (0.0 to 1.0).
- **Technical Score:** Objective scoring on a 1.0 to 10.0 scale.

### FR-4: TAM Velocity & Market Share Simulator
- Simulate 3-to-5 year financial trajectories using:
  $$\text{Projected Sales}_t = \text{Industry TAM}_t \times \text{Projected Market Share } \%_t$$
- Produce 3 distinct scenarios: Conservative (Bear), Base (Expected), and Best (Super-Cycle).
- Output projected Revenue, EBITDA, PAT, and EPS with 5-year CAGRs.

### FR-5: Native PDF Corporate Filing Extractor
- Asynchronously extract text from PDF investor presentations and concall transcripts without external C-binaries (using `lopdf`).
- Execute SIMD regex miners for:
  - Capex investments (₹ Cr).
  - Order book values and execution horizons.
  - Capacity expansions (Greenfield / Brownfield).
  - Promoter pledging and encumbrance disclosures.
- Pass extracted disclosures through local `BgeRerankerLocal` (`C:\AI_Models\bge_reranker`) to prioritize critical catalysts.

### FR-6: Autonomous CDP Web Scraper
- Direct headless browser automation via `chromiumoxide` over native WebSockets.
- Zero Selenium / Puppeteer wrappers.
- Extract real-time financial news, RBI circulars, and PIB policy announcements.
- Pass text through local `QwenLocal` (`C:\AI_Models\qwen2-vl-2b`) for policy impact scoring.

### FR-7: 24x7 Outlier Scanner & Master Stock Report
- Background Tokio task continuously monitoring all 5,000+ stocks for:
  - Volume surges ($> 2.5\times$ 20-day average).
  - Bollinger Band squeeze breakouts.
  - New Capex filings.
- Synthesize all findings into a unified `MasterStockReport`:
  - `composite_score` (1.0 to 10.0).
  - `master_verdict`: `StrongAccumulation`, `NeutralWatch`, `AvoidRisk`.
  - Serialized to clean JSON with $< 50\text{ ms}$ latency for PWA / Web dashboards.

---

## 4. Non-Functional Requirements & Performance SLAs

| Metric | Target SLA | Production Reality |
| :--- | :--- | :--- |
| **Language & Runtime** | 100% Pure Rust (0% Python) | Pure Rust 2021 Edition |
| **Report Generation Latency** | $< 100\text{ ms}$ | **$< 50\text{ ms}$** |
| **In-Memory RAM Budget** | $< 250\text{ MB}$ | **$< 75\text{ MB}$ (5,000 stocks)** |
| **Data Ingestion Cost** | ₹0 / month | 100% Free Public Architecture |
| **Crash Rate** | 0% (Zero Panics) | Handled via `anyhow::Result` & graceful fallbacks |
