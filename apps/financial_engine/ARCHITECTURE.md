# 🏛️ System Architecture: Pure Rust Financial Super Engine
## High-Performance, Zero-Python Autonomous Equity Analytics Engine

---

## 1. Architectural Philosophy & Guarantees

The **Financial Super Engine** (`apps/financial_engine`) is architected with three strict engineering axioms:

```mermaid
graph TD
    subgraph Layer 1: Ingestion [100% Free Public Data Ingestion]
        NSE_API[NSE Public JSON & Bhavcopy]
        BSE_XBRL[BSE Corporate Filings & Announcements]
        PIB_RBI[Gov Circulars & PIB / RBI Feeds]
        CDP_CRX[Chromiumoxide CDP Headless Browser]
    end

    subgraph Layer 2: In-Memory Storage [5,000+ Stock Lock-Free Database]
        DASH[DashMap 6,000 Pre-allocated Buckets]
        PROFILES[StockProfile: Statements, Balance Sheets, Cash Flows]
        CANDLES[Rolling OHLCV Price & Volume Buffers]
        DASH --> PROFILES
        DASH --> CANDLES
    end

    subgraph Layer 3: Deterministic Core [90% Priority: High-Precision Math]
        M1[FinancialRatioEngine: ROCE, D/E, OPM Delta, PE Z-Score, CFO/PAT]
        M2[TechnicalEngine: RSI-14, MACD, Bollinger Bands, EMAs, Patterns]
        M3[TamSimulationEngine: 3-5 Year Multi-Scenario Earnings Trajectories]
        M4[PdfFilingParser: lopdf Native Extraction & Regex Miners]
    end

    subgraph Layer 4: Local AI Inference [10% Priority: C:\\AI_Models Weights]
        BGE[BgeRerankerLocal: C:\\AI_Models\\bge_reranker]
        QWEN[QwenLocal: C:\\AI_Models\\qwen2-vl-2b]
    end

    subgraph Layer 5: Orchestration & Serving [Autonomous 24x7 Engine]
        SCANNER[StockScanner: Background Tokio Worker Loop]
        ENGINE[FinancialSuperEngine: MasterStockReport Synthesizer]
        DASHBOARD[PWA / Web Dashboard JSON <50ms Latency]
    end

    Layer 1 --> Layer 2
    Layer 2 --> Layer 3
    M4 -.-> BGE
    CDP_CRX -.-> QWEN
    Layer 3 --> ENGINE
    BGE --> ENGINE
    QWEN --> ENGINE
    SCANNER --> Layer 2
    ENGINE --> DASHBOARD
```

---

## 2. In-Memory Database Design (`StockDatabase`)

### Lock-Free Concurrent Sharding
Rather than using a single global `RwLock<HashMap<...>>` which suffers from lock contention during concurrent 24x7 scans, the database uses **`DashMap<String, StockProfile>`**:
- Internally partitioned into **64 independent shard locks**.
- Allows concurrent reads and writes across all 5,000+ stocks with zero thread starvation.
- Pre-allocated capacity for 6,000 tickers to prevent re-hashing overhead.

### Memory Footprint Breakdown:
| Data Component | Per Stock Memory | 5,000 Stocks Total |
| :--- | :--- | :--- |
| Core Profile & Metadata | ~256 bytes | ~1.28 MB |
| 5-Year Income Statements (20 quarters) | ~1.6 KB | ~8.00 MB |
| 5-Year Balance Sheets (5 annual) | ~640 bytes | ~3.20 MB |
| 5-Year Cash Flow Statements | ~320 bytes | ~1.60 MB |
| 250-Day Rolling OHLCV Candles | ~12.0 KB | ~60.00 MB |
| **Total In-Memory Database** | **~14.8 KB** | **< 75 MB RAM** |

*Result:* The entire Indian stock market universe (NSE + BSE 5,000+ equities) fits entirely into **less than 75 megabytes of RAM**, delivering sub-microsecond random access times.

---

## 3. Mathematical Models & Deterministic Engines

### 3.1 Financial Ratio Engine (`fundamentals.rs`)
1. **Return on Capital Employed (ROCE):**
   $$\text{EBIT} = \text{EBITDA} - \text{Depreciation}$$
   $$\text{Capital Employed} = \text{Shareholders Equity} + \text{Total Debt}$$
   $$\text{ROCE } \% = \frac{\text{EBIT}}{\text{Capital Employed}} \times 100$$
2. **OPM Expansion Delta:**
   $$\Delta \text{OPM} = \text{Current OPM } \% - \text{Median}(\text{Historical OPMs})$$
3. **PE Mean Reversion Z-Score:**
   $$Z_{\text{PE}} = \frac{\text{Current PE} - \text{Median}(\text{PE}_{5Y})}{\sigma(\text{PE}_{5Y})}$$
4. **CFO to Net Profit (Quality of Earnings):**
   $$\text{Accrual Ratio} = \frac{\text{Cash From Operations (CFO)}}{\text{Net Profit (PAT)}}$$
   - $> 1.0$: Pristine cash conversion.
   - $< 0.5$: Severe earnings quality red flag (inflated uncollected revenue).

---

### 3.2 Technical & Candlestick Pattern Engine (`technicals.rs`)
1. **Wilder's Smoothed RSI (14-period):**
   $$\text{RS} = \frac{\text{Smoothed Avg Gain}}{\text{Smoothed Avg Loss}}, \quad \text{RSI} = 100 - \frac{100}{1 + \text{RS}}$$
2. **MACD (12, 26, 9):**
   $$\text{MACD Line} = \text{EMA}_{12}(\text{Close}) - \text{EMA}_{26}(\text{Close})$$
   $$\text{Signal Line} = \text{EMA}_9(\text{MACD Line}), \quad \text{Histogram} = \text{MACD} - \text{Signal}$$
3. **Bollinger Bands (20, 2σ):**
   $$\text{Bandwidth } \% = \frac{\text{Upper Band} - \text{Lower Band}}{\text{SMA}_{20}} \times 100$$
   - $\text{Bandwidth } \% < 5.0\%$: Volatility Squeeze (Imminent explosive breakout).
4. **Deterministic Candlestick Recognizers:**
   - **Doji:** $\frac{\text{Body}}{\text{High} - \text{Low}} \le 0.10$
   - **Hammer:** $\text{Lower Wick} \ge 2 \times \text{Body} \land \text{Upper Wick} \le 0.2 \times \text{Body}$
   - **Bullish Engulfing:** $\text{Close}_t > \text{Open}_t \land \text{Close}_{t-1} < \text{Open}_{t-1} \land \text{Body}_t > \text{Body}_{t-1}$
   - **Morning Star:** 3-bar reversal confirming bottom liquidity exhaustion.

---

### 3.3 Total Addressable Market (TAM) Simulation Engine (`tam_engine.rs`)
Deterministic multi-scenario simulation model:
$$\text{Projected Sales}_t = \text{Industry TAM}_t \times \left( \frac{\text{Projected Market Share } \%_t}{100} \right)$$
Trajectories generated:
- **Conservative (Bear):** Slower TAM CAGR ($60\%$ of median), compressed EBITDA margins, zero market share gains.
- **Base (Expected):** Industry median CAGR, $+50\text{ bps}$ annual organic market share gains.
- **Best (Super-Cycle):** $+40\%$ TAM growth bonus (PLI / Export Tailwinds), operating leverage margin expansion, $+150\text{ bps}$ annual market share gains.

---

## 4. Local AI Model Integration (`C:\AI_Models`)

The financial engine directly integrates with `crates/core` (`ai_core`):

```text
C:\AI_Models\
├── bge_reranker\
│   ├── model.onnx          -> BgeRerankerLocal for ranking PDF filing disclosures
│   └── tokenizer.json
├── bge_small_embedding\
│   ├── model.onnx          -> BgeEmbeddingLocal for semantic filing vectors
│   └── tokenizer.json
└── qwen2-vl-2b\
    ├── llmmodel.onnx       -> QwenLocal for macro policy sentiment reasoning
    ├── visionmodel.onnx
    ├── vision_encoder.onnx
    └── tokenizer.json
```

### Graceful Fallback Guarantee
If local ONNX models are offline or GPU is occupied by other AI Core appliances:
- The engine automatically activates deterministic SIMD regex extraction and keyword policy evaluation.
- Zero exceptions, zero thread panics, and zero interruption to the 24x7 scanning loop.

---

## 5. Threading & Concurrency Architecture

- **Primary Runtime:** `tokio` multi-threaded async executor.
- **Background Worker (`scanner.rs`):** Detached Tokio task running at configurable intervals with `AtomicBool` graceful shutdown semantics.
- **Inter-Task Communication:** Tokio `broadcast` channels delivering `OutlierAlert` events to connected websockets or dashboards without blocking scanner execution.
- **Latency Guarantee:** In-memory cached reports generated in **$< 50\text{ ms}$**.
