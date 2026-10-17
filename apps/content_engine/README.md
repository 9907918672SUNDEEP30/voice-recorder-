# 🚀 Universal Autonomous Content & Media Engine (`apps/content_engine`)
**8th Autonomous App & Cross-App Integration — Shambhavi AI Labs**  
**Architect:** Sandeep Udasi  

---

## 📌 Overview

`content_engine` is an autonomous, niche-neutral digital media production pipeline. Given any subject (e.g. AI News, Stock Market, 80s Retro Trends), it performs deep cross-platform reconnaissance, writes viral scripts and blog articles, synthesizes voiceovers, extracts word captions, drives 4K human avatars via cloud GPUs, and publishes autonomously.

---

## ⚙️ Configuration (`.env`)

Create a `.env` file at the root of the repository:

```env
EXECUTION_ENV=LOCAL

LOCAL_MODELS_DIR=C:/AI_Models
LOCAL_QWEN_PATH=C:/AI_Models/qwen2-vl-2b
LOCAL_TTS_PATH=C:/AI_Models/piper_hi
LOCAL_WHISPER_PATH=C:/AI_Models/whisper_tiny_onnx
LOCAL_AVATAR_DEFAULT=assets/avatars/default_anchor.png
LOCAL_PREVIEW_DIR=output/previews
LOCAL_CONTENT_DIR=output/omnichannel_content

# Lightning AI Cloud Configuration (SSH)
LIGHTNING_SSH_HOST=lightning-ai-node
LIGHTNING_SSH_PORT=22
LIGHTNING_SSH_USER=zeus
LIGHTNING_SSH_KEY_PATH=~/.ssh/lightning_rsa
LIGHTNING_REMOTE_DIR=/home/zeus/content_engine_workspace
LIGHTNING_REMOTE_MODELS_DIR=/home/zeus/AI_Models
LIGHTNING_GPU_ACTIVE=false
DIRECT_CLOUD_PUBLISH=true
DOWNLOAD_PREVIEW_ONLY=true
```

---

## 🛠️ CLI Usage

### 1. Full Autonomous Pipeline
Run the complete end-to-end cycle for any dynamic topic:
```powershell
cargo run -p content_engine -- pipeline --topic "80s Retro AI Photo Trend"
```

### 2. Trend Reconnaissance Only
Analyze competitors, top YouTube videos, and reverse-engineer tools:
```powershell
cargo run -p content_engine -- recon --topic "Stock Market Crash 2026"
```

### 3. Generate Omnichannel Assets (Scripts, Blog, Twitter Thread)
```powershell
cargo run -p content_engine -- generate --topic "Quantum Computing Breakthroughs"
```

### 4. Synthesize Audio & Auto-Captions
```powershell
cargo run -p content_engine -- speech --text "Here is the breakthrough..."
```

---

## 📂 Output Structure

All generated campaign assets are cleanly organized:
```
output/omnichannel_content/
├── trend_dossier.json        # Competitor hooks, tools used, metrics
├── short_script.json         # 60s viral video script with visual cues
├── youtube_long_script.md    # 8-10m detailed explainer
├── seo_article.md            # Complete formatted blog post
├── twitter_thread.json       # 5-7 viral tweets
├── audio.wav                 # Mastered 24kHz studio voiceover
└── captions.ass              # Word-by-word glowing karaoke subtitles
```
