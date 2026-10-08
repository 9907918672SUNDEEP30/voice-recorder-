use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ai_core::engine::context::ExecutionContext;
use ai_core::engine::runner::Orchestrator;
use ai_core::engine::step::{Step, StepOutcome};
use ai_core::telemetry::sink::TelemetrySink;

/// एजेंट के लिए एक सामान्य इंटरफ़ेस (Trait) जो async-trait का उपयोग करता है
#[async_trait]
pub trait AgentComponent: Send + Sync {
    /// कंपोनेंट को शुरू करने का मेथड
    async fn initialize(&mut self) -> Result<()>;

    /// इनपुट प्रॉम्प्ट या डेटा को प्रोसेस करके आउटपुट देने का मेथड
    async fn process(&mut self, input: &str) -> Result<String>;
}

/// एक उदाहरण के तौर पर बुनियादी एजेंट ढांचा
pub struct CoreAgent {
    pub name: String,
}

impl CoreAgent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl AgentComponent for CoreAgent {
    async fn initialize(&mut self) -> Result<()> {
        println!("🤖 एजेंट '{}' इनिशियलाइज हो रहा है...", self.name);
        Ok(())
    }

    async fn process(&mut self, input: &str) -> Result<String> {
        println!("📥 एजेंट को इनपुट मिला: {}", input);
        let response = format!("एजेंट '{}' ने प्रोसेस किया: {}", self.name, input);
        Ok(response)
    }
}

// ============================================================================
// 🔄 Unifying Abstractions: Adapters between AgentComponent and crates/core Step
// ============================================================================

/// Adapter that wraps any `AgentComponent` and exposes it as an atomic `Step`
/// for the `crates/core` Orchestrator.
pub struct AgentStepAdapter<T: AgentComponent> {
    id: String,
    pub component: T,
    input_key: String,
    output_key: String,
    retries: u32,
}

impl<T: AgentComponent> AgentStepAdapter<T> {
    pub fn new(
        id: impl Into<String>,
        component: T,
        input_key: impl Into<String>,
        output_key: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            component,
            input_key: input_key.into(),
            output_key: output_key.into(),
            retries: 0,
        }
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }
}

impl<T: AgentComponent> Step for AgentStepAdapter<T> {
    fn id(&self) -> &str {
        &self.id
    }

    fn max_retries(&self) -> u32 {
        self.retries
    }

    fn execute(&mut self, ctx: &mut ExecutionContext) -> StepOutcome {
        let input_val = match ctx.get(&self.input_key) {
            Some(v) => v.to_string(),
            None => return StepOutcome::Fatal(format!("Context missing input key '{}'", self.input_key)),
        };

        // Bridge async execution into the synchronous engine step safely
        // NOTE: block_in_place requires a multi-threaded Tokio runtime.
        // Using current_thread runtime will panic here.
        let res = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(self.component.process(&input_val))
            })
        } else {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => return StepOutcome::Fatal(format!("Failed to create runtime: {}", e)),
            };
            rt.block_on(self.component.process(&input_val))
        };

        match res {
            Ok(output) => {
                ctx.set(&self.output_key, output);
                StepOutcome::Success
            }
            Err(e) => StepOutcome::Retry(e.to_string()),
        }
    }
}

/// Orchestrated Pipeline Agent
/// Wraps a chain of orchestrated `Step`s and exposes them as a unified `AgentComponent`.
pub struct OrchestratedPipelineAgent<T: TelemetrySink> {
    pub name: String,
    pub orchestrator: std::sync::Mutex<Orchestrator<T>>,
    pub steps: std::sync::Mutex<Vec<Box<dyn Step>>>,
    pub input_key: String,
    pub output_key: String,
}

impl<T: TelemetrySink> OrchestratedPipelineAgent<T> {
    pub fn new(
        name: impl Into<String>,
        orchestrator: Orchestrator<T>,
        steps: Vec<Box<dyn Step>>,
        input_key: impl Into<String>,
        output_key: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            orchestrator: std::sync::Mutex::new(orchestrator),
            steps: std::sync::Mutex::new(steps),
            input_key: input_key.into(),
            output_key: output_key.into(),
        }
    }
}

#[async_trait]
impl<T: TelemetrySink + Send + Sync> AgentComponent for OrchestratedPipelineAgent<T> {
    async fn initialize(&mut self) -> Result<()> {
        println!("🚀 OrchestratedPipelineAgent '{}' initialized.", self.name);
        Ok(())
    }

    async fn process(&mut self, input: &str) -> Result<String> {
        let run_id = format!("run_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis());
        let mut ctx = ExecutionContext::new(run_id);
        ctx.set(&self.input_key, input);

        let mut orch_guard = self.orchestrator.lock().map_err(|_| anyhow!("Orchestrator mutex poisoned"))?;
        let mut steps_guard = self.steps.lock().map_err(|_| anyhow!("Steps mutex poisoned"))?;

        let mut step_slice = std::mem::take(&mut *steps_guard);
        let result = orch_guard.run(&mut ctx, &mut step_slice);
        *steps_guard = step_slice;

        result.map_err(|e| anyhow!("Pipeline execution failed: {}", e))?;

        ctx.get(&self.output_key)
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Output key '{}' not populated by pipeline", self.output_key))
    }
}