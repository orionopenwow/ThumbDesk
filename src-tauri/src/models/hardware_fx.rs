use serde::{Deserialize, Serialize};

/// DualSense adaptive trigger mechanical resistance preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum AdaptiveTriggerMode {
    #[default]
    Off,
    SoftClick,
    MediumDetent,
    RigidWall,
    DualStage,
}

/// DualSense / DualShock 4 lightbar behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum LightbarMode {
    #[default]
    AutoTelemetry,
    CustomColor,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareFxPreferences {
    #[serde(default)]
    pub left_trigger_mode: AdaptiveTriggerMode,
    #[serde(default)]
    pub right_trigger_mode: AdaptiveTriggerMode,
    #[serde(default)]
    pub lightbar_mode: LightbarMode,
    #[serde(default = "default_custom_rgb")]
    pub custom_rgb: [u8; 3],
}

fn default_custom_rgb() -> [u8; 3] {
    [56, 189, 248] // ThumbDesk cyan
}

impl Default for HardwareFxPreferences {
    fn default() -> Self {
        Self {
            left_trigger_mode: AdaptiveTriggerMode::SoftClick,
            right_trigger_mode: AdaptiveTriggerMode::Off,
            lightbar_mode: LightbarMode::AutoTelemetry,
            custom_rgb: default_custom_rgb(),
        }
    }
}

/// Computes the effective RGB lightbar color from live engine state.
pub fn resolve_lightbar_rgb(
    prefs: HardwareFxPreferences,
    guard_suspended: bool,
    voice_listening: bool,
    modifier_held: bool,
    battery_level: Option<f32>,
    runtime_enabled: bool,
) -> [u8; 3] {
    match prefs.lightbar_mode {
        LightbarMode::Off => [0, 0, 0],
        LightbarMode::CustomColor => prefs.custom_rgb,
        LightbarMode::AutoTelemetry => {
            if guard_suspended {
                [245, 158, 11] // Amber: Anti-Cheat / Steam Guard paused
            } else if !runtime_enabled {
                [100, 116, 139] // Muted slate: Paused
            } else if battery_level.map(|b| b < 0.15).unwrap_or(false) {
                [239, 68, 68] // Red: Low battery
            } else if voice_listening {
                [52, 211, 153] // Emerald: Voice Mode active
            } else if modifier_held {
                [168, 85, 247] // Purple: Modifier layer held
            } else {
                [56, 189, 248] // Cyan: Active 120 Hz desktop mode
            }
        }
    }
}

/// Encodes an 11-byte DualSense adaptive trigger effect block (for report 0x02).
pub fn encode_dualsense_trigger_effect(mode: AdaptiveTriggerMode) -> [u8; 11] {
    let mut b = [0u8; 11];
    match mode {
        AdaptiveTriggerMode::Off => {
            b[0] = 0x05; // Off / Continuous release
        }
        AdaptiveTriggerMode::SoftClick => {
            // Section resistance with crisp click feel around 45% travel
            b[0] = 0x02;
            b[1] = 0x40;
            b[2] = 0x90;
            b[3] = 0x80;
        }
        AdaptiveTriggerMode::MediumDetent => {
            // Continuous resistance starting at 30%
            b[0] = 0x01;
            b[1] = 0x35;
            b[2] = 0xA8;
        }
        AdaptiveTriggerMode::RigidWall => {
            // Firm wall at 55% travel
            b[0] = 0x01;
            b[1] = 0x70;
            b[2] = 0xFF;
        }
        AdaptiveTriggerMode::DualStage => {
            // Two-stage trigger wall (hover -> click)
            b[0] = 0x02;
            b[1] = 0x55;
            b[2] = 0xB8;
            b[3] = 0xE8;
        }
    }
    b
}
