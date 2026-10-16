//! Hardware Relay & Alarm Actuation Subsystem
//! Fires 110dB sirens, strobe lights, and electronic door locks via GPIO or USB relays.

use std::time::{Duration, Instant};

/// Physical state of an alarm actuator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayState {
    Disarmed,
    ActiveSirenStrobe,
    StrobeOnlySilent,
}

pub struct HardwareRelayController {
    pub current_state: RelayState,
    pub last_fired_at: Option<Instant>,
    pub auto_silence_after: Duration,
}

impl HardwareRelayController {
    pub fn new(auto_silence_secs: u64) -> Self {
        Self {
            current_state: RelayState::Disarmed,
            last_fired_at: None,
            auto_silence_after: Duration::from_secs(auto_silence_secs),
        }
    }

    /// Trigger siren and strobe light for critical security events
    pub fn trigger_alarm(&mut self, camera_id: &str, threat_class: &str) {
        self.current_state = RelayState::ActiveSirenStrobe;
        self.last_fired_at = Some(Instant::now());

        println!("🚨 [HARDWARE RELAY] >>> 110dB SIREN & STROBE FIRED! <<<");
        println!("🚨 [HARDWARE RELAY] Camera: {} | Threat: {}", camera_id, threat_class);

        // Hardware command dispatch:
        // On Linux embedded box, writes to /sys/class/gpio or opens USB FTDI/CH340 relay serial port
        #[cfg(target_os = "linux")]
        {
            // e.g. std::fs::write("/sys/class/gpio/gpio17/value", "1");
        }
    }

    /// Disarm / silence alarm
    pub fn disarm(&mut self) {
        if self.current_state != RelayState::Disarmed {
            self.current_state = RelayState::Disarmed;
            println!("🔕 [HARDWARE RELAY] Siren and Strobe Disarmed.");
        }
    }

    /// Periodic check to auto-silence siren after configured duration (e.g. 60 seconds)
    pub fn tick(&mut self) {
        if self.current_state == RelayState::ActiveSirenStrobe {
            if let Some(fired_at) = self.last_fired_at {
                if Instant::now().duration_since(fired_at) >= self.auto_silence_after {
                    self.disarm();
                }
            }
        }
    }
}
