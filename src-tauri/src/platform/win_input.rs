use std::mem::size_of;
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC, MOUSEEVENTF_HWHEEL,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT,
    MOUSE_EVENT_FLAGS, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SetCursorPos, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

use crate::models::companion::{
    CompanionEdge, CompanionMouseButton, CompanionMousePhase, CompanionShortcutPhase,
    SpaceSwitchDirection,
};
use crate::models::shortcut::{vk, KeyboardModifier, ShortcutDescriptor};

pub const EDGE_HANDOFF_THRESHOLD: f64 = 2.0;
pub const LOCAL_RESTORE_INSET: f64 = 12.0;
pub const REMOTE_ENTRY_INSET: f64 = 8.0;
const WHEEL_DELTA: i32 = 120;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesktopBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub width: f64,
    pub height: f64,
}

impl DesktopBounds {
    pub fn max_x(&self) -> f64 {
        self.min_x + self.width
    }

    pub fn max_y(&self) -> f64 {
        self.min_y + self.height
    }
}

pub fn virtual_desktop_bounds() -> DesktopBounds {
    unsafe {
        DesktopBounds {
            min_x: GetSystemMetrics(SM_XVIRTUALSCREEN) as f64,
            min_y: GetSystemMetrics(SM_YVIRTUALSCREEN) as f64,
            width: GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1) as f64,
            height: GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1) as f64,
        }
    }
}

pub fn get_cursor_pos() -> (f64, f64) {
    let mut pt = POINT::default();
    unsafe {
        if GetCursorPos(&mut pt).is_ok() {
            return (pt.x as f64, pt.y as f64);
        }
    }
    (0.0, 0.0)
}

pub fn set_cursor_pos(x: f64, y: f64) {
    unsafe {
        let _ = SetCursorPos(x.round() as i32, y.round() as i32);
    }
}

/// Checks if pointer movement pushes past `edge` within `EDGE_HANDOFF_THRESHOLD` (2 px).
pub fn should_begin_handoff(
    loc: (f64, f64),
    delta: (f64, f64),
    edge: CompanionEdge,
    bounds: DesktopBounds,
) -> bool {
    let proj_x = (loc.0 + delta.0).clamp(bounds.min_x, bounds.max_x() - 1.0);
    let proj_y = (loc.1 + delta.1).clamp(bounds.min_y, bounds.max_y() - 1.0);
    match edge {
        CompanionEdge::Left => {
            (loc.0 <= bounds.min_x + EDGE_HANDOFF_THRESHOLD
                || proj_x <= bounds.min_x + EDGE_HANDOFF_THRESHOLD)
                && delta.0 < 0.0
        }
        CompanionEdge::Right => {
            (loc.0 >= bounds.max_x() - EDGE_HANDOFF_THRESHOLD
                || proj_x >= bounds.max_x() - EDGE_HANDOFF_THRESHOLD)
                && delta.0 > 0.0
        }
        CompanionEdge::Top => {
            (loc.1 <= bounds.min_y + EDGE_HANDOFF_THRESHOLD
                || proj_y <= bounds.min_y + EDGE_HANDOFF_THRESHOLD)
                && delta.1 < 0.0
        }
        CompanionEdge::Bottom => {
            (loc.1 >= bounds.max_y() - EDGE_HANDOFF_THRESHOLD
                || proj_y >= bounds.max_y() - EDGE_HANDOFF_THRESHOLD)
                && delta.1 > 0.0
        }
    }
}

pub fn normalized_edge_position(
    loc: (f64, f64),
    edge: CompanionEdge,
    bounds: DesktopBounds,
) -> f64 {
    match edge {
        CompanionEdge::Left | CompanionEdge::Right => {
            ((loc.1 - bounds.min_y) / bounds.height.max(1.0)).clamp(0.0, 1.0)
        }
        CompanionEdge::Top | CompanionEdge::Bottom => {
            ((loc.0 - bounds.min_x) / bounds.width.max(1.0)).clamp(0.0, 1.0)
        }
    }
}

pub fn local_restore_point(
    loc: (f64, f64),
    edge: CompanionEdge,
    bounds: DesktopBounds,
) -> (f64, f64) {
    match edge {
        CompanionEdge::Left => (bounds.min_x + LOCAL_RESTORE_INSET, loc.1),
        CompanionEdge::Right => (bounds.max_x() - LOCAL_RESTORE_INSET, loc.1),
        CompanionEdge::Top => (loc.0, bounds.min_y + LOCAL_RESTORE_INSET),
        CompanionEdge::Bottom => (loc.0, bounds.max_y() - LOCAL_RESTORE_INSET),
    }
}

pub fn remote_entry_point(
    edge: CompanionEdge,
    normalized_pos: f64,
    bounds: DesktopBounds,
) -> (f64, f64) {
    let norm = normalized_pos.clamp(0.0, 1.0);
    match edge {
        CompanionEdge::Left => (
            bounds.max_x() - REMOTE_ENTRY_INSET,
            bounds.min_y + bounds.height * norm,
        ),
        CompanionEdge::Right => (
            bounds.min_x + REMOTE_ENTRY_INSET,
            bounds.min_y + bounds.height * norm,
        ),
        CompanionEdge::Top => (
            bounds.min_x + bounds.width * norm,
            bounds.max_y() - REMOTE_ENTRY_INSET,
        ),
        CompanionEdge::Bottom => (
            bounds.min_x + bounds.width * norm,
            bounds.min_y + REMOTE_ENTRY_INSET,
        ),
    }
}

fn send_inputs(inputs: &[INPUT]) {
    if inputs.is_empty() {
        return;
    }
    unsafe {
        let _ = SendInput(inputs, size_of::<INPUT>() as i32);
    }
}

fn mouse_input(dx: i32, dy: i32, data: u32, flags: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn move_mouse_relative(dx: i32, dy: i32) {
    if dx == 0 && dy == 0 {
        return;
    }
    send_inputs(&[mouse_input(dx, dy, 0, MOUSEEVENTF_MOVE)]);
}

pub fn send_mouse_button(button: CompanionMouseButton, phase: CompanionMousePhase) {
    let (down_flag, up_flag) = match button {
        CompanionMouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        CompanionMouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        CompanionMouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    };
    match phase {
        CompanionMousePhase::Down => send_inputs(&[mouse_input(0, 0, 0, down_flag)]),
        CompanionMousePhase::Up => send_inputs(&[mouse_input(0, 0, 0, up_flag)]),
        CompanionMousePhase::Click => send_inputs(&[
            mouse_input(0, 0, 0, down_flag),
            mouse_input(0, 0, 0, up_flag),
        ]),
        CompanionMousePhase::DoubleClick => {
            send_inputs(&[
                mouse_input(0, 0, 0, down_flag),
                mouse_input(0, 0, 0, up_flag),
            ]);
            thread::sleep(Duration::from_millis(25));
            send_inputs(&[
                mouse_input(0, 0, 0, down_flag),
                mouse_input(0, 0, 0, up_flag),
            ]);
        }
    }
}

/// Sends vertical and/or horizontal mouse wheel scroll.
/// Convention: vertical < 0 => ScrollUp (+WHEEL_DELTA), vertical > 0 => ScrollDown (-WHEEL_DELTA).
pub fn send_scroll(vertical: i32, horizontal: i32) {
    let mut inputs = Vec::new();
    if vertical != 0 {
        let delta = (-vertical * WHEEL_DELTA) as u32;
        inputs.push(mouse_input(0, 0, delta, MOUSEEVENTF_WHEEL));
    }
    if horizontal != 0 {
        let delta = (horizontal * WHEEL_DELTA) as u32;
        inputs.push(mouse_input(0, 0, delta, MOUSEEVENTF_HWHEEL));
    }
    send_inputs(&inputs);
}

fn is_extended_vk(code: u16) -> bool {
    matches!(
        code,
        0x21..=0x28 | 0x2D | 0x2E | 0x5B | 0x5C | 0xA3 | 0xA5
    )
}

fn key_input(vk_code: u16, key_up: bool) -> INPUT {
    let scan = unsafe { MapVirtualKeyW(vk_code as u32, MAPVK_VK_TO_VSC) as u16 };
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if is_extended_vk(vk_code) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk_code),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn send_shortcut(shortcut: &ShortcutDescriptor, phase: CompanionShortcutPhase) {
    let mods = shortcut.ordered_modifiers();
    let mut inputs = Vec::new();

    match phase {
        CompanionShortcutPhase::Down => {
            for m in &mods {
                inputs.push(key_input(m.vk_code(), false));
            }
            inputs.push(key_input(shortcut.key_code, false));
        }
        CompanionShortcutPhase::Up => {
            inputs.push(key_input(shortcut.key_code, true));
            for m in mods.iter().rev() {
                inputs.push(key_input(m.vk_code(), true));
            }
        }
        CompanionShortcutPhase::Tap => {
            for m in &mods {
                inputs.push(key_input(m.vk_code(), false));
            }
            inputs.push(key_input(shortcut.key_code, false));
            inputs.push(key_input(shortcut.key_code, true));
            for m in mods.iter().rev() {
                inputs.push(key_input(m.vk_code(), true));
            }
        }
    }
    send_inputs(&inputs);
}

/// Switches Windows Virtual Desktop using Ctrl + Win + Left / Right.
pub fn send_switch_virtual_desktop(direction: SpaceSwitchDirection) {
    let arrow = match direction {
        SpaceSwitchDirection::Left => vk::LEFT,
        SpaceSwitchDirection::Right => vk::RIGHT,
    };
    let shortcut = ShortcutDescriptor::new(
        arrow,
        vec![KeyboardModifier::Control, KeyboardModifier::Win],
    );
    send_shortcut(&shortcut, CompanionShortcutPhase::Tap);
}
