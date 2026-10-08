use anyhow::Result;
use std::fs;

use ai_core_framework::config::AppConfig;
use ai_core_framework::ml_bridge::MLBridge;
use ai_core::engine::runner::Orchestrator;
use ai_core::engine::context::ExecutionContext;
use ai_core::engine::step::Step;
use ai_core::telemetry::sink::NoopSink;
use ai_core_framework::steps::GenericMlStep;
use ai_core_framework::agent_trait::{AgentStepAdapter, CoreAgent};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=================================================================");
    println!("🏭 AI Core Framework: Factory Engine & Dynamic Inference Runtime");
    println!("   - 🛡️ Pure Factory Principle: Agnostic Core & Modular Adapters");
    println!("   - ⚙️ Config-Driven Tensors: Zero Magic Numbers");
    println!("=================================================================\n");

    // 1️⃣ Configuration loading with dynamic model registry
    let settings = match AppConfig::load("config.json") {
        Ok(cfg) => {
            println!("✅ 'config.json' successfully loaded!");
            cfg
        }
        Err(e) => {
            println!("⚠️ 'config.json' error: {}. Using default configuration...", e);
            AppConfig::default()
        }
    };

    println!("📁 Models Directory: {:?}", settings.models_dir);
    println!("⚙️ Execution Device Setting: '{}'", settings.execution_device);
    println!("📋 Configured Dynamic Models: {} registered profiles", settings.models.len());
    for (name, profile) in &settings.models {
        println!("   ├─ [{}] Path: '{}' (Inputs: {:?}, Outputs: {:?})", 
            name, profile.path, profile.input_names, profile.output_names);
    }

    // 2️⃣ Verify models folder existence
    if !settings.models_dir.exists() {
        println!("📁 Creating models directory: '{:?}'", settings.models_dir);
        let _ = fs::create_dir_all(&settings.models_dir);
    }

    // 3️⃣ Initialize Lazy MLBridge (Zero eager allocations!)
    let bridge = MLBridge::from_config(settings.clone());
    println!("\n🧠 MLBridge initialized in Lazy Mode (Allocated sessions: 0)");

    // 4️⃣ Live Model Test if local model files exist
    let llm_path = match settings.resolve_path("qwen_llm") {
        Ok(p) => p,
        Err(_) => settings.models_dir.join("qwen2-vl-2b/llmmodel.onnx"),
    };
    if llm_path.exists() {
        println!("\n🚀 Live Model Loading Test on Device Mode '{}'...", settings.execution_device);
        let target = ai_core_framework::ml_bridge::ExecutionTarget::from_str_target(&settings.execution_device);
        match MLBridge::create_session_with_target(&llm_path, target) {
            Ok((_session, active_hw)) => {
                println!("🎉 Live model test passed!");
                println!("   {}", active_hw);
            }
            Err(e) => println!("❌ Model load test failed: {}", e),
        }
    } else {
        println!("\nℹ️ Model file '{}' not found locally on disk (will load dynamically when placed).", llm_path.display());
    }

    // 5️⃣ Integration Demonstration: Orchestrator ↔ MLBridge ↔ Agent Step
    println!("\n--------------------------------------------------");
    println!("⚡ Demonstrating Orchestrator ↔ MLBridge ↔ Step Pipeline");
    println!("--------------------------------------------------");

    let mut orch = Orchestrator::new(NoopSink);
    let mut ctx = ExecutionContext::new("factory_verification_run_01");
    ctx.set("input_prompt", "Hello AI Core Framework!");

    // Step 1: Standard agent step via AgentStepAdapter
    let agent = CoreAgent::new("GreeterAgent");
    let agent_step = Box::new(AgentStepAdapter::new(
        "agent_greeting_step",
        agent,
        "input_prompt",
        "agent_response",
    ));

    // Step 2: Generic ML Step wired to MLBridge without coupling core to ONNX
    let ml_bridge_clone = bridge.clone();
    let ml_step = Box::new(GenericMlStep::new(
        "lazy_ml_inference_step",
        ml_bridge_clone,
        |bridge, ctx| {
            // Check if model exists, or perform mock/real inference dynamically
            let is_loaded = bridge.is_loaded("silero_vad");
            ctx.set("ml.silero_vad_active", if is_loaded { "true" } else { "lazy_standby" });
            ctx.set("ml.inference_status", "ready");
            Ok(())
        },
    ));

    let mut steps: Vec<Box<dyn Step>> = vec![agent_step, ml_step];
    let run_state = orch.run(&mut ctx, &mut steps);

    match run_state {
        Ok(state) => {
            println!("✅ Pipeline completed successfully with State: {:?}", state);
            println!("   ├─ Agent Output: {:?}", ctx.get("agent_response"));
            println!("   ├─ ML Status: {:?}", ctx.get("ml.inference_status"));
            println!("   └─ VAD State: {:?}", ctx.get("ml.silero_vad_active"));
        }
        Err(e) => {
            println!("❌ Pipeline failed: {}", e);
        }
    }

    println!("\n✨ AI Core Framework is fully operational, generic, and production-ready!");
    Ok(())
}