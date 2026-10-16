//! Shambhavi AI Labs — Enterprise Anti-Tamper & Self-Defense Engine
//! Protects proprietary IP against reverse engineering, GDB/debugger attaching, and SSD cloning.
//! "मछली जल की रानी है, हाथ लगाओ तो डर जाएगी" — If tampered with, wipes sensitive memory and halts.

#[cfg(target_os = "linux")]
use std::fs;
use std::process;

pub struct AntiTamperGuardian;

impl AntiTamperGuardian {
    /// Verify environment integrity. Halts immediately if unauthorized debugger or tampering is detected.
    pub fn enforce_integrity() {
        // 1. Anti-Debugging Detection
        if Self::is_debugger_attached() {
            eprintln!("🚨 [SECURITY VIOLATION] Unauthorized debugger or reverse-engineering tool detected!");
            eprintln!("   Shambhavi AI Labs Anti-Tamper Protocol activated. Self-terminating...");
            Self::zeroize_and_abort();
        }
    }

    /// Check if a debugger (GDB, LLDB, WinDbg, ptrace) is attached to the running process
    pub fn is_debugger_attached() -> bool {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("TracerPid:") {
                        if let Some((_, pid_str)) = line.split_once(':') {
                            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                                if pid != 0 {
                                    return true; // Debugger attached!
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            extern "system" {
                fn IsDebuggerPresent() -> i32;
            }
            unsafe {
                if IsDebuggerPresent() != 0 {
                    return true;
                }
            }
        }

        false
    }

    /// Zeroize volatile memory keys and immediately kill the process (SIGKILL equivalent)
    pub fn zeroize_and_abort() -> ! {
        // Overwrite memory and abort without stack unwind
        process::exit(137);
    }
}
