use anyhow::{anyhow, Result};
use ort::session::Session;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use crate::config::AppConfig;

/// मॉडल निष्पादन (Execution) लक्ष्य
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionTarget {
    Auto,       // 1. CUDA (Linux/Win NVIDIA) -> 2. DirectML (Windows GPU) -> 3. CPU
    Gpu,        // GPU auto: CUDA first, then DirectML
    Cuda,       // Explicit NVIDIA CUDA
    DirectML,   // Explicit DirectML (Windows)
    Cpu,        // Explicit CPU (Multi-threaded)
}

impl ExecutionTarget {
    pub fn from_str_target(target: &str) -> Self {
        match target.to_lowercase().trim() {
            "cuda" | "nvidia" => ExecutionTarget::Cuda,
            "directml" | "dml" => ExecutionTarget::DirectML,
            "gpu" => ExecutionTarget::Gpu,
            "cpu" => ExecutionTarget::Cpu,
            _ => ExecutionTarget::Auto,
        }
    }
}

impl Default for ExecutionTarget {
    fn default() -> Self {
        ExecutionTarget::Auto
    }
}

/// मॉडल लोड होने के बाद सक्रिय हार्डवेयर
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveHardware {
    GpuCuda,
    GpuDirectML,
    Cpu,
}

impl std::fmt::Display for ActiveHardware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActiveHardware::GpuCuda => write!(f, "⚡ Active Hardware: NVIDIA CUDA (High-Performance GPU)"),
            ActiveHardware::GpuDirectML => write!(f, "⚡ Active Hardware: DirectML (Direct3D 12 GPU)"),
            ActiveHardware::Cpu => write!(f, "💻 Active Hardware: CPU (Multi-threaded)"),
        }
    }
}

/// सेलेक्टिव / लेज़ी मॉडल लोडर और सेशन मैनेजर (Lazy Session Pool)
#[derive(Clone)]
pub struct MLBridge {
    pub config: Arc<AppConfig>,
    sessions: Arc<RwLock<HashMap<String, Arc<Mutex<Session>>>>>,
}

impl MLBridge {
    /// नए MLBridge इंस्टैंस का निर्माण (Zero Eager Allocations)
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// कॉन्फिग रेफरेंस से आसानी से इनिशियलाइज करने के लिए
    pub fn from_config(config: AppConfig) -> Self {
        Self::new(Arc::new(config))
    }

    /// किसी मॉडल के सेशन को ऑन-डिमांड / लेज़ी लोड करता है
    pub fn get_or_load_session(&self, model_key: &str) -> Result<Arc<Mutex<Session>>> {
        // Fast path: Read lock
        {
            let guard = self.sessions.read().map_err(|_| anyhow!("RwLock poisoned"))?;
            if let Some(session) = guard.get(model_key) {
                return Ok(Arc::clone(session));
            }
        }

        // Slow path: Write lock
        let mut guard = self.sessions.write().map_err(|_| anyhow!("RwLock poisoned"))?;
        if let Some(session) = guard.get(model_key) {
            return Ok(Arc::clone(session));
        }

        let full_path = self.config.resolve_path(model_key)?;
        println!("🔄 [Lazy Loading] मॉडल लोड किया जा रहा है: '{}' (Path: {:?})", model_key, full_path);

        let (session, active_hw) = Self::create_session_with_target(
            &full_path,
            ExecutionTarget::from_str_target(&self.config.execution_device),
        )?;

        println!("✅ [Loaded] '{}' तैयार है -> {}", model_key, active_hw);
        let arc_session = Arc::new(Mutex::new(session));
        guard.insert(model_key.to_string(), Arc::clone(&arc_session));

        Ok(arc_session)
    }

    /// मॉडल पाथ से सीधे सेशन लोड और कैश करना
    pub fn get_or_load_session_from_path<P: AsRef<Path>>(&self, path: P, device: &str) -> Result<Arc<Mutex<Session>>> {
        let path_ref = path.as_ref();
        let key = path_ref.to_string_lossy().to_string();

        {
            let guard = self.sessions.read().map_err(|_| anyhow!("RwLock poisoned"))?;
            if let Some(session) = guard.get(&key) {
                return Ok(Arc::clone(session));
            }
        }

        let mut guard = self.sessions.write().map_err(|_| anyhow!("RwLock poisoned"))?;
        if let Some(session) = guard.get(&key) {
            return Ok(Arc::clone(session));
        }

        let (session, _) = Self::create_session_with_target(
            path_ref,
            ExecutionTarget::from_str_target(device),
        )?;

        let arc_session = Arc::new(Mutex::new(session));
        guard.insert(key, Arc::clone(&arc_session));
        Ok(arc_session)
    }

    /// चुनिंदा मॉडल्स को पहले से मेमोरी में प्री-वार्म / प्री-लोड करना
    pub fn preload(&self, keys: &[&str]) -> Result<()> {
        for &key in keys {
            self.get_or_load_session(key)?;
        }
        Ok(())
    }

    /// VRAM/RAM खाली करने के लिए मॉडल अनलोड करना
    pub fn unload(&self, model_key: &str) -> Result<bool> {
        let mut guard = self.sessions.write().map_err(|_| anyhow!("RwLock poisoned"))?;
        let removed = guard.remove(model_key).is_some();
        if removed {
            println!("🧹 [Unloaded] मॉडल '{}' को मेमोरी से हटा दिया गया है।", model_key);
        }
        Ok(removed)
    }

    /// जांचें कि क्या मॉडल पहले से लोड है
    pub fn is_loaded(&self, model_key: &str) -> bool {
        if let Ok(guard) = self.sessions.read() {
            guard.contains_key(model_key)
        } else {
            false
        }
    }

    /// किसी भी ONNX मॉडल पाथ से डिफ़ॉल्ट "auto" मोड के साथ सेशन बनाने का कोर फंक्शन
    pub fn create_session<P: AsRef<Path>>(model_path: P) -> Result<Session> {
        Self::create_session_with_device(model_path, "auto")
    }

    /// डिवाइस स्ट्रिंग ("auto", "cuda", "directml", "gpu", "cpu") के आधार पर सेफ़ लोडर
    pub fn create_session_with_device<P: AsRef<Path>>(model_path: P, device: &str) -> Result<Session> {
        let target = ExecutionTarget::from_str_target(device);
        let (session, _) = Self::create_session_with_target(model_path, target)?;
        Ok(session)
    }

    /// Multi-Provider Safe Loader: लक्ष्य (Auto, Cuda, DirectML, Gpu, Cpu) के अनुसार डायरेक्ट सेशन और एक्टिव हार्डवेयर तैयार करता है
    pub fn create_session_with_target<P: AsRef<Path>>(
        model_path: P,
        target: ExecutionTarget,
    ) -> Result<(Session, ActiveHardware)> {
        let path = model_path.as_ref();
        if !path.exists() {
            return Err(anyhow!("Model file does not exist at path: {:?}", path));
        }

        match target {
            ExecutionTarget::Cpu => {
                let session = Self::try_create_cpu(path)?;
                Ok((session, ActiveHardware::Cpu))
            }
            ExecutionTarget::Cuda => {
                let cuda_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Self::try_create_cuda(path)
                }));

                match cuda_res {
                    Ok(Ok(session)) => Ok((session, ActiveHardware::GpuCuda)),
                    Ok(Err(e)) => Err(anyhow!("CUDA initialization failed: {}", e)),
                    Err(_) => Err(anyhow!("CUDA crashed at native C++ boundary")),
                }
            }
            ExecutionTarget::DirectML => {
                let directml_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Self::try_create_directml(path)
                }));

                match directml_res {
                    Ok(Ok(session)) => Ok((session, ActiveHardware::GpuDirectML)),
                    Ok(Err(e)) => Err(anyhow!("DirectML initialization failed: {}", e)),
                    Err(_) => Err(anyhow!("DirectML crashed at native C++ boundary")),
                }
            }
            ExecutionTarget::Gpu => {
                let cuda_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Self::try_create_cuda(path)
                }));
                if let Ok(Ok(session)) = cuda_res {
                    return Ok((session, ActiveHardware::GpuCuda));
                }

                let directml_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Self::try_create_directml(path)
                }));
                if let Ok(Ok(session)) = directml_res {
                    return Ok((session, ActiveHardware::GpuDirectML));
                }

                Err(anyhow!("GPU initialization failed: Neither CUDA nor DirectML was available on this system."))
            }
            ExecutionTarget::Auto => {
                // 1. CUDA
                let cuda_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Self::try_create_cuda(path)
                }));
                if let Ok(Ok(session)) = cuda_res {
                    return Ok((session, ActiveHardware::GpuCuda));
                }

                // 2. DirectML
                let directml_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Self::try_create_directml(path)
                }));
                if let Ok(Ok(session)) = directml_res {
                    return Ok((session, ActiveHardware::GpuDirectML));
                }

                // 3. CPU Fallback
                let session = Self::try_create_cpu(path)?;
                Ok((session, ActiveHardware::Cpu))
            }
        }
    }

    fn try_create_cuda(path: &Path) -> Result<Session> {
        let mut builder = Session::builder()
            .map_err(|e| anyhow!("Session builder error: {}", e))?
            .with_execution_providers([
                ort::ep::CUDA::default().into(),
            ])
            .map_err(|e| anyhow!("Failed to set CUDA execution provider: {}", e))?;

        let session = builder
            .commit_from_file(path)
            .map_err(|e| anyhow!("Failed to commit model with CUDA: {}", e))?;

        Ok(session)
    }

    #[cfg(windows)]
    fn try_create_directml(path: &Path) -> Result<Session> {
        let mut builder = Session::builder()
            .map_err(|e| anyhow!("Session builder error: {}", e))?
            .with_execution_providers([
                ort::ep::DirectML::default().into(),
            ])
            .map_err(|e| anyhow!("Failed to set DirectML execution provider: {}", e))?;

        let session = builder
            .commit_from_file(path)
            .map_err(|e| anyhow!("Failed to commit model with DirectML: {}", e))?;

        Ok(session)
    }

    #[cfg(not(windows))]
    fn try_create_directml(_path: &Path) -> Result<Session> {
        Err(anyhow!("DirectML is only supported on Windows"))
    }

    fn try_create_cpu(path: &Path) -> Result<Session> {
        let mut builder = Session::builder()
            .map_err(|e| anyhow!("Session builder error: {}", e))?;

        let session = builder
            .commit_from_file(path)
            .map_err(|e| anyhow!("Failed to commit model with CPU provider: {}", e))?;

        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_bridge_instantiation() {
        let config = Arc::new(AppConfig::default());
        let bridge = MLBridge::new(config);
        assert!(!bridge.is_loaded("silero_vad"));
    }
}