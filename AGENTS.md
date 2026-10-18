# 🤖 Antigravity Agent Guidelines & Operational Rules (AGENTS.md)
## Workspace: `d:\ai_core_framework - Copy (2)`

---

### 🚨 MANDATORY PROTOCOL: Single Source of Truth (`SYSTEM_STATE.md`)

1. **FIRST STEP ON ANY TASK:**
   - **DO NOT** perform broad `grep_search`, recursive directory scanning, or multi-file exploratory searches across the workspace.
   - **ALWAYS** read [SYSTEM_STATE.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/SYSTEM_STATE.md) first to get the immediate, exact state of the core framework, all 6 crates, and all 8 apps.

2. **PREVENT TOKEN & TIME WASTE:**
   - [SYSTEM_STATE.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/SYSTEM_STATE.md) contains the authoritative, concise ledger of all crates, applications, key source files, port numbers, DRM configurations, model file paths, and build commands.
   - When asked to inspect, modify, or run any application, look up its exact file path from `SYSTEM_STATE.md` and take **immediate targeted action**.

3. **IMMEDIATE SYNCHRONIZATION MANDATE:**
   - Whenever any architectural change, new feature, dependency modification, port change, or file creation occurs in the workspace, you **MUST** update [SYSTEM_STATE.md](file:///d:/ai_core_framework%20-%20Copy%20(2)/SYSTEM_STATE.md) in the exact same turn.
   - Keep `SYSTEM_STATE.md` concise, structured, high-density, and free of fluff.

4. **DRM & LICENSING PRESERVATION:**
   - All 8 apps must preserve the `crates/drm_guard` integration (24-hour rotating cryptographic lease + 10-minute heartbeat ping + `ZeroizeOnDrop` memory zeroization + Jio-TV style silent OTA atomic updater).

---

### 💎 5. MANDATORY ENTERPRISE PRODUCTION QUALITY MANDATE (ZERO DUMMY POLICY):

- **NEVER WRITE DUMMY, PROTOTYPE, OR SHORTCUT CODE:**
  - Absolutely **ZERO** placeholder functions, fake random generators, mock tensor returns, dummy model inference, or crude pixel-counting heuristics.
  - If an AI model or DSP algorithm is specified, it must execute authentic tensor operations (e.g. Qwen2-VL 2D patch extraction `[N, 1176]`, 80-channel Hann + Real DFT + Mel filterbanks) or return a typed, handled error.
- **DEPLOYMENT-GRADE STANDARD (Google / Microsoft / Adobe Tier):**
  - All code must be built for final enterprise production deployment.
  - Every line must withstand rigorous line-by-line audits by top-tier models (Claude Opus 5, ChatGPT-6, Grok 4.2) and Principal Systems Architects with **ZERO shortcuts or flaws found**.
- **FAIL-CLOSED CRYPTOGRAPHIC RIGOR:**
  - Never fail-open. Enforce Ed25519 root signatures, AES-256-GCM encryption, monotonic anti-rollback clocks, and `ZeroizeOnDrop` volatile memory scrubbing.
- **MATHEMATICAL & FINANCIAL DETERMINISM:**
  - All financial ratios, S-Curve TAM forecasts, and multiple expansion re-rating models must follow institutional CFA/SEBI standards with zero fake multipliers.
