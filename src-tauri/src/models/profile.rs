use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

use super::control::{ControllerControlID, StickAssignment};
use super::shortcut::{vk, KeyboardModifier, ShortcutDescriptor};
use super::vibration::ControllerVibration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossEdgeDirection {
    Left,
    Right,
    Up,
    Down,
}

impl CrossEdgeDirection {
    pub fn motion_vector(self) -> (f64, f64) {
        match self {
            Self::Left => (-1.0, 0.0),
            Self::Right => (1.0, 0.0),
            Self::Up => (0.0, -1.0),
            Self::Down => (0.0, 1.0),
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Up => "Up",
            Self::Down => "Down",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerMode {
    #[default]
    Tap,
    HoldWhilePressed,
    RepeatWhileHeld,
    Toggle,
}

impl TriggerMode {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Tap => "Tap",
            Self::HoldWhilePressed => "Hold while pressed",
            Self::RepeatWhileHeld => "Repeat while held",
            Self::Toggle => "Toggle",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionType {
    #[default]
    None,
    KeyboardShortcut,
    LeftClick,
    RightClick,
    MiddleClick,
    LeftMouseHold,
    DoubleClick,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    SwitchSpaceLeft,
    SwitchSpaceRight,
    CrossEdgeLeft,
    CrossEdgeRight,
    CrossEdgeUp,
    CrossEdgeDown,
    ToggleCursorSpeeds,
    RunCommand,
    ToggleVoiceMode,
    WindowSnapLeft,
    WindowSnapRight,
    WindowMaximize,
    WindowMinimize,
    WindowNextMonitor,
    MediaPlayPause,
    MediaNext,
    MediaPrev,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    OpenRadialMenu,
    OpenOnScreenKeyboard,
}

impl ActionType {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::KeyboardShortcut => "Keyboard Shortcut",
            Self::LeftClick => "Left Click",
            Self::RightClick => "Right Click",
            Self::MiddleClick => "Middle Click",
            Self::LeftMouseHold => "Left Mouse Hold (drag)",
            Self::DoubleClick => "Double Click",
            Self::ScrollUp => "Scroll Up",
            Self::ScrollDown => "Scroll Down",
            Self::ScrollLeft => "Scroll Left",
            Self::ScrollRight => "Scroll Right",
            Self::SwitchSpaceLeft => "Switch Desktop Left",
            Self::SwitchSpaceRight => "Switch Desktop Right",
            Self::CrossEdgeLeft => "Cross Edge Left",
            Self::CrossEdgeRight => "Cross Edge Right",
            Self::CrossEdgeUp => "Cross Edge Up",
            Self::CrossEdgeDown => "Cross Edge Down",
            Self::ToggleCursorSpeeds => "Toggle Cursor Speeds",
            Self::RunCommand => "Run Command / Script",
            Self::ToggleVoiceMode => "Toggle Voice Mode",
            Self::WindowSnapLeft => "Window Snap Left",
            Self::WindowSnapRight => "Window Snap Right",
            Self::WindowMaximize => "Window Maximize",
            Self::WindowMinimize => "Window Minimize",
            Self::WindowNextMonitor => "Move Window to Next Monitor",
            Self::MediaPlayPause => "Media Play / Pause",
            Self::MediaNext => "Media Next Track",
            Self::MediaPrev => "Media Previous Track",
            Self::VolumeUp => "Volume Up",
            Self::VolumeDown => "Volume Down",
            Self::VolumeMute => "Volume Mute",
            Self::OpenRadialMenu => "Open Radial Pie Menu",
            Self::OpenOnScreenKeyboard => "Open On-Screen Keyboard",
        }
    }

    pub fn default_trigger_mode(self) -> TriggerMode {
        match self {
            Self::LeftMouseHold => TriggerMode::HoldWhilePressed,
            Self::ScrollUp | Self::ScrollDown | Self::ScrollLeft | Self::ScrollRight => {
                TriggerMode::RepeatWhileHeld
            }
            _ => TriggerMode::Tap,
        }
    }

    pub fn supported_trigger_modes(self) -> &'static [TriggerMode] {
        match self {
            Self::None
            | Self::CrossEdgeLeft
            | Self::CrossEdgeRight
            | Self::CrossEdgeUp
            | Self::CrossEdgeDown
            | Self::ToggleCursorSpeeds
            | Self::RunCommand
            | Self::ToggleVoiceMode
            | Self::WindowSnapLeft
            | Self::WindowSnapRight
            | Self::WindowMaximize
            | Self::WindowMinimize
            | Self::WindowNextMonitor
            | Self::MediaPlayPause
            | Self::MediaNext
            | Self::MediaPrev
            | Self::VolumeMute
            | Self::OpenRadialMenu
            | Self::OpenOnScreenKeyboard => &[TriggerMode::Tap],
            Self::VolumeUp | Self::VolumeDown => &[TriggerMode::Tap, TriggerMode::RepeatWhileHeld],
            Self::KeyboardShortcut => &[
                TriggerMode::Tap,
                TriggerMode::HoldWhilePressed,
                TriggerMode::RepeatWhileHeld,
            ],
            Self::LeftMouseHold => &[TriggerMode::HoldWhilePressed, TriggerMode::Toggle],
            Self::LeftClick
            | Self::RightClick
            | Self::MiddleClick
            | Self::DoubleClick
            | Self::ScrollUp
            | Self::ScrollDown
            | Self::ScrollLeft
            | Self::ScrollRight
            | Self::SwitchSpaceLeft
            | Self::SwitchSpaceRight => &[TriggerMode::Tap, TriggerMode::RepeatWhileHeld],
        }
    }

    pub fn cross_edge_direction(self) -> Option<CrossEdgeDirection> {
        match self {
            Self::CrossEdgeLeft => Some(CrossEdgeDirection::Left),
            Self::CrossEdgeRight => Some(CrossEdgeDirection::Right),
            Self::CrossEdgeUp => Some(CrossEdgeDirection::Up),
            Self::CrossEdgeDown => Some(CrossEdgeDirection::Down),
            _ => None,
        }
    }

    pub fn scroll_step(self) -> Option<(i32, i32)> {
        match self {
            Self::ScrollUp => Some((-1, 0)),
            Self::ScrollDown => Some((1, 0)),
            Self::ScrollLeft => Some((0, -1)),
            Self::ScrollRight => Some((0, 1)),
            _ => None,
        }
    }
}

fn default_repeat_delay() -> f64 {
    0.35
}

fn default_repeat_interval() -> f64 {
    0.08
}

fn deserialize_vibration_lenient<'de, D>(deserializer: D) -> Result<ControllerVibration, D::Error>
where
    D: Deserializer<'de>,
{
    let val = serde_json::Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(val).unwrap_or(ControllerVibration::None))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerActionMapping {
    pub action_type: ActionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<ShortcutDescriptor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_script: Option<String>,
    #[serde(default)]
    pub trigger_mode: TriggerMode,
    #[serde(default = "default_repeat_delay")]
    pub repeat_delay: f64,
    #[serde(default = "default_repeat_interval")]
    pub repeat_interval: f64,
    #[serde(default, deserialize_with = "deserialize_vibration_lenient")]
    pub vibration: ControllerVibration,
}

impl Default for ControllerActionMapping {
    fn default() -> Self {
        Self {
            action_type: ActionType::None,
            shortcut: None,
            command_script: None,
            trigger_mode: TriggerMode::Tap,
            repeat_delay: default_repeat_delay(),
            repeat_interval: default_repeat_interval(),
            vibration: ControllerVibration::None,
        }
    }
}

impl ControllerActionMapping {
    pub fn new(action_type: ActionType) -> Self {
        Self {
            action_type,
            shortcut: None,
            command_script: None,
            trigger_mode: action_type.default_trigger_mode(),
            repeat_delay: default_repeat_delay(),
            repeat_interval: default_repeat_interval(),
            vibration: ControllerVibration::None,
        }
    }

    pub fn shortcut(shortcut: ShortcutDescriptor, trigger_mode: TriggerMode) -> Self {
        Self {
            action_type: ActionType::KeyboardShortcut,
            shortcut: Some(shortcut),
            command_script: None,
            trigger_mode,
            repeat_delay: default_repeat_delay(),
            repeat_interval: default_repeat_interval(),
            vibration: ControllerVibration::None,
        }
    }

    pub fn command(script: String) -> Self {
        Self {
            action_type: ActionType::RunCommand,
            shortcut: None,
            command_script: Some(script),
            trigger_mode: TriggerMode::Tap,
            repeat_delay: default_repeat_delay(),
            repeat_interval: default_repeat_interval(),
            vibration: ControllerVibration::None,
        }
    }

    pub fn with_vibration(mut self, vibration: ControllerVibration) -> Self {
        self.vibration = vibration;
        self
    }

    pub fn summary(&self) -> String {
        match self.action_type {
            ActionType::None => "No action".to_string(),
            ActionType::KeyboardShortcut => self
                .shortcut
                .as_ref()
                .map(|s| s.display_string())
                .unwrap_or_else(|| "Set shortcut".to_string()),
            ActionType::RunCommand => self
                .command_script
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!("Run: {}", s.trim()))
                .unwrap_or_else(|| "Set command".to_string()),
            other => other.display_name().to_string(),
        }
    }
}

fn serialize_control_map<S>(
    map: &BTreeMap<ControllerControlID, ControllerActionMapping>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string_map: BTreeMap<String, &ControllerActionMapping> = map
        .iter()
        .map(|(k, v)| (k.to_json_key(), v))
        .collect();
    string_map.serialize(serializer)
}

fn deserialize_control_map<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<ControllerControlID, ControllerActionMapping>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: BTreeMap<String, ControllerActionMapping> = BTreeMap::deserialize(deserializer)?;
    let mut out = BTreeMap::new();
    for (k, v) in raw {
        if let Some(id) = ControllerControlID::from_json_key(&k) {
            out.insert(id, v);
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerModifierLayer {
    pub modifier_control: ControllerControlID,
    #[serde(
        default,
        serialize_with = "serialize_control_map",
        deserialize_with = "deserialize_control_map"
    )]
    pub mappings: BTreeMap<ControllerControlID, ControllerActionMapping>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationMappingOverrides {
    pub bundle_identifier: String,
    pub display_name: String,
    #[serde(
        default,
        serialize_with = "serialize_control_map",
        deserialize_with = "deserialize_control_map"
    )]
    pub mappings: BTreeMap<ControllerControlID, ControllerActionMapping>,
    #[serde(default)]
    pub modifier_layers: Vec<ControllerModifierLayer>,
}

impl ApplicationMappingOverrides {
    pub const CODEX_BUNDLE_IDENTIFIER: &'static str = "com.openai.codex";

    pub fn modifier_layer(
        &self,
        control: ControllerControlID,
    ) -> Option<&ControllerModifierLayer> {
        self.modifier_layers
            .iter()
            .find(|l| l.modifier_control == control)
    }

    pub fn exact_override(
        &self,
        control: ControllerControlID,
        modifier_control: Option<ControllerControlID>,
    ) -> Option<&ControllerActionMapping> {
        if let Some(modifier) = modifier_control {
            self.modifier_layer(modifier)
                .and_then(|layer| layer.mappings.get(&control))
        } else {
            self.mappings.get(&control)
        }
    }

    pub fn override_count(&self) -> usize {
        self.mappings.len()
            + self
                .modifier_layers
                .iter()
                .map(|l| l.mappings.len())
                .sum::<usize>()
    }
}

fn default_true() -> bool {
    true
}

fn default_magic_snap_radius() -> f64 {
    76.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorConfiguration {
    pub primary_stick: StickAssignment,
    pub precision_stick: StickAssignment,
    pub primary_speed: f64,
    pub precision_speed: f64,
    pub dead_zone: f64,
    pub response_curve: f64,
    pub smoothing: f64,
    pub acceleration_enabled: bool,
    pub invert_primary_x: bool,
    pub invert_primary_y: bool,
    pub invert_precision_x: bool,
    pub invert_precision_y: bool,
    pub horizontal_speed_multiplier: f64,
    pub vertical_speed_multiplier: f64,
    #[serde(default = "default_true")]
    pub flick_boost_enabled: bool,
    #[serde(default)]
    pub zoom_gesture_enabled: bool,
    #[serde(default = "default_true")]
    pub magic_select_enabled: bool,
    #[serde(default = "default_magic_snap_radius")]
    pub magic_snap_radius: f64,
    #[serde(default = "default_true")]
    pub touchpad_mouse_enabled: bool,
}

impl Default for CursorConfiguration {
    fn default() -> Self {
        Self {
            primary_stick: StickAssignment::Left,
            precision_stick: StickAssignment::Right,
            primary_speed: 2227.5417018581084,
            precision_speed: 565.6513935810809,
            dead_zone: 0.12,
            response_curve: 1.8,
            smoothing: 0.5,
            acceleration_enabled: true,
            invert_primary_x: false,
            invert_primary_y: false,
            invert_precision_x: false,
            invert_precision_y: false,
            horizontal_speed_multiplier: 1.0,
            vertical_speed_multiplier: 1.0,
            flick_boost_enabled: false,
            zoom_gesture_enabled: false,
            magic_select_enabled: true,
            magic_snap_radius: default_magic_snap_radius(),
            touchpad_mouse_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadialMenuSlice {
    pub id: String,
    pub label: String,
    pub action_type: ActionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_script: Option<String>,
}

pub fn default_radial_menu() -> Vec<RadialMenuSlice> {
    vec![
        RadialMenuSlice {
            id: "s1".into(),
            label: "Terminal".into(),
            action_type: ActionType::RunCommand,
            command_script: Some("wt.exe".into()),
        },
        RadialMenuSlice {
            id: "s2".into(),
            label: "Snap Left".into(),
            action_type: ActionType::WindowSnapLeft,
            command_script: None,
        },
        RadialMenuSlice {
            id: "s3".into(),
            label: "Snap Right".into(),
            action_type: ActionType::WindowSnapRight,
            command_script: None,
        },
        RadialMenuSlice {
            id: "s4".into(),
            label: "Next Monitor".into(),
            action_type: ActionType::WindowNextMonitor,
            command_script: None,
        },
        RadialMenuSlice {
            id: "s5".into(),
            label: "Play / Pause".into(),
            action_type: ActionType::MediaPlayPause,
            command_script: None,
        },
        RadialMenuSlice {
            id: "s6".into(),
            label: "Mute Audio".into(),
            action_type: ActionType::VolumeMute,
            command_script: None,
        },
        RadialMenuSlice {
            id: "s7".into(),
            label: "Voice Mode".into(),
            action_type: ActionType::ToggleVoiceMode,
            command_script: None,
        },
        RadialMenuSlice {
            id: "s8".into(),
            label: "Maximize".into(),
            action_type: ActionType::WindowMaximize,
            command_script: None,
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerProfile {
    pub id: String,
    pub name: String,
    pub cursor: CursorConfiguration,
    #[serde(default)]
    pub gyro: crate::engines::gyro_engine::GyroConfiguration,
    #[serde(default = "default_radial_menu")]
    pub radial_menu: Vec<RadialMenuSlice>,
    #[serde(
        default,
        serialize_with = "serialize_control_map",
        deserialize_with = "deserialize_control_map"
    )]
    pub mappings: BTreeMap<ControllerControlID, ControllerActionMapping>,
    #[serde(default)]
    pub modifier_layers: Vec<ControllerModifierLayer>,
    #[serde(default)]
    pub application_mappings: Vec<ApplicationMappingOverrides>,
}

impl ControllerProfile {
    pub fn modifier_layer(
        &self,
        control: ControllerControlID,
    ) -> Option<&ControllerModifierLayer> {
        self.modifier_layers
            .iter()
            .find(|l| l.modifier_control == control)
    }

    pub fn application_mapping(
        &self,
        bundle_identifier: &str,
    ) -> Option<&ApplicationMappingOverrides> {
        let target = bundle_identifier.to_ascii_lowercase();
        self.application_mappings.iter().find(|app| {
            let id = app.bundle_identifier.to_ascii_lowercase();
            id == target
                || (id == ApplicationMappingOverrides::CODEX_BUNDLE_IDENTIFIER
                    && (target == "codex.exe" || target == "chatgpt.exe"))
        })
    }

    /// Exact precedence:
    /// 1. App override for (control, modifier_control)
    /// 2. Global modifier layer for (control, modifier_control)
    /// 3. Global base mapping for control
    /// 4. Default `.none` mapping
    pub fn effective_mapping(
        &self,
        control: ControllerControlID,
        modifier_control: Option<ControllerControlID>,
        application_bundle_identifier: Option<&str>,
    ) -> ControllerActionMapping {
        if let Some(app_id) = application_bundle_identifier {
            if let Some(app) = self.application_mapping(app_id) {
                if let Some(found) = app.exact_override(control, modifier_control) {
                    return found.clone();
                }
            }
        }
        if let Some(modifier) = modifier_control {
            if let Some(layer) = self.modifier_layer(modifier) {
                if let Some(found) = layer.mappings.get(&control) {
                    return found.clone();
                }
            }
        }
        self.mappings.get(&control).cloned().unwrap_or_default()
    }

    pub fn suggested_vibration(mapping: &ControllerActionMapping) -> ControllerVibration {
        match mapping.action_type {
            ActionType::LeftMouseHold => ControllerVibration::GrabRelease,
            ActionType::CrossEdgeLeft => ControllerVibration::LeftPulse,
            ActionType::CrossEdgeRight => ControllerVibration::RightPulse,
            ActionType::CrossEdgeUp | ActionType::CrossEdgeDown => ControllerVibration::Thump,
            ActionType::KeyboardShortcut => {
                if let Some(s) = &mapping.shortcut {
                    let mods = s.ordered_modifiers();
                    // Windows Voice Typing (Win+H) or macOS Fn (63)
                    if (s.key_code == vk::H && mods == [KeyboardModifier::Win])
                        || (s.key_code == 63 && mods.is_empty())
                    {
                        return ControllerVibration::SoftTap;
                    }
                    // Windows Snipping Tool (Win+Shift+S)
                    if s.key_code == vk::S
                        && mods == [KeyboardModifier::Shift, KeyboardModifier::Win]
                    {
                        return ControllerVibration::SoftTap;
                    }
                    // PowerToys Text Extractor OCR (Win+Shift+T)
                    if s.key_code == vk::T
                        && mods == [KeyboardModifier::Shift, KeyboardModifier::Win]
                    {
                        return ControllerVibration::DoubleTap;
                    }
                }
                ControllerVibration::None
            }
            _ => ControllerVibration::None,
        }
    }

    pub fn apply_suggested_vibrations(&mut self) {
        let modifiers: BTreeSet<ControllerControlID> = self
            .modifier_layers
            .iter()
            .map(|l| l.modifier_control)
            .collect();

        for (control, mapping) in &mut self.mappings {
            mapping.vibration = if modifiers.contains(control) {
                ControllerVibration::SoftTap
            } else {
                Self::suggested_vibration(mapping)
            };
        }
        for layer in &mut self.modifier_layers {
            for mapping in layer.mappings.values_mut() {
                mapping.vibration = Self::suggested_vibration(mapping);
            }
        }
        for app in &mut self.application_mappings {
            for (control, mapping) in &mut app.mappings {
                mapping.vibration = if modifiers.contains(control) {
                    ControllerVibration::SoftTap
                } else {
                    Self::suggested_vibration(mapping)
                };
            }
            for layer in &mut app.modifier_layers {
                for mapping in layer.mappings.values_mut() {
                    mapping.vibration = Self::suggested_vibration(mapping);
                }
            }
        }
    }

    pub fn codex_starter_application_mappings() -> ApplicationMappingOverrides {
        let mut base = BTreeMap::new();
        base.insert(
            ControllerControlID::Menu,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::N, vec![KeyboardModifier::Control]),
                TriggerMode::Tap,
            ),
        );

        let make_layer = |modifier_control: ControllerControlID| {
            let mut m = BTreeMap::new();
            m.insert(
                ControllerControlID::Menu,
                ControllerActionMapping::shortcut(
                    ShortcutDescriptor::new(
                        vk::A,
                        vec![KeyboardModifier::Control, KeyboardModifier::Alt],
                    ),
                    TriggerMode::Tap,
                ),
            );
            m.insert(
                ControllerControlID::Options,
                ControllerActionMapping::shortcut(
                    ShortcutDescriptor::new(
                        vk::F,
                        vec![
                            KeyboardModifier::Control,
                            KeyboardModifier::Alt,
                            KeyboardModifier::Shift,
                        ],
                    ),
                    TriggerMode::Tap,
                ),
            );
            ControllerModifierLayer {
                modifier_control,
                mappings: m,
            }
        };

        ApplicationMappingOverrides {
            bundle_identifier: ApplicationMappingOverrides::CODEX_BUNDLE_IDENTIFIER.to_string(),
            display_name: "Codex / ChatGPT".to_string(),
            mappings: base,
            modifier_layers: vec![
                make_layer(ControllerControlID::LeftShoulder),
                make_layer(ControllerControlID::RightShoulder),
            ],
        }
    }

    /// Windows-native Gabe's Defaults profile (complete parity with macOS GAPE.json).
    pub fn gabes_defaults() -> Self {
        let mut mappings = BTreeMap::new();
        mappings.insert(
            ControllerControlID::LeftTrigger,
            ControllerActionMapping::new(ActionType::LeftMouseHold),
        );
        mappings.insert(
            ControllerControlID::LeftShoulder,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::ESCAPE, vec![]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::RightTrigger,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::H, vec![KeyboardModifier::Win]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::RightShoulder,
            ControllerActionMapping::new(ActionType::RightClick),
        );
        mappings.insert(
            ControllerControlID::ButtonSouth,
            ControllerActionMapping::new(ActionType::LeftClick),
        );
        mappings.insert(
            ControllerControlID::ButtonEast,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::S, vec![KeyboardModifier::Shift, KeyboardModifier::Win]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::ButtonWest,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::T, vec![KeyboardModifier::Shift, KeyboardModifier::Win]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::ButtonNorth,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::V, vec![KeyboardModifier::Control]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::DpadUp,
            ControllerActionMapping::new(ActionType::ScrollUp),
        );
        mappings.insert(
            ControllerControlID::DpadDown,
            ControllerActionMapping::new(ActionType::ScrollDown),
        );
        mappings.insert(
            ControllerControlID::DpadLeft,
            ControllerActionMapping::new(ActionType::SwitchSpaceLeft),
        );
        mappings.insert(
            ControllerControlID::DpadRight,
            ControllerActionMapping::new(ActionType::SwitchSpaceRight),
        );
        mappings.insert(
            ControllerControlID::LeftThumbstickButton,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::RETURN, vec![]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::RightThumbstickButton,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::BACKSPACE, vec![]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::Menu,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::T, vec![KeyboardModifier::Control]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::Options,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::C, vec![KeyboardModifier::Control]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::Home,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::W, vec![KeyboardModifier::Control]),
                TriggerMode::Tap,
            ),
        );
        mappings.insert(
            ControllerControlID::Share,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::OEM_2, vec![]),
                TriggerMode::Tap,
            ),
        );

        let mut lb_layer = BTreeMap::new();
        lb_layer.insert(
            ControllerControlID::ButtonNorth,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::SPACE, vec![]),
                TriggerMode::Tap,
            ),
        );
        lb_layer.insert(
            ControllerControlID::ButtonWest,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::OEM_PERIOD, vec![]),
                TriggerMode::Tap,
            ),
        );
        lb_layer.insert(
            ControllerControlID::Menu,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::TAB, vec![]),
                TriggerMode::Tap,
            ),
        );
        lb_layer.insert(
            ControllerControlID::LeftThumbstickButton,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::RETURN, vec![KeyboardModifier::Shift]),
                TriggerMode::Tap,
            ),
        );
        lb_layer.insert(
            ControllerControlID::RightShoulder,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::left_right_modifier_chord(KeyboardModifier::Control),
                TriggerMode::Tap,
            ),
        );
        lb_layer.insert(
            ControllerControlID::Share,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::KEY_2, vec![KeyboardModifier::Shift]),
                TriggerMode::Tap,
            ),
        );
        lb_layer.insert(
            ControllerControlID::DpadLeft,
            ControllerActionMapping::new(ActionType::CrossEdgeLeft),
        );
        lb_layer.insert(
            ControllerControlID::DpadRight,
            ControllerActionMapping::new(ActionType::CrossEdgeRight),
        );
        lb_layer.insert(
            ControllerControlID::DpadUp,
            ControllerActionMapping::new(ActionType::CrossEdgeUp),
        );
        lb_layer.insert(
            ControllerControlID::DpadDown,
            ControllerActionMapping::new(ActionType::CrossEdgeDown),
        );

        let mut rb_layer = BTreeMap::new();
        rb_layer.insert(
            ControllerControlID::ButtonNorth,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::V, vec![KeyboardModifier::Control]),
                TriggerMode::Tap,
            ),
        );
        rb_layer.insert(
            ControllerControlID::Menu,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::TAB, vec![]),
                TriggerMode::Tap,
            ),
        );
        rb_layer.insert(
            ControllerControlID::LeftThumbstickButton,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::RETURN, vec![KeyboardModifier::Shift]),
                TriggerMode::Tap,
            ),
        );
        rb_layer.insert(
            ControllerControlID::Share,
            ControllerActionMapping::shortcut(
                ShortcutDescriptor::new(vk::KEY_4, vec![KeyboardModifier::Shift]),
                TriggerMode::Tap,
            ),
        );
        rb_layer.insert(
            ControllerControlID::DpadLeft,
            ControllerActionMapping::new(ActionType::CrossEdgeLeft),
        );
        rb_layer.insert(
            ControllerControlID::DpadRight,
            ControllerActionMapping::new(ActionType::CrossEdgeRight),
        );
        rb_layer.insert(
            ControllerControlID::DpadUp,
            ControllerActionMapping::new(ActionType::CrossEdgeUp),
        );
        rb_layer.insert(
            ControllerControlID::DpadDown,
            ControllerActionMapping::new(ActionType::CrossEdgeDown),
        );

        let mut profile = Self {
            id: "gabes-defaults".to_string(),
            name: "Gabe's Defaults".to_string(),
            cursor: CursorConfiguration::default(),
            gyro: crate::engines::gyro_engine::GyroConfiguration::default(),
            radial_menu: default_radial_menu(),
            mappings,
            modifier_layers: vec![
                ControllerModifierLayer {
                    modifier_control: ControllerControlID::LeftShoulder,
                    mappings: lb_layer,
                },
                ControllerModifierLayer {
                    modifier_control: ControllerControlID::RightShoulder,
                    mappings: rb_layer,
                },
            ],
            application_mappings: vec![Self::codex_starter_application_mappings()],
        };
        profile.apply_suggested_vibrations();
        // Gabe's explicit custom overrides over suggested vibrations
        if let Some(m) = profile.mappings.get_mut(&ControllerControlID::Home) {
            m.vibration = ControllerVibration::Thump;
        }
        if let Some(m) = profile
            .mappings
            .get_mut(&ControllerControlID::LeftThumbstickButton)
        {
            m.vibration = ControllerVibration::SoftTap;
        }
        profile
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDocument {
    pub version: u32,
    pub profiles: Vec<ControllerProfile>,
    pub active_profile_id: String,
}

impl ProfileDocument {
    pub const CURRENT_VERSION: u32 = 5;

    pub fn default_document() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            profiles: vec![ControllerProfile::gabes_defaults()],
            active_profile_id: "gabes-defaults".to_string(),
        }
    }
}
