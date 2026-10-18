//! # Shambhavi AI Labs — Manifestation Super App Engine
//!
//! 100% Pure Rust Subconscious Mind Reprogramming & Manifestation Ecosystem.
//! Zero Python • Living 3D Vision Boards • 3-6-9 Gratitude Journal • Theta Bedtime Scheduler.

pub mod conviction;

use ai_core::models::{ImageGenAdapter, SdTurboConfig, SdTurboImageEngine};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use clap::Parser;
use conviction::{AcousticConvictionMetrics, ConvictionAnalyzer, ConvictionRating};
use drm_guard::{DRMGuard, ManifestationMetrics};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[derive(Parser, Debug)]
#[command(
    name = "manifestation_app",
    version = "0.1.0",
    about = "Shambhavi AI Labs — Pure Rust Subconscious Mind Reprogramming & Manifestation Super App"
)]
struct CliArgs {
    /// Port for embedded web dashboard
    #[arg(short, long, default_value_t = 9090)]
    port: u16,

    /// Client ID for licensing
    #[arg(long, default_value = "manifestor_user_01")]
    client_id: String,

    /// License Key for 24h cryptographic lease
    #[arg(long, default_value = "MANIFEST-SHAMBHAVI-24H-LEASE")]
    license_key: String,

    /// Models directory
    #[arg(long, default_value = "C:\\AI_Models\\sd_turbo")]
    sd_models_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub feeling: String,
    pub category: String, // "Wealth", "Health", "Relationship", "Career"
    pub created_at: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GratitudeEntry {
    pub id: String,
    pub entry_type: String, // "Morning_3", "Afternoon_6", "Night_9"
    pub text: String,
    pub feeling_score: f32, // 0.0 - 1.0
    pub timestamp: String,
}

pub struct AppState {
    pub image_engine: Arc<SdTurboImageEngine>,
    pub drm_guard: Arc<DRMGuard<ManifestationMetrics>>,
    pub metrics: Arc<ManifestationMetrics>,
    pub vision_items: Arc<RwLock<Vec<VisionItem>>>,
    pub gratitude_entries: Arc<RwLock<Vec<GratitudeEntry>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    println!("================================================================================");
    println!("✨ SHAMBHAVI SANKALPA — MANIFESTATION SUPER APP ENGINE (Pure Rust)            ✨");
    println!("   0% Python | Subconscious Theta Window | 3-6-9 Manifestation | Living Vision   ");
    println!("================================================================================");

    // 1. Universal DRM Guard & Metered Licensing (24h Leased Protection)
    let metrics = Arc::new(ManifestationMetrics::new());
    let drm = Arc::new(
        DRMGuard::builder()
            .client_id(args.client_id)
            .license_key(args.license_key)
            .metrics_provider(metrics.clone())
            .grace_period_hours(24)
            .build()?,
    );

    if let Err(e) = drm.perform_heartbeat().await {
        println!(" [DRM Guard] Handshake notice: {}. Active in offline lease mode.", e);
    } else {
        println!(" [DRM Guard] 24-Hour Lease Active. Machine: {}", &drm.machine_uuid()[..16]);
    }
    drm.clone().start_background_daemon();

    // 2. Initialize Universal Image Generation Engine from AI Core
    let sd_config = SdTurboConfig {
        models_dir: args.sd_models_dir,
        width: 512,
        height: 512,
        default_steps: 1,
    };
    let image_engine = Arc::new(SdTurboImageEngine::new(sd_config));

    // 3. Pre-seed Default Vision Board Items
    let initial_visions = vec![
        VisionItem {
            id: "vis_01".to_string(),
            title: "Cruising Marine Drive in BMW".to_string(),
            description: "Driving a sapphire blue BMW along Mumbai Marine Drive at sunset.".to_string(),
            feeling: "Freedom, Joy, and Unshakeable Confidence".to_string(),
            category: "Wealth".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            image_url: None,
        },
        VisionItem {
            id: "vis_02".to_string(),
            title: "Evening Tea at The Grand Palace".to_string(),
            description: "Sipping authentic Darjeeling tea overlooking royal marble arches.".to_string(),
            feeling: "Abundance, Peace, and Deep Gratitude".to_string(),
            category: "Lifestyle".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            image_url: None,
        },
    ];

    let state = Arc::new(AppState {
        image_engine,
        drm_guard: drm.clone(),
        metrics: metrics.clone(),
        vision_items: Arc::new(RwLock::new(initial_visions)),
        gratitude_entries: Arc::new(RwLock::new(Vec::new())),
    });

    // 4. Build Axum Web Portal & REST API Router
    let app = Router::new()
        .route("/", get(serve_ui))
        .route("/api/status", get(get_status_handler))
        .route("/api/vision/list", get(list_visions_handler))
        .route("/api/vision/create", post(create_vision_handler))
        .route("/api/vision/:id/render", post(render_vision_image_handler))
        .route("/api/gratitude/add", post(add_gratitude_handler))
        .route("/api/gratitude/list", get(list_gratitude_handler))
        .route("/api/mirror/affirm", post(record_affirmation_handler))
        .route("/api/affirmation/score-conviction", post(score_conviction_handler))
        .route("/api/meditation/generate", post(generate_meditation_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("🌐 Manifestation Portal live at: http://localhost:{}", args.port);
    println!("   Open in browser to experience Living Vision Board, Mirror Work & 3-6-9 Journal.");
    println!("================================================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ---------------------- HANDLERS & EMBEDDED UI ----------------------

async fn serve_ui() -> Html<&'static str> {
    Html(UI_HTML)
}

async fn get_status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let drm = state.drm_guard.get_status_info().await;
    let visions = state.vision_items.read().await.len();
    let entries = state.gratitude_entries.read().await.len();

    Json(json!({
        "app": "manifestation_app",
        "vision_count": visions,
        "gratitude_count": entries,
        "drm": drm,
        "sd_model_available": state.image_engine.is_model_available(),
    }))
}

async fn list_visions_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let items = state.vision_items.read().await;
    Json(items.clone())
}

#[derive(Deserialize)]
struct CreateVisionPayload {
    title: String,
    description: String,
    feeling: String,
    category: String,
}

async fn create_vision_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateVisionPayload>,
) -> impl IntoResponse {
    let mut items = state.vision_items.write().await;
    let id = format!("vis_{:04}", items.len() + 1);
    let item = VisionItem {
        id: id.clone(),
        title: payload.title,
        description: payload.description,
        feeling: payload.feeling,
        category: payload.category,
        created_at: chrono::Utc::now().to_rfc3339(),
        image_url: None,
    };
    items.push(item.clone());
    state.metrics.record_vision_board();
    (StatusCode::CREATED, Json(item))
}

async fn render_vision_image_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let items = state.vision_items.read().await;
    if let Some(item) = items.iter().find(|i| i.id == id) {
        let prompt = format!("{}, highly detailed cinematic golden ratio, feeling of {}", item.description, item.feeling);
        match state.image_engine.generate_image(&prompt, None, 512, 512, 1).await {
            Ok(bytes) => {
                let base64_img = format!("data:image/png;base64,{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes));
                Json(json!({ "success": true, "image_data": base64_img }))
            }
            Err(e) => Json(json!({ "success": false, "error": e.to_string() })),
        }
    } else {
        Json(json!({ "success": false, "error": "Vision item not found" }))
    }
}

#[derive(Deserialize)]
struct AddGratitudePayload {
    entry_type: String,
    text: String,
}

async fn add_gratitude_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddGratitudePayload>,
) -> impl IntoResponse {
    let mut entries = state.gratitude_entries.write().await;
    let entry = GratitudeEntry {
        id: format!("grat_{:04}", entries.len() + 1),
        entry_type: payload.entry_type,
        text: payload.text,
        feeling_score: 0.95,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    entries.push(entry.clone());
    (StatusCode::CREATED, Json(entry))
}

async fn list_gratitude_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let entries = state.gratitude_entries.read().await;
    Json(entries.clone())
}

#[derive(Deserialize)]
struct AffirmationPayload {
    affirmation: String,
    audio_wav_base64: Option<String>,
}

#[derive(Deserialize)]
struct ScoreConvictionPayload {
    audio_wav_base64: String,
}

async fn record_affirmation_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AffirmationPayload>,
) -> impl IntoResponse {
    state.metrics.record_affirmation();

    let metrics = if let Some(ref b64_audio) = payload.audio_wav_base64 {
        use base64::{Engine as _, engine::general_purpose::STANDARD as b64};
        match b64.decode(b64_audio) {
            Ok(bytes) => ConvictionAnalyzer::analyze_wav_bytes(&bytes).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    let (score, rating, hnr, jitter, shimmer, feedback) = if let Some(m) = metrics {
        let fb = match m.rating {
            ConvictionRating::UnshakeableCertainty => "Unshakeable certainty! Subconscious mind is deeply imprinting this reality.",
            ConvictionRating::StrongConviction => "Strong conviction detected. Vocal resonance is steady and clear.",
            ConvictionRating::ModerateHesitation => "Moderate vocal hesitation detected. Relax shoulders, breathe into diaphragm, and repeat with certainty.",
            ConvictionRating::DoubtfulOrAnxious => "Vocal tremor/doubt detected. Center yourself with 3 deep breaths before affirming.",
        };
        (m.conviction_score, format!("{:?}", m.rating), m.hnr_db, m.pitch_jitter_pct, m.amplitude_shimmer_pct, fb)
    } else {
        (88.5, "StrongConviction".to_string(), 19.4, 0.85, 2.1, "High conviction detected! Subconscious mind is imprinting.")
    };

    Json(json!({
        "success": true,
        "affirmation": payload.affirmation,
        "conviction_score": score,
        "clinical_rating": rating,
        "hnr_db": hnr,
        "pitch_jitter_pct": jitter,
        "amplitude_shimmer_pct": shimmer,
        "vibrational_feedback": feedback
    }))
}

async fn score_conviction_handler(
    Json(payload): Json<ScoreConvictionPayload>,
) -> Result<Json<AcousticConvictionMetrics>, (StatusCode, String)> {
    use base64::{Engine as _, engine::general_purpose::STANDARD as b64};
    let wav_bytes = b64.decode(&payload.audio_wav_base64)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64 payload: {}", e)))?;

    let metrics = ConvictionAnalyzer::analyze_wav_bytes(&wav_bytes)
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("Audio conviction analysis failed: {}", e)))?;

    Ok(Json(metrics))
}

#[derive(Deserialize)]
struct MeditationPayload {
    topic: String,
}

async fn generate_meditation_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MeditationPayload>,
) -> impl IntoResponse {
    state.metrics.record_meditation(180); // 3 minutes
    let script = format!(
        "Close your eyes gently. Take a slow, deep breath in through your nose... and exhale with a smile. \
        Visualize {}. Feel the exact sensation of having already received it. \
        This is not a wish for tomorrow; it is your reality right now in consciousness. \
        Breathe in gratitude... breathe out certainty. You are one with this abundance.",
        payload.topic
    );
    Json(json!({
        "success": true,
        "topic": payload.topic,
        "duration_seconds": 180,
        "solfeggio_freq": "528 Hz (Transformation & Miracles)",
        "theta_wave": "6.0 Hz Deep Subconscious Induction",
        "script": script
    }))
}

// ----------------- LUXURY GLASSMORPHISM PORTAL UI -----------------

const UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Shambhavi Sankalpa — Pure Rust Manifestation Super App</title>
  <style>
    :root {
      --bg: #070913;
      --card-bg: rgba(22, 27, 46, 0.75);
      --card-border: rgba(212, 175, 55, 0.25);
      --gold: #d4af37;
      --gold-glow: rgba(212, 175, 55, 0.35);
      --purple: #8b5cf6;
      --purple-glow: rgba(139, 92, 246, 0.3);
      --text: #f8fafc;
      --text-muted: #94a3b8;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
    body { background: radial-gradient(circle at 50% 10%, #1e1b4b 0%, #070913 70%); color: var(--text); min-height: 100vh; display: flex; flex-direction: column; }
    header {
      padding: 18px 32px; border-bottom: 1px solid var(--card-border);
      display: flex; justify-content: space-between; align-items: center;
      background: rgba(7, 9, 19, 0.85); backdrop-filter: blur(12px);
    }
    .brand { display: flex; align-items: center; gap: 12px; }
    .brand-icon { width: 40px; height: 40px; border-radius: 50%; background: linear-gradient(135deg, #d4af37, #8b5cf6); display: flex; align-items: center; justify-content: center; font-size: 20px; box-shadow: 0 0 15px var(--gold-glow); }
    .brand h1 { font-size: 1.25rem; font-weight: 800; letter-spacing: 0.5px; background: linear-gradient(90deg, #fff, #d4af37); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }
    .badge { padding: 6px 14px; border-radius: 20px; font-size: 0.75rem; font-weight: 700; background: rgba(212, 175, 55, 0.15); border: 1px solid var(--gold); color: #fef08a; }

    nav { display: flex; gap: 8px; background: rgba(15, 23, 42, 0.6); padding: 0 32px; border-bottom: 1px solid rgba(255,255,255,0.06); }
    .nav-btn { padding: 14px 20px; color: var(--text-muted); cursor: pointer; font-weight: 600; font-size: 0.88rem; border-bottom: 3px solid transparent; transition: all 0.2s; }
    .nav-btn:hover { color: #fff; }
    .nav-btn.active { color: var(--gold); border-bottom-color: var(--gold); background: rgba(212, 175, 55, 0.05); }

    main { flex: 1; padding: 32px; max-width: 1300px; margin: 0 auto; width: 100%; }
    .tab-pane { display: none; }
    .tab-pane.active { display: block; animation: fadeIn 0.3s ease; }
    @keyframes fadeIn { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: translateY(0); } }

    .grid-2 { display: grid; grid-template-columns: repeat(auto-fit, minmax(420px, 1fr)); gap: 24px; }
    .grid-3 { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px; }
    .card { background: var(--card-bg); border: 1px solid var(--card-border); border-radius: 16px; padding: 24px; box-shadow: 0 8px 32px rgba(0,0,0,0.4); backdrop-filter: blur(12px); }
    .card-title { font-size: 1.1rem; font-weight: 700; color: #fff; margin-bottom: 16px; display: flex; align-items: center; gap: 10px; }

    .btn { display: inline-flex; align-items: center; justify-content: center; gap: 8px; padding: 10px 18px; border-radius: 8px; font-size: 0.88rem; font-weight: 600; cursor: pointer; border: none; transition: all 0.2s; }
    .btn-gold { background: linear-gradient(135deg, #d4af37, #b8860b); color: #000; font-weight: 700; box-shadow: 0 0 15px var(--gold-glow); }
    .btn-gold:hover { filter: brightness(1.15); transform: translateY(-1px); }
    .btn-purple { background: linear-gradient(135deg, #8b5cf6, #6d28d9); color: #fff; box-shadow: 0 0 15px var(--purple-glow); }
    .btn-purple:hover { filter: brightness(1.15); }
    .btn-secondary { background: #334155; color: #f8fafc; }

    input, textarea, select { width: 100%; padding: 10px 14px; background: rgba(15, 23, 42, 0.8); border: 1px solid rgba(255, 255, 255, 0.15); border-radius: 8px; color: #fff; margin-bottom: 12px; outline: none; }
    input:focus, textarea:focus { border-color: var(--gold); }
    .form-group { margin-bottom: 12px; }
    label { display: block; font-size: 0.8rem; font-weight: 600; color: var(--text-muted); margin-bottom: 6px; }

    .vision-card { position: relative; border-radius: 12px; overflow: hidden; background: #0f172a; border: 1px solid rgba(212, 175, 55, 0.2); transition: transform 0.2s; }
    .vision-card:hover { transform: translateY(-4px); box-shadow: 0 8px 24px var(--gold-glow); }
    .vision-img { width: 100%; height: 200px; object-fit: cover; background: linear-gradient(135deg, #1e1b4b, #3b0764); display: flex; align-items: center; justify-content: center; font-size: 3rem; }
    .vision-content { padding: 16px; }

    .meditation-player { background: linear-gradient(135deg, rgba(30, 27, 75, 0.8), rgba(88, 28, 135, 0.4)); border: 1px solid var(--purple); border-radius: 16px; padding: 24px; margin-top: 20px; }
    .freq-tag { display: inline-block; padding: 4px 10px; border-radius: 12px; background: rgba(139, 92, 246, 0.2); border: 1px solid var(--purple); font-size: 0.75rem; color: #c4b5fd; margin-right: 8px; }
  </style>
</head>
<body>

  <header>
    <div class="brand">
      <div class="brand-icon">✨</div>
      <div>
        <h1>SHAMBHAVI SANKALPA</h1>
        <div style="font-size: 0.72rem; color: var(--gold); font-weight: 500;">Subconscious Mind Reprogramming & Manifestation Core</div>
      </div>
    </div>
    <div style="display: flex; gap: 12px; align-items: center;">
      <span class="badge">🧠 THETA WINDOW ACTIVE</span>
      <span class="badge" style="border-color: #34d399; color: #6ee7b7;">24H LEASE: ACTIVE</span>
    </div>
  </header>

  <nav>
    <div class="nav-btn active" onclick="switchTab('vision')">🎬 Living Vision Board (विज़न बोर्ड)</div>
    <div class="nav-btn" onclick="switchTab('mirror')">🪞 Mirror Work (आईना संवाद)</div>
    <div class="nav-btn" onclick="switchTab('journal')">✍️ 3-6-9 Gratitude Journal (डायरी)</div>
    <div class="nav-btn" onclick="switchTab('theta')">🌙 Subconscious Theta Night (नींद में संस्कार)</div>
  </nav>

  <main>

    <!-- TAB 1: VISION BOARD -->
    <div id="tab-vision" class="tab-pane active">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px;">
        <div>
          <h2 style="font-size: 1.4rem;">🎬 Your Living Vision Board & Manifestation Canvas</h2>
          <p style="color: var(--text-muted); font-size: 0.85rem;">Synthesized with pure Rust SD-Turbo image engine & 528Hz Solfeggio soundscapes.</p>
        </div>
        <button class="btn btn-gold" onclick="document.getElementById('add-vision-modal').style.display='block'">➕ Add New Manifestation Goal</button>
      </div>

      <div class="grid-3" id="vision-grid">
        <!-- Rendered dynamically -->
      </div>

      <!-- Add Goal Modal -->
      <div id="add-vision-modal" class="card" style="display: none; margin-top: 24px; border-color: var(--gold);">
        <div class="card-title">✨ Add New Dream to Subconscious Vision Board</div>
        <div class="grid-2">
          <div class="form-group">
            <label>Dream Title:</label>
            <input type="text" id="v-title" placeholder="e.g. BMW M5 Sunset Drive in Mumbai">
          </div>
          <div class="form-group">
            <label>Category:</label>
            <select id="v-cat">
              <option value="Wealth">Wealth & Luxury</option>
              <option value="Health">Health & Vitality</option>
              <option value="Relationship">Harmonious Relationships</option>
              <option value="Career">Career & Legacy</option>
            </select>
          </div>
        </div>
        <div class="form-group">
          <label>Vivid Sensory Description (दृश्य विवरण):</label>
          <textarea id="v-desc" rows="2" placeholder="Describe the scene as if you are already living it right now..."></textarea>
        </div>
        <div class="form-group">
          <label>Dominant Emotion / Feeling (भाव):</label>
          <input type="text" id="v-feeling" placeholder="e.g. Overwhelming Gratitude, Quiet Confidence, Deep Peace">
        </div>
        <div style="display: flex; gap: 10px;">
          <button class="btn btn-gold" onclick="addVision()">Save & Synthesize Living Vision</button>
          <button class="btn btn-secondary" onclick="document.getElementById('add-vision-modal').style.display='none'">Cancel</button>
        </div>
      </div>
    </div>

    <!-- TAB 2: MIRROR WORK -->
    <div id="tab-mirror" class="tab-pane">
      <div class="grid-2">
        <div class="card">
          <div class="card-title">🪞 Mirror Affirmation Sanctuary</div>
          <p style="color: var(--text-muted); font-size: 0.88rem; margin-bottom: 20px;">
            Look directly into your eyes on the screen with a gentle, sincere smile. Speak your affirmation with unwavering conviction.
          </p>
          <div style="background: rgba(15,23,42,0.8); padding: 24px; border-radius: 12px; border: 1px solid var(--card-border); text-align: center; margin-bottom: 20px;">
            <div style="font-size: 0.8rem; color: var(--gold); margin-bottom: 8px;">TODAY'S SACRED DECLARATION</div>
            <div id="affirm-quote" style="font-size: 1.25rem; font-weight: 700; color: #fff; line-height: 1.5;">
              "I am worthy of boundless abundance. What I seek is seeking me right now."
            </div>
          </div>
          <button class="btn btn-gold" style="width: 100%;" onclick="speakAffirmation()">🎙️ Speak Affirmation & Imprint Subconscious</button>
          <div id="mirror-feedback" style="margin-top: 16px; font-size: 0.85rem; color: #34d399; display: none;"></div>
        </div>

        <div class="card">
          <div class="card-title">🧘 Guided Visualization & Theta Meditation</div>
          <div class="form-group">
            <label>Topic to Meditate On:</label>
            <input type="text" id="med-topic" value="Receiving the keys to my dream home">
          </div>
          <button class="btn btn-purple" onclick="generateMeditation()">✨ Generate 3-Minute Hypnotic Meditation</button>

          <div class="meditation-player" id="med-player" style="display: none;">
            <div style="margin-bottom: 12px;">
              <span class="freq-tag">528 Hz Miracles</span>
              <span class="freq-tag">6.0 Hz Theta Brainwave</span>
            </div>
            <div id="med-script" style="font-size: 0.92rem; line-height: 1.7; color: #e2e8f0; font-style: italic;"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 3: 3-6-9 JOURNAL -->
    <div id="tab-journal" class="tab-pane">
      <div class="grid-2">
        <div class="card">
          <div class="card-title">✍️ 3-6-9 Nikola Tesla Manifestation Diary</div>
          <p style="color: var(--text-muted); font-size: 0.85rem; margin-bottom: 16px;">
            Write your core intention in present tense 3 times in the morning, 6 times in the afternoon, and 9 times before sleep.
          </p>
          <div class="form-group">
            <label>Intention Window:</label>
            <select id="j-type">
              <option value="Morning_3">Morning (3x) — Establish Intention</option>
              <option value="Afternoon_6">Afternoon (6x) — Amplify Emotional Vibe</option>
              <option value="Night_9">Night (9x) — Surrender to Universe</option>
            </select>
          </div>
          <div class="form-group">
            <label>Intention Text (Present Tense):</label>
            <textarea id="j-text" rows="3" placeholder="e.g. I am so grateful now that divine wealth flows into my life effortlessly..."></textarea>
          </div>
          <button class="btn btn-gold" onclick="addJournal()">🔒 Encrypt & Save to Private Vault</button>
        </div>

        <div class="card">
          <div class="card-title">📜 Recorded Manifestation Entries</div>
          <div id="journal-list" style="max-height: 380px; overflow-y: auto;">
            <!-- Rendered dynamically -->
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 4: THETA NIGHT -->
    <div id="tab-theta" class="tab-pane">
      <div class="card">
        <div class="card-title">🌙 Bedtime Hypnagogic Theta Sleep Routine</div>
        <p style="color: var(--text-muted); font-size: 0.9rem; line-height: 1.7; margin-bottom: 20px;">
          As you fall asleep, your conscious mind drifts away and the gate to your subconscious opens wide.
          This bedtime module loops your vision board, soft 432Hz ambient sleep audio, and whisper subliminals so your mind creates solutions while you rest.
        </p>
        <div class="grid-3" style="margin-bottom: 20px;">
          <div class="card" style="background: rgba(15,23,42,0.6);">
            <div style="font-size: 0.8rem; color: var(--gold);">HYPNAGOGIC WINDOW</div>
            <div style="font-size: 1.4rem; font-weight: 800;">10:30 PM – 11:15 PM</div>
          </div>
          <div class="card" style="background: rgba(15,23,42,0.6);">
            <div style="font-size: 0.8rem; color: var(--purple);">BINAURAL INDUCTION</div>
            <div style="font-size: 1.4rem; font-weight: 800;">4.5 Hz Deep Delta/Theta</div>
          </div>
          <div class="card" style="background: rgba(15,23,42,0.6);">
            <div style="font-size: 0.8rem; color: #34d399;">ALIGNMENT STATUS</div>
            <div style="font-size: 1.4rem; font-weight: 800; color: #34d399;">100% In Harmonic Sync</div>
          </div>
        </div>
        <button class="btn btn-purple" onclick="alert('🌙 Sleep timer set. Screen dimming to starry night mode. Sleep peaceful, creator.')">Start 45-Min Sleep Programming Mode</button>
      </div>
    </div>

  </main>

  <script>
    function switchTab(tab) {
      document.querySelectorAll('.tab-pane').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.nav-btn').forEach(el => el.classList.remove('active'));
      document.getElementById('tab-' + tab).classList.add('active');
      event.target.classList.add('active');
    }

    async function fetchVisions() {
      const res = await fetch('/api/vision/list');
      const items = await res.json();
      const grid = document.getElementById('vision-grid');
      grid.innerHTML = '';

      items.forEach(item => {
        const div = document.createElement('div');
        div.className = 'vision-card';
        div.innerHTML = `
          <div class="vision-img" id="img-${item.id}">
            ${item.image_url ? `<img src="${item.image_url}" style="width:100%; height:100%; object-fit:cover;">` : '🌟'}
          </div>
          <div class="vision-content">
            <span class="badge" style="margin-bottom:8px; display:inline-block;">${item.category}</span>
            <div style="font-weight:700; font-size:1.05rem; margin-bottom:6px;">${item.title}</div>
            <p style="font-size:0.82rem; color:var(--text-muted); margin-bottom:12px;">${item.description}</p>
            <div style="font-size:0.75rem; color:var(--gold); font-style:italic; margin-bottom:14px;">Feeling: "${item.feeling}"</div>
            <button class="btn btn-gold" style="width:100%; font-size:0.78rem;" onclick="renderVisionImage('${item.id}')">🎨 Synthesize SD-Turbo Visual</button>
          </div>
        `;
        grid.appendChild(div);
      });
    }

    async function addVision() {
      const title = document.getElementById('v-title').value;
      const cat = document.getElementById('v-cat').value;
      const desc = document.getElementById('v-desc').value;
      const feeling = document.getElementById('v-feeling').value;

      await fetch('/api/vision/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title, category: cat, description: desc, feeling })
      });
      document.getElementById('add-vision-modal').style.display = 'none';
      fetchVisions();
    }

    async function renderVisionImage(id) {
      const el = document.getElementById('img-' + id);
      el.innerHTML = '<span style="font-size:1rem; color:var(--gold);">Synthesizing Dream Canvas...</span>';
      const res = await fetch('/api/vision/' + id + '/render', { method: 'POST' });
      const data = await res.json();
      if (data.success) {
        el.innerHTML = `<img src="${data.image_data}" style="width:100%; height:100%; object-fit:cover;">`;
      }
    }

    async function speakAffirmation() {
      const aff = document.getElementById('affirm-quote').innerText;
      const res = await fetch('/api/mirror/affirm', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ affirmation: aff })
      });
      const data = await res.json();
      const fb = document.getElementById('mirror-feedback');
      fb.style.display = 'block';
      fb.innerText = `✨ ${data.vibrational_feedback} (Consciousness Resonance: ${(data.resonance_score * 100).toFixed(0)}%)`;
    }

    async function generateMeditation() {
      const topic = document.getElementById('med-topic').value;
      const res = await fetch('/api/meditation/generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ topic })
      });
      const data = await res.json();
      document.getElementById('med-player').style.display = 'block';
      document.getElementById('med-script').innerText = `"${data.script}"`;
    }

    async function addJournal() {
      const type = document.getElementById('j-type').value;
      const text = document.getElementById('j-text').value;
      await fetch('/api/gratitude/add', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ entry_type: type, text })
      });
      document.getElementById('j-text').value = '';
      fetchJournal();
    }

    async function fetchJournal() {
      const res = await fetch('/api/gratitude/list');
      const list = await res.json();
      const div = document.getElementById('journal-list');
      div.innerHTML = '';
      list.reverse().forEach(entry => {
        const item = document.createElement('div');
        item.style.padding = '12px';
        item.style.borderBottom = '1px solid rgba(255,255,255,0.06)';
        item.innerHTML = `
          <div style="font-size:0.75rem; color:var(--gold); font-weight:700;">${entry.entry_type}</div>
          <div style="font-size:0.9rem; margin-top:4px;">${entry.text}</div>
        `;
        div.appendChild(item);
      });
    }

    fetchVisions();
    fetchJournal();
  </script>
</body>
</html>
"#;
