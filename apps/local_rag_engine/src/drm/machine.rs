//! # Hardware Machine Binding Module
//!
//! Generates a deterministic, tamper-resistant Machine UUID based on hardware
//! identifiers (CPU ID, Motherboard serial/UUID, and system machine ID).

use sha2::{Digest, Sha256};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct MachineFingerprint {
    pub machine_uuid: String,
    pub platform: String,
}

impl MachineFingerprint {
    /// Collects hardware metrics and produces a deterministic SHA-256 machine UUID.
    pub fn generate() -> Self {
        let platform = std::env::consts::OS.to_string();
        let raw_fingerprint = Self::collect_raw_hardware_data();

        let mut hasher = Sha256::new();
        hasher.update(raw_fingerprint.as_bytes());
        let hash_result = hasher.finalize();
        let machine_uuid = hex::encode(hash_result);

        Self {
            machine_uuid,
            platform,
        }
    }

    /// Verifies if a given UUID matches the local physical hardware.
    pub fn matches(&self, expected_uuid: &str) -> bool {
        self.machine_uuid.eq_ignore_ascii_case(expected_uuid.trim())
    }

    #[cfg(target_os = "windows")]
    fn collect_raw_hardware_data() -> String {
        let mut components = Vec::new();

        // 1. Motherboard Serial / UUID via WMIC or PowerShell
        let board = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-CimInstance Win32_BaseBoard | Select-Object -ExpandProperty SerialNumber"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if !board.is_empty() {
            components.push(format!("MB:{}", board));
        }

        // 2. BIOS / ComputerSystemProduct UUID
        let bios_uuid = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-CimInstance Win32_ComputerSystemProduct | Select-Object -ExpandProperty UUID"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if !bios_uuid.is_empty() {
            components.push(format!("BIOS:{}", bios_uuid));
        }

        // 3. Processor ID
        let cpu_id = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty ProcessorId"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if !cpu_id.is_empty() {
            components.push(format!("CPU:{}", cpu_id));
        }

        // Fallback if PowerShell commands failed in restricted environments
        if components.is_empty() {
            let computer_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "DEFAULT_WIN_HOST".to_string());
            let username = std::env::var("USERNAME").unwrap_or_else(|_| "DEFAULT_WIN_USER".to_string());
            components.push(format!("FALLBACK:{}:{}", computer_name, username));
        }

        components.join("|")
    }

    #[cfg(not(target_os = "windows"))]
    fn collect_raw_hardware_data() -> String {
        let mut components = Vec::new();

        // 1. Linux /etc/machine-id
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                components.push(format!("MACHINE_ID:{}", trimmed));
            }
        }

        // 2. DMI Product UUID
        if let Ok(id) = std::fs::read_to_string("/sys/class/dmi/id/product_uuid") {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                components.push(format!("DMI_UUID:{}", trimmed));
            }
        }

        if components.is_empty() {
            let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "UNIX_HOST".to_string());
            components.push(format!("FALLBACK:{}", hostname));
        }

        components.join("|")
    }
}
