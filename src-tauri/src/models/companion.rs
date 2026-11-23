use serde::{Deserialize, Serialize};

use super::shortcut::ShortcutDescriptor;

pub const COMPANION_PROTOCOL_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompanionMode {
    #[default]
    Off,
    Controller,
    Receiver,
}

impl CompanionMode {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Off => "Local Windows Input",
            Self::Controller => "Controller PC",
            Self::Receiver => "Receiver PC",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompanionEdge {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

impl CompanionEdge {
    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionPeer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", content = "detail", rename_all = "camelCase")]
pub enum CompanionConnectionState {
    Off,
    Browsing,
    Listening,
    Connecting(String),
    Connected(String),
    Error(String),
}

impl Default for CompanionConnectionState {
    fn default() -> Self {
        Self::Off
    }
}

impl CompanionConnectionState {
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected(_))
    }

    pub fn display_text(&self) -> String {
        match self {
            Self::Off => "Off".to_string(),
            Self::Browsing => "Browsing for receiver".to_string(),
            Self::Listening => "Listening for controller".to_string(),
            Self::Connecting(name) => format!("Connecting to {}", name),
            Self::Connected(name) => format!("Connected to {}", name),
            Self::Error(msg) => msg.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompanionMouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompanionMousePhase {
    Click,
    DoubleClick,
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompanionShortcutPhase {
    Tap,
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpaceSwitchDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompanionMessageType {
    Hello,
    HandoffStart,
    PointerDelta,
    Mouse,
    Scroll,
    Shortcut,
    SpaceSwitch,
    HandoffBack,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionMessage {
    #[serde(rename = "type")]
    pub message_type: CompanionMessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge: Option<CompanionEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_position: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dx: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dy: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button: Option<CompanionMouseButton>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<CompanionMousePhase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<ShortcutDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut_phase: Option<CompanionShortcutPhase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_switch_direction: Option<SpaceSwitchDirection>,
}

impl CompanionMessage {
    fn base(message_type: CompanionMessageType) -> Self {
        Self {
            message_type,
            name: None,
            protocol_version: None,
            build_version: None,
            edge: None,
            normalized_position: None,
            dx: None,
            dy: None,
            button: None,
            phase: None,
            vertical: None,
            horizontal: None,
            shortcut: None,
            shortcut_phase: None,
            space_switch_direction: None,
        }
    }

    pub fn hello(name: String, build_version: String) -> Self {
        let mut msg = Self::base(CompanionMessageType::Hello);
        msg.name = Some(name);
        msg.protocol_version = Some(COMPANION_PROTOCOL_VERSION);
        msg.build_version = Some(build_version);
        msg
    }

    pub fn handoff_start(edge: CompanionEdge, normalized_position: f64) -> Self {
        let mut msg = Self::base(CompanionMessageType::HandoffStart);
        msg.edge = Some(edge);
        msg.normalized_position = Some(normalized_position.clamp(0.0, 1.0));
        msg
    }

    pub fn pointer_delta(dx: f64, dy: f64) -> Self {
        let mut msg = Self::base(CompanionMessageType::PointerDelta);
        msg.dx = Some(dx);
        msg.dy = Some(dy);
        msg
    }

    pub fn mouse(button: CompanionMouseButton, phase: CompanionMousePhase) -> Self {
        let mut msg = Self::base(CompanionMessageType::Mouse);
        msg.button = Some(button);
        msg.phase = Some(phase);
        msg
    }

    pub fn scroll(vertical: i32, horizontal: i32) -> Self {
        let mut msg = Self::base(CompanionMessageType::Scroll);
        msg.vertical = Some(vertical);
        msg.horizontal = Some(horizontal);
        msg
    }

    pub fn shortcut(shortcut: ShortcutDescriptor, phase: CompanionShortcutPhase) -> Self {
        let mut msg = Self::base(CompanionMessageType::Shortcut);
        msg.shortcut = Some(shortcut);
        msg.shortcut_phase = Some(phase);
        msg
    }

    pub fn space_switch(direction: SpaceSwitchDirection) -> Self {
        let mut msg = Self::base(CompanionMessageType::SpaceSwitch);
        msg.space_switch_direction = Some(direction);
        msg
    }

    pub fn handoff_back() -> Self {
        Self::base(CompanionMessageType::HandoffBack)
    }
}
