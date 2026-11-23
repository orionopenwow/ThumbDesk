use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControllerFamily {
    Xbox,
    PlayStation,
    Generic,
}

impl ControllerFamily {
    pub fn inferred(vendor_name: Option<&str>) -> Self {
        let lower = vendor_name.unwrap_or_default().to_ascii_lowercase();
        if lower.contains("dualsense")
            || lower.contains("dualshock")
            || lower.contains("playstation")
            || lower.contains("sony")
        {
            Self::PlayStation
        } else if lower.contains("xbox") || lower.contains("microsoft") || lower.contains("xinput") {
            Self::Xbox
        } else {
            Self::Generic
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Xbox => "Xbox",
            Self::PlayStation => "PlayStation",
            Self::Generic => "Game Controller",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StickSide {
    Left,
    Right,
}

impl StickSide {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Left => "Left Stick",
            Self::Right => "Right Stick",
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StickAssignment {
    Off,
    Left,
    Right,
}

impl StickAssignment {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Left => "Left Stick",
            Self::Right => "Right Stick",
        }
    }

    pub fn stick_side(self) -> Option<StickSide> {
        match self {
            Self::Off => None,
            Self::Left => Some(StickSide::Left),
            Self::Right => Some(StickSide::Right),
        }
    }

    pub fn from_side(side: StickSide) -> Self {
        match side {
            StickSide::Left => Self::Left,
            StickSide::Right => Self::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StickRoleChoice {
    Primary,
    Precision,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControllerControlID {
    ButtonSouth,
    ButtonEast,
    ButtonWest,
    ButtonNorth,
    LeftShoulder,
    RightShoulder,
    LeftTrigger,
    RightTrigger,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    LeftThumbstick,
    RightThumbstick,
    LeftThumbstickButton,
    RightThumbstickButton,
    Menu,
    Options,
    Home,
    Share,
    TouchpadButton,
}

impl ControllerControlID {
    pub const ALL: &'static [Self] = &[
        Self::ButtonSouth,
        Self::ButtonEast,
        Self::ButtonWest,
        Self::ButtonNorth,
        Self::LeftShoulder,
        Self::RightShoulder,
        Self::LeftTrigger,
        Self::RightTrigger,
        Self::DpadUp,
        Self::DpadDown,
        Self::DpadLeft,
        Self::DpadRight,
        Self::LeftThumbstick,
        Self::RightThumbstick,
        Self::LeftThumbstickButton,
        Self::RightThumbstickButton,
        Self::Menu,
        Self::Options,
        Self::Home,
        Self::Share,
        Self::TouchpadButton,
    ];

    pub const MAPPING_CONTROLS: &'static [Self] = &[
        Self::ButtonSouth,
        Self::ButtonEast,
        Self::ButtonWest,
        Self::ButtonNorth,
        Self::LeftShoulder,
        Self::RightShoulder,
        Self::LeftTrigger,
        Self::RightTrigger,
        Self::DpadUp,
        Self::DpadDown,
        Self::DpadLeft,
        Self::DpadRight,
        Self::LeftThumbstickButton,
        Self::RightThumbstickButton,
        Self::Menu,
        Self::Options,
        Self::Home,
        Self::Share,
        Self::TouchpadButton,
    ];

    pub fn is_stick_role_control(self) -> bool {
        matches!(self, Self::LeftThumbstick | Self::RightThumbstick)
    }

    pub fn is_mapping_eligible(self) -> bool {
        !self.is_stick_role_control()
    }

    pub fn mapping_controls_for(family: ControllerFamily) -> Vec<Self> {
        Self::MAPPING_CONTROLS
            .iter()
            .copied()
            .filter(|c| match family {
                ControllerFamily::PlayStation => *c != Self::Share,
                ControllerFamily::Xbox | ControllerFamily::Generic => *c != Self::TouchpadButton,
            })
            .collect()
    }

    pub fn display_name(self, family: ControllerFamily) -> &'static str {
        if family == ControllerFamily::PlayStation {
            return match self {
                Self::ButtonSouth => "Cross",
                Self::ButtonEast => "Circle",
                Self::ButtonWest => "Square",
                Self::ButtonNorth => "Triangle",
                Self::LeftShoulder => "L1",
                Self::RightShoulder => "R1",
                Self::LeftTrigger => "L2",
                Self::RightTrigger => "R2",
                Self::LeftThumbstickButton => "L3",
                Self::RightThumbstickButton => "R3",
                Self::Menu => "Options",
                Self::Options => "Create",
                Self::Home => "PS",
                _ => self.default_display_name(),
            };
        }
        self.default_display_name()
    }

    pub fn default_display_name(self) -> &'static str {
        match self {
            Self::ButtonSouth => "A",
            Self::ButtonEast => "B",
            Self::ButtonWest => "X",
            Self::ButtonNorth => "Y",
            Self::LeftShoulder => "LB",
            Self::RightShoulder => "RB",
            Self::LeftTrigger => "LT",
            Self::RightTrigger => "RT",
            Self::DpadUp => "D-Pad Up",
            Self::DpadDown => "D-Pad Down",
            Self::DpadLeft => "D-Pad Left",
            Self::DpadRight => "D-Pad Right",
            Self::LeftThumbstick => "Left Stick",
            Self::RightThumbstick => "Right Stick",
            Self::LeftThumbstickButton => "Left Stick Click",
            Self::RightThumbstickButton => "Right Stick Click",
            Self::Menu => "Menu",
            Self::Options => "View",
            Self::Home => "Home",
            Self::Share => "Share",
            Self::TouchpadButton => "Touchpad",
        }
    }

    pub fn from_json_key(key: &str) -> Option<Self> {
        serde_json::from_value(serde_json::Value::String(key.to_string())).ok()
    }

    pub fn to_json_key(self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StickSnapshot {
    pub x: f64,
    pub y: f64,
    pub pressed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerSnapshot {
    pub is_connected: bool,
    pub name: Option<String>,
    pub family: ControllerFamily,
    pub transport: Option<String>,
    pub battery_level: Option<f32>,
    pub left_stick: StickSnapshot,
    pub right_stick: StickSnapshot,
    pub pressed_controls: BTreeSet<ControllerControlID>,
    pub analog_values: BTreeMap<ControllerControlID, f64>,
    pub event_count: u64,
    pub share_supported: bool,
    #[serde(default)]
    pub has_gyro: bool,
    #[serde(default)]
    pub gyro: [f32; 3],
    #[serde(default)]
    pub touchpad_touched: bool,
    #[serde(default)]
    pub touchpad_pos: [f32; 2],
    #[serde(default)]
    pub polling_rate_hz: u32,
}

impl Default for ControllerSnapshot {
    fn default() -> Self {
        Self {
            is_connected: false,
            name: None,
            family: ControllerFamily::Xbox,
            transport: None,
            battery_level: None,
            left_stick: StickSnapshot::default(),
            right_stick: StickSnapshot::default(),
            pressed_controls: BTreeSet::new(),
            analog_values: BTreeMap::new(),
            event_count: 0,
            share_supported: true,
            has_gyro: false,
            gyro: [0.0, 0.0, 0.0],
            touchpad_touched: false,
            touchpad_pos: [0.5, 0.5],
            polling_rate_hz: 120,
        }
    }
}

impl ControllerSnapshot {
    pub fn value_for(&self, control: ControllerControlID) -> f64 {
        if let Some(&v) = self.analog_values.get(&control) {
            return v.clamp(0.0, 1.0);
        }
        match control {
            ControllerControlID::LeftThumbstickButton => {
                if self.left_stick.pressed || self.pressed_controls.contains(&control) {
                    1.0
                } else {
                    0.0
                }
            }
            ControllerControlID::RightThumbstickButton => {
                if self.right_stick.pressed || self.pressed_controls.contains(&control) {
                    1.0
                } else {
                    0.0
                }
            }
            _ => {
                if self.pressed_controls.contains(&control) {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    pub fn stick(&self, side: StickSide) -> StickSnapshot {
        match side {
            StickSide::Left => self.left_stick,
            StickSide::Right => self.right_stick,
        }
    }

    pub fn has_same_input_payload(&self, other: &Self) -> bool {
        self.is_connected == other.is_connected
            && self.family == other.family
            && self.name == other.name
            && self.left_stick == other.left_stick
            && self.right_stick == other.right_stick
            && self.pressed_controls == other.pressed_controls
            && self.analog_values == other.analog_values
    }
}
