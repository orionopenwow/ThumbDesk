use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VibrationPhase {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VibrationLocality {
    Handles,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VibrationPulse {
    pub time: f64,
    pub duration: f64,
    pub intensity: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControllerVibration {
    #[default]
    None,
    SoftTap,
    DoubleTap,
    Thump,
    LeftPulse,
    RightPulse,
    GrabRelease,
}

impl ControllerVibration {
    pub const ALL: &'static [Self] = &[
        Self::None,
        Self::SoftTap,
        Self::DoubleTap,
        Self::Thump,
        Self::LeftPulse,
        Self::RightPulse,
        Self::GrabRelease,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::SoftTap => "Soft tap",
            Self::DoubleTap => "Double tap",
            Self::Thump => "Thump",
            Self::LeftPulse => "Left pulse",
            Self::RightPulse => "Right pulse",
            Self::GrabRelease => "Grab & release",
        }
    }

    pub fn locality(self) -> VibrationLocality {
        match self {
            Self::LeftPulse => VibrationLocality::Left,
            Self::RightPulse => VibrationLocality::Right,
            _ => VibrationLocality::Handles,
        }
    }

    pub fn pulses(self, phase: VibrationPhase) -> &'static [VibrationPulse] {
        match (self, phase) {
            (Self::None, _) => &[],
            (Self::SoftTap, VibrationPhase::Press) => &[VibrationPulse {
                time: 0.0,
                duration: 0.075,
                intensity: 0.55,
            }],
            (Self::DoubleTap, VibrationPhase::Press) => &[
                VibrationPulse {
                    time: 0.0,
                    duration: 0.065,
                    intensity: 0.60,
                },
                VibrationPulse {
                    time: 0.14,
                    duration: 0.065,
                    intensity: 0.50,
                },
            ],
            (Self::Thump, VibrationPhase::Press)
            | (Self::LeftPulse, VibrationPhase::Press)
            | (Self::RightPulse, VibrationPhase::Press) => &[VibrationPulse {
                time: 0.0,
                duration: 0.120,
                intensity: 0.85,
            }],
            (Self::GrabRelease, VibrationPhase::Press) => &[VibrationPulse {
                time: 0.0,
                duration: 0.075,
                intensity: 0.55,
            }],
            (Self::GrabRelease, VibrationPhase::Release) => &[VibrationPulse {
                time: 0.0,
                duration: 0.055,
                intensity: 0.40,
            }],
            (_, VibrationPhase::Release) => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HapticsPreferences {
    pub enabled: bool,
    pub strength: f32,
    pub connection_feedback: bool,
}

impl Default for HapticsPreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            strength: 0.5,
            connection_feedback: true,
        }
    }
}

impl HapticsPreferences {
    pub fn clamped_strength(&self) -> f32 {
        if !self.strength.is_finite() {
            0.5
        } else {
            self.strength.clamp(0.1, 1.0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HapticsStatus {
    pub can_play: bool,
    pub backend: String,
    pub message: String,
}

impl Default for HapticsStatus {
    fn default() -> Self {
        Self {
            can_play: false,
            backend: "none".to_string(),
            message: "Connect a controller to test vibration.".to_string(),
        }
    }
}
