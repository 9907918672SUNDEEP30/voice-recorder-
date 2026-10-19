use anyhow::Result;
use content_engine::generator::{OmnichannelPackage, ScriptLine, ShortVideoScript};
use content_engine::recon::{CompetitorVideo, NewsArticle, TrendDossier};
use content_engine::supabase::SupabaseContentClient;
use std::fs;

#[tokio::test]
async fn test_content_engine_supabase_cloud_lifecycle() -> Result<()> {
    // 1. Load config
    let config_content = fs::read_to_string("config.json")
        .or_else(|_| fs::read_to_string("../../config.json"))
        .unwrap_or_else(|_| "{}".to_string());
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    let client = SupabaseContentClient::new_from_env_or_config(&config);

    // 2. Build sample OmnichannelPackage
    println!("Step 1: Preparing Omnichannel Content Matrix...");
    let short_script = ShortVideoScript {
        topic: "Shambhavi AI Pehredar NVR Launch".to_string(),
        hook: "Is your commercial warehouse CCTV vulnerable to offline wire cuts?".to_string(),
        lines: vec![
            ScriptLine {
                timestamp_sec: 0.0,
                spoken_text: "Traditional NVRs fail silently when cables are severed.".to_string(),
                visual_cue: "Close up of cut ethernet cable with warning indicator".to_string(),
            },
            ScriptLine {
                timestamp_sec: 5.5,
                spoken_text: "Pehredar NVR triggers instant 10-minute proactive heartbeats to the Cloudflare edge.".to_string(),
                visual_cue: "Animated telemetry pulse reaching edge server".to_string(),
            },
            ScriptLine {
                timestamp_sec: 12.0,
                spoken_text: "You get alerted on WhatsApp within 250 milliseconds.".to_string(),
                visual_cue: "Smartphone notification showing live camera snapshot".to_string(),
            },
        ],
        cta: "Deploy Pehredar on-premise hardware today at shambhaviai.com".to_string(),
        full_spoken_transcript: "Traditional NVRs fail silently when cables are severed. Pehredar NVR triggers instant 10-minute proactive heartbeats to the Cloudflare edge. You get alerted on WhatsApp within 250 milliseconds. Deploy Pehredar on-premise hardware today at shambhaviai.com".to_string(),
    };

    let long_form_script = r#"# Pehredar NVR: The Complete On-Premises Architecture Breakdown

## Chapter 1: The Zero-Trust CCTV Security Problem (00:00 - 03:45)
Most commercial security systems rely on passive recording...

## Chapter 2: Hardware Fingerprinting and Ed25519 Leases (03:45 - 08:30)
Using CPU UUID, Motherboard Serial, and MAC Address...

## Chapter 3: Live Benchmark vs Hikvision & Dahua (08:30 - 14:15)
Real-time sub-5ms YOLOv8 inference directly on RTSP streams...
"#.to_string();

    let blog_article = r#"# Building an Autonomous Enterprise CCTV Appliance with Pure Rust

Commercial security cameras are often vulnerable to silent network partition attacks...

## Why Ed25519 Signed Heartbeats Matter
By incorporating rotating 24-hour cryptographic leases and 10-minute proactive edge telemetry...

## Frequently Asked Questions (FAQ)
### Can Pehredar work completely offline?
Yes, Pehredar continues recording locally with fail-closed cryptographic safeguards...
"#.to_string();

    let twitter_thread = vec![
        "1/5 🚨 90% of warehouse security breaches happen when cameras are silently unplugged. Here is how we fixed it forever with Rust & Edge AI: 🧵".to_string(),
        "2/5 Traditional CCTV systems don't alert you until after an incident. We built 10-minute active wire-fault telemetry directly into the Linux NVR kernel.".to_string(),
        "3/5 Powered by DirectML YOLOv8 and zero-copy RTSP streams, it classifies threats in <5ms without sending video to third-party clouds.".to_string(),
        "4/5 All licensing runs through Cloudflare D1 with Ed25519 root signatures for tamper-proof protection.".to_string(),
        "5/5 Read the full technical whitepaper and architecture guide on our blog: https://shambhaviai.com/pehredar-arch".to_string(),
    ];

    let package = OmnichannelPackage {
        topic: "Shambhavi AI Pehredar NVR Launch".to_string(),
        short_script,
        long_form_script,
        blog_article,
        twitter_thread,
    };

    // 3. Save package to Supabase Cloud
    println!("Step 2: Persisting Omnichannel Package to Supabase...");
    let content_id = client.save_content_package(
        &package,
        Some("https://storage.shambhaviai.com/audio/pehredar_short_voiceover.wav"),
        Some("https://storage.shambhaviai.com/captions/pehredar_karaoke.ass"),
        Some("https://storage.shambhaviai.com/video/pehredar_preview_1080p.mp4"),
    ).await?;

    println!("Saved Content Piece ID: {}", content_id);
    assert!(!content_id.is_empty(), "Content Piece ID must be returned");

    // 4. Save Trend Dossier to Supabase Cloud
    println!("Step 3: Saving Research Trend Dossier to Supabase...");
    let dossier = TrendDossier {
        topic: "Shambhavi AI Pehredar NVR Launch".to_string(),
        timestamp_utc: chrono::Utc::now().to_rfc3339(),
        top_competitors: vec![
            CompetitorVideo {
                title: "Best Security Camera Systems 2026".to_string(),
                video_id: "sample001".to_string(),
                channel: "Tech Security Reviews".to_string(),
                view_count_str: "350K views".to_string(),
                estimated_views: 350000,
                upload_time_str: "14 days ago".to_string(),
                estimated_hours_ago: 336.0,
                view_velocity: 1041.6,
            }
        ],
        breaking_news: vec![
            NewsArticle {
                title: "Edge AI in Surveillance: The 2026 Security Revolution".to_string(),
                link: "https://securityjournal.io/edge-ai-surveillance".to_string(),
                source: "Security Tech Journal".to_string(),
                published_date: "2026-10-15".to_string(),
                summary_snippet: "On-premises AI processing replaces cloud-dependent CCTV monitoring.".to_string(),
            }
        ],
        detected_viral_hooks: vec![
            "Is your CCTV secretly offline right now?".to_string(),
            "Why commercial warehouses are ditching cloud NVRs".to_string(),
        ],
        reverse_engineered_tools: vec!["YOLOv8 ONNX".to_string(), "FFmpeg NVENC".to_string()],
        target_audience_pain_point: "Silent wire cuts and camera sabotage without operator notification".to_string(),
        recommended_angle: "Exposing silent CCTV failures and presenting fail-closed edge heartbeats".to_string(),
    };

    let dossier_id = client.save_trend_dossier(&dossier).await?;
    println!("Saved Trend Dossier ID: {}", dossier_id);
    assert!(!dossier_id.is_empty(), "Dossier ID must be returned");

    // 5. Log Cross-Platform Publishing Events
    println!("Step 4: Logging syndication publishing events...");
    client.log_publishing_event(&content_id, "youtube_shorts", Some("yt_short_9921"), "published").await?;
    client.log_publishing_event(&content_id, "twitter_x", Some("x_status_18928371"), "published").await?;
    client.log_publishing_event(&content_id, "blog_seo", Some("blog_post_441"), "published").await?;

    // 6. Fetch recent campaigns from Supabase Cloud
    println!("Step 5: Verifying retrieval from Supabase Cloud...");
    let recent = client.fetch_recent_campaigns(5).await?;
    println!("Fetched {} campaigns from cloud.", recent.len());
    let found = recent.iter().find(|c| c.id.as_deref() == Some(&content_id));
    assert!(found.is_some(), "The newly saved content piece must exist in Supabase");
    let piece = found.unwrap();
    assert_eq!(piece.topic, "Shambhavi AI Pehredar NVR Launch");
    assert!(piece.short_hook.is_some());
    assert!(piece.blog_article.is_some());
    assert!(piece.twitter_thread.is_some());

    println!("✅ All 5 Content Engine Cloud Persistence Operations Verified Live on Supabase!");
    Ok(())
}
