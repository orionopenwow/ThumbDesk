use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::profile::TriggerMode;

/// Cross-platform keyboard modifier, supporting Windows native names and macOS JSON aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KeyboardModifier {
    Control,
    #[serde(alias = "option")]
    Alt,
    Shift,
    #[serde(alias = "command")]
    Win,
}

impl KeyboardModifier {
    pub const DISPLAY_ORDER: &'static [Self] = &[Self::Control, Self::Alt, Self::Shift, Self::Win];

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Control => "Ctrl",
            Self::Alt => "Alt",
            Self::Shift => "Shift",
            Self::Win => "Win",
        }
    }

    pub fn sided_display_name(self) -> &'static str {
        match self {
            Self::Control => "LCtrl",
            Self::Alt => "LAlt",
            Self::Shift => "LShift",
            Self::Win => "LWin",
        }
    }

    pub fn right_sided_display_name(self) -> &'static str {
        match self {
            Self::Control => "RCtrl",
            Self::Alt => "RAlt",
            Self::Shift => "RShift",
            Self::Win => "RWin",
        }
    }

    /// Windows Virtual-Key code (left modifier).
    pub fn vk_code(self) -> u16 {
        match self {
            Self::Control => 0xA2, // VK_LCONTROL
            Self::Alt => 0xA4,     // VK_LMENU
            Self::Shift => 0xA0,   // VK_LSHIFT
            Self::Win => 0x5B,     // VK_LWIN
        }
    }

    /// Windows Virtual-Key code (right modifier).
    pub fn right_vk_code(self) -> u16 {
        match self {
            Self::Control => 0xA3, // VK_RCONTROL
            Self::Alt => 0xA5,     // VK_RMENU
            Self::Shift => 0xA1,   // VK_RSHIFT
            Self::Win => 0x5C,     // VK_RWIN
        }
    }

    pub fn from_right_vk(vk: u16) -> Option<Self> {
        match vk {
            0xA3 => Some(Self::Control),
            0xA5 => Some(Self::Alt),
            0xA1 => Some(Self::Shift),
            0x5C => Some(Self::Win),
            _ => None,
        }
    }
}

pub fn normalize_modifiers(modifiers: &[KeyboardModifier]) -> Vec<KeyboardModifier> {
    let set: BTreeSet<KeyboardModifier> = modifiers.iter().copied().collect();
    KeyboardModifier::DISPLAY_ORDER
        .iter()
        .copied()
        .filter(|m| set.contains(m))
        .collect()
}

/// Windows Virtual-Key shortcut descriptor (compatible with macOS profile JSON import).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutDescriptor {
    /// Windows Virtual-Key code (VK_*).
    pub key_code: u16,
    #[serde(default)]
    pub modifiers: Vec<KeyboardModifier>,
}

impl ShortcutDescriptor {
    pub fn new(key_code: u16, modifiers: Vec<KeyboardModifier>) -> Self {
        Self {
            key_code,
            modifiers: normalize_modifiers(&modifiers),
        }
    }

    pub fn ordered_modifiers(&self) -> Vec<KeyboardModifier> {
        normalize_modifiers(&self.modifiers)
    }

    pub fn left_right_modifier_chord(modifier: KeyboardModifier) -> Self {
        Self::new(modifier.right_vk_code(), vec![modifier])
    }

    pub fn is_modifier_key_code(vk: u16) -> bool {
        matches!(
            vk,
            0x10 | 0x11 | 0x12 | 0x14 | 0x5B | 0x5C | 0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5
        )
    }

    pub fn is_supported_modifier_chord(&self) -> bool {
        if let Some(m) = KeyboardModifier::from_right_vk(self.key_code) {
            return self.ordered_modifiers().contains(&m);
        }
        false
    }

    pub fn is_modifier_only(&self) -> bool {
        Self::is_modifier_key_code(self.key_code)
    }

    pub fn is_assignable(&self) -> bool {
        !self.is_modifier_only() || self.is_supported_modifier_chord()
    }

    pub fn is_assignable_for(&self, trigger_mode: TriggerMode) -> bool {
        self.is_assignable()
            || (self.is_modifier_only() && trigger_mode == TriggerMode::HoldWhilePressed)
    }

    pub fn display_string(&self) -> String {
        if self.is_supported_modifier_chord() {
            if let Some(m) = KeyboardModifier::from_right_vk(self.key_code) {
                return format!("{} + {}", m.sided_display_name(), m.right_sided_display_name());
            }
        }
        let mods = self.ordered_modifiers();
        let key_name = display_name_for_vk(self.key_code);
        if mods.is_empty() {
            key_name
        } else {
            let prefix = mods
                .iter()
                .map(|m| m.display_label())
                .collect::<Vec<_>>()
                .join("+");
            format!("{}+{}", prefix, key_name)
        }
    }

    pub fn duplicate_key(&self) -> String {
        let mods = self
            .ordered_modifiers()
            .iter()
            .map(|m| m.display_label())
            .collect::<Vec<_>>()
            .join("+");
        format!("{}|{}", self.key_code, mods)
    }

    pub fn known_system_conflict_warning(&self) -> Option<&'static str> {
        let mods = self.ordered_modifiers();
        match (self.key_code, mods.as_slice()) {
            (0x09, [KeyboardModifier::Alt]) => {
                Some("Alt+Tab is reserved by the Windows task switcher.")
            }
            (0x73, [KeyboardModifier::Alt]) => {
                Some("Alt+F4 usually closes the frontmost window.")
            }
            (0x4C, [KeyboardModifier::Win]) => Some("Win+L locks Windows immediately."),
            (0x2E, [KeyboardModifier::Control, KeyboardModifier::Alt]) => {
                Some("Ctrl+Alt+Delete is intercepted by Windows Security.")
            }
            _ => None,
        }
    }

    /// Converts a macOS CGKeyCode + macOS modifiers shortcut (such as from GAPE.json)
    /// into an idiomatic Windows Virtual-Key shortcut.
    pub fn from_macos_shortcut(cg_key_code: u16, macos_modifiers: &[KeyboardModifier]) -> Self {
        // Special macOS system workflow shortcuts mapped to native Windows equivalents:
        let ordered = normalize_modifiers(macos_modifiers);
        // Fn (63) -> Win+H (Windows Voice Typing)
        if cg_key_code == 63 && ordered.is_empty() {
            return Self::new(vk::H, vec![KeyboardModifier::Win]);
        }
        // ^⇧⌘4 (area screenshot to clipboard) -> Win+Shift+S (Snipping Tool)
        if cg_key_code == 21
            && ordered == [KeyboardModifier::Control, KeyboardModifier::Shift, KeyboardModifier::Win]
        {
            return Self::new(vk::S, vec![KeyboardModifier::Shift, KeyboardModifier::Win]);
        }
        // ⇧⌘2 (TextSniper OCR) -> Win+Shift+T (PowerToys Text Extractor)
        if cg_key_code == 19 && ordered == [KeyboardModifier::Shift, KeyboardModifier::Win] {
            return Self::new(vk::T, vec![KeyboardModifier::Shift, KeyboardModifier::Win]);
        }
        // L⌘ + R⌘ (54 + [command]) -> LCtrl + RCtrl
        if cg_key_code == 54 && ordered == [KeyboardModifier::Win] {
            return Self::left_right_modifier_chord(KeyboardModifier::Control);
        }

        let win_vk = macos_cg_keycode_to_vk(cg_key_code);
        // On macOS, Command is the standard app shortcut modifier -> map Command to Control on Windows
        let win_mods: Vec<KeyboardModifier> = ordered
            .into_iter()
            .map(|m| match m {
                KeyboardModifier::Win => KeyboardModifier::Control,
                other => other,
            })
            .collect();
        Self::new(win_vk, win_mods)
    }
}

/// Common Windows Virtual-Key constants.
pub mod vk {
    pub const BACKSPACE: u16 = 0x08;
    pub const TAB: u16 = 0x09;
    pub const RETURN: u16 = 0x0D;
    pub const ESCAPE: u16 = 0x1B;
    pub const SPACE: u16 = 0x20;
    pub const PAGE_UP: u16 = 0x21;
    pub const PAGE_DOWN: u16 = 0x22;
    pub const END: u16 = 0x23;
    pub const HOME: u16 = 0x24;
    pub const LEFT: u16 = 0x25;
    pub const UP: u16 = 0x26;
    pub const RIGHT: u16 = 0x27;
    pub const DOWN: u16 = 0x28;
    pub const DELETE: u16 = 0x2E;

    pub const KEY_0: u16 = 0x30;
    pub const KEY_1: u16 = 0x31;
    pub const KEY_2: u16 = 0x32;
    pub const KEY_3: u16 = 0x33;
    pub const KEY_4: u16 = 0x34;
    pub const KEY_5: u16 = 0x35;
    pub const KEY_6: u16 = 0x36;
    pub const KEY_7: u16 = 0x37;
    pub const KEY_8: u16 = 0x38;
    pub const KEY_9: u16 = 0x39;

    pub const A: u16 = 0x41;
    pub const B: u16 = 0x42;
    pub const C: u16 = 0x43;
    pub const D: u16 = 0x44;
    pub const E: u16 = 0x45;
    pub const F: u16 = 0x46;
    pub const G: u16 = 0x47;
    pub const H: u16 = 0x48;
    pub const I: u16 = 0x49;
    pub const J: u16 = 0x4A;
    pub const K: u16 = 0x4B;
    pub const L: u16 = 0x4C;
    pub const M: u16 = 0x4D;
    pub const N: u16 = 0x4E;
    pub const O: u16 = 0x4F;
    pub const P: u16 = 0x50;
    pub const Q: u16 = 0x51;
    pub const R: u16 = 0x52;
    pub const S: u16 = 0x53;
    pub const T: u16 = 0x54;
    pub const U: u16 = 0x55;
    pub const V: u16 = 0x56;
    pub const W: u16 = 0x57;
    pub const X: u16 = 0x58;
    pub const Y: u16 = 0x59;
    pub const Z: u16 = 0x5A;

    pub const F1: u16 = 0x70;
    pub const F2: u16 = 0x71;
    pub const F3: u16 = 0x72;
    pub const F4: u16 = 0x73;
    pub const F5: u16 = 0x74;
    pub const F6: u16 = 0x75;
    pub const F7: u16 = 0x76;
    pub const F8: u16 = 0x77;
    pub const F9: u16 = 0x78;
    pub const F10: u16 = 0x79;
    pub const F11: u16 = 0x7A;
    pub const F12: u16 = 0x7B;

    pub const OEM_1: u16 = 0xBA;      // ;:
    pub const OEM_PLUS: u16 = 0xBB;   // =+
    pub const OEM_COMMA: u16 = 0xBC;  // ,<
    pub const OEM_MINUS: u16 = 0xBD;  // -_
    pub const OEM_PERIOD: u16 = 0xBE; // .>
    pub const OEM_2: u16 = 0xBF;      // /?
    pub const OEM_3: u16 = 0xC0;      // `~
    pub const OEM_4: u16 = 0xDB;      // [{
    pub const OEM_5: u16 = 0xDC;      // \|
    pub const OEM_6: u16 = 0xDD;      // ]}
    pub const OEM_7: u16 = 0xDE;      // '"
}

pub fn display_name_for_vk(code: u16) -> String {
    let static_name = match code {
        0x08 => Some("Backspace"),
        0x09 => Some("Tab"),
        0x0D => Some("Enter"),
        0x14 => Some("Caps Lock"),
        0x1B => Some("Esc"),
        0x20 => Some("Space"),
        0x21 => Some("PgUp"),
        0x22 => Some("PgDn"),
        0x23 => Some("End"),
        0x24 => Some("Home"),
        0x25 => Some("Left"),
        0x26 => Some("Up"),
        0x27 => Some("Right"),
        0x28 => Some("Down"),
        0x2E => Some("Del"),
        0x5B => Some("LWin"),
        0x5C => Some("RWin"),
        0xA0 => Some("LShift"),
        0xA1 => Some("RShift"),
        0xA2 => Some("LCtrl"),
        0xA3 => Some("RCtrl"),
        0xA4 => Some("LAlt"),
        0xA5 => Some("RAlt"),
        0x70 => Some("F1"),
        0x71 => Some("F2"),
        0x72 => Some("F3"),
        0x73 => Some("F4"),
        0x74 => Some("F5"),
        0x75 => Some("F6"),
        0x76 => Some("F7"),
        0x77 => Some("F8"),
        0x78 => Some("F9"),
        0x79 => Some("F10"),
        0x7A => Some("F11"),
        0x7B => Some("F12"),
        0xBA => Some(";"),
        0xBB => Some("="),
        0xBC => Some(","),
        0xBD => Some("-"),
        0xBE => Some("."),
        0xBF => Some("/"),
        0xC0 => Some("`"),
        0xDB => Some("["),
        0xDC => Some("\\"),
        0xDD => Some("]"),
        0xDE => Some("'"),
        _ => None,
    };
    if let Some(name) = static_name {
        return name.to_string();
    }
    if (0x30..=0x39).contains(&code) || (0x41..=0x5A).contains(&code) {
        return (code as u8 as char).to_string();
    }
    format!("Key 0x{:02X}", code)
}

pub fn macos_cg_keycode_to_vk(cg: u16) -> u16 {
    match cg {
        0 => vk::A,
        1 => vk::S,
        2 => vk::D,
        3 => vk::F,
        4 => vk::H,
        5 => vk::G,
        6 => vk::Z,
        7 => vk::X,
        8 => vk::C,
        9 => vk::V,
        11 => vk::B,
        12 => vk::Q,
        13 => vk::W,
        14 => vk::E,
        15 => vk::R,
        16 => vk::Y,
        17 => vk::T,
        18 => vk::KEY_1,
        19 => vk::KEY_2,
        20 => vk::KEY_3,
        21 => vk::KEY_4,
        22 => vk::KEY_6,
        23 => vk::KEY_5,
        24 => vk::OEM_PLUS,
        25 => vk::KEY_9,
        26 => vk::KEY_7,
        27 => vk::OEM_MINUS,
        28 => vk::KEY_8,
        29 => vk::KEY_0,
        30 => vk::OEM_6,
        31 => vk::O,
        32 => vk::U,
        33 => vk::OEM_4,
        34 => vk::I,
        35 => vk::P,
        36 => vk::RETURN,
        37 => vk::L,
        38 => vk::J,
        39 => vk::OEM_7,
        40 => vk::K,
        41 => vk::OEM_1,
        42 => vk::OEM_5,
        43 => vk::OEM_COMMA,
        44 => vk::OEM_2,
        45 => vk::N,
        46 => vk::M,
        47 => vk::OEM_PERIOD,
        48 => vk::TAB,
        49 => vk::SPACE,
        50 => vk::OEM_3,
        51 => vk::BACKSPACE,
        53 => vk::ESCAPE,
        54 => 0x5C, // RWin
        55 => 0x5B, // LWin
        56 => 0xA0, // LShift
        57 => 0x14, // CapsLock
        58 => 0xA4, // LAlt
        59 => 0xA2, // LCtrl
        60 => 0xA1, // RShift
        61 => 0xA5, // RAlt
        62 => 0xA3, // RCtrl
        96 => vk::F5,
        97 => vk::F6,
        98 => vk::F7,
        99 => vk::F3,
        100 => vk::F8,
        101 => vk::F9,
        103 => vk::F11,
        109 => vk::F10,
        111 => vk::F12,
        115 => vk::HOME,
        116 => vk::PAGE_UP,
        117 => vk::DELETE,
        118 => vk::F4,
        119 => vk::END,
        120 => vk::F2,
        121 => vk::PAGE_DOWN,
        122 => vk::F1,
        123 => vk::LEFT,
        124 => vk::RIGHT,
        125 => vk::DOWN,
        126 => vk::UP,
        other => other,
    }
}
