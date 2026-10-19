//! # Hardware Fingerprinting for Tier 3 DRM Machine-Binding
//!
//! Computes unique SHA-256 machine hash from CPU ID and Motherboard Serial.

use sha2::{Digest, Sha256};
use std::process::Command;

/// Retrieves a deterministic, machine-bound hardware fingerprint
pub fn get_machine_fingerprint() -> String {
    let cpu_info = get_cpu_identifier();
    let mb_info = get_motherboard_serial();

    let mut hasher = Sha256::new();
    hasher.update(cpu_info.as_bytes());
    hasher.update(b"::");
    hasher.update(mb_info.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(target_os = "windows")]
fn get_cpu_identifier() -> String {
    let output = Command::new("wmic")
        .args(&["cpu", "get", "processorid"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_default();

    output
        .lines()
        .nth(1)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "CPU_WIN_DEFAULT_NODE".to_string())
}

#[cfg(not(target_os = "windows"))]
fn get_cpu_identifier() -> String {
    std::fs::read_to_string("/etc/machine-id")
        .or_else(|_| std::fs::read_to_string("/var/lib/dbus/machine-id"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "CPU_LINUX_DEFAULT_NODE".to_string())
}

#[cfg(target_os = "windows")]
fn get_motherboard_serial() -> String {
    let output = Command::new("wmic")
        .args(&["baseboard", "get", "serialnumber"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_default();

    output
        .lines()
        .nth(1)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "MB_WIN_DEFAULT_SERIAL".to_string())
}

#[cfg(not(target_os = "windows"))]
fn get_motherboard_serial() -> String {
    std::fs::read_to_string("/sys/class/dmi/id/product_uuid")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "MB_LINUX_DEFAULT_SERIAL".to_string())
}
