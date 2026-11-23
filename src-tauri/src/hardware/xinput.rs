use std::collections::{BTreeMap, BTreeSet};

use windows::Win32::UI::Input::XboxController::{
    XInputGetBatteryInformation, XInputGetState, XInputSetState, BATTERY_DEVTYPE_GAMEPAD,
    BATTERY_LEVEL_EMPTY, BATTERY_LEVEL_FULL, BATTERY_LEVEL_LOW, BATTERY_LEVEL_MEDIUM,
    BATTERY_TYPE_WIRED, XINPUT_BATTERY_INFORMATION, XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B,
    XINPUT_GAMEPAD_BACK, XINPUT_GAMEPAD_DPAD_DOWN, XINPUT_GAMEPAD_DPAD_LEFT,
    XINPUT_GAMEPAD_DPAD_RIGHT, XINPUT_GAMEPAD_DPAD_UP, XINPUT_GAMEPAD_LEFT_SHOULDER,
    XINPUT_GAMEPAD_LEFT_THUMB, XINPUT_GAMEPAD_RIGHT_SHOULDER, XINPUT_GAMEPAD_RIGHT_THUMB,
    XINPUT_GAMEPAD_START, XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y, XINPUT_STATE, XINPUT_VIBRATION,
};

use crate::models::control::{
    ControllerControlID, ControllerFamily, ControllerSnapshot, StickSnapshot,
};

fn normalize_thumb(raw: i16) -> f64 {
    if raw >= 0 {
        (raw as f64 / 32767.0).clamp(-1.0, 1.0)
    } else {
        (raw as f64 / 32768.0).clamp(-1.0, 1.0)
    }
}

fn normalize_trigger(raw: u8) -> f64 {
    (raw as f64 / 255.0).clamp(0.0, 1.0)
}

#[derive(Debug, Clone)]
pub struct XInputReading {
    pub slot: u32,
    pub snapshot: ControllerSnapshot,
}

/// Polls XInput slots 0..4 and returns the first connected Xbox controller snapshot.
pub fn poll_xinput(event_count: u64) -> Option<XInputReading> {
    for slot in 0..4u32 {
        let mut state = XINPUT_STATE::default();
        let res = unsafe { XInputGetState(slot, &mut state) };
        if res != 0 {
            continue;
        }
        let gp = state.Gamepad;
        let buttons = gp.wButtons.0;
        let mut pressed = BTreeSet::new();

        let map_btn = |mask: u16, id: ControllerControlID, set: &mut BTreeSet<_>| {
            if (buttons & mask) != 0 {
                set.insert(id);
            }
        };

        map_btn(XINPUT_GAMEPAD_A.0, ControllerControlID::ButtonSouth, &mut pressed);
        map_btn(XINPUT_GAMEPAD_B.0, ControllerControlID::ButtonEast, &mut pressed);
        map_btn(XINPUT_GAMEPAD_X.0, ControllerControlID::ButtonWest, &mut pressed);
        map_btn(XINPUT_GAMEPAD_Y.0, ControllerControlID::ButtonNorth, &mut pressed);
        map_btn(
            XINPUT_GAMEPAD_LEFT_SHOULDER.0,
            ControllerControlID::LeftShoulder,
            &mut pressed,
        );
        map_btn(
            XINPUT_GAMEPAD_RIGHT_SHOULDER.0,
            ControllerControlID::RightShoulder,
            &mut pressed,
        );
        map_btn(XINPUT_GAMEPAD_DPAD_UP.0, ControllerControlID::DpadUp, &mut pressed);
        map_btn(
            XINPUT_GAMEPAD_DPAD_DOWN.0,
            ControllerControlID::DpadDown,
            &mut pressed,
        );
        map_btn(
            XINPUT_GAMEPAD_DPAD_LEFT.0,
            ControllerControlID::DpadLeft,
            &mut pressed,
        );
        map_btn(
            XINPUT_GAMEPAD_DPAD_RIGHT.0,
            ControllerControlID::DpadRight,
            &mut pressed,
        );
        map_btn(
            XINPUT_GAMEPAD_LEFT_THUMB.0,
            ControllerControlID::LeftThumbstickButton,
            &mut pressed,
        );
        map_btn(
            XINPUT_GAMEPAD_RIGHT_THUMB.0,
            ControllerControlID::RightThumbstickButton,
            &mut pressed,
        );
        map_btn(XINPUT_GAMEPAD_START.0, ControllerControlID::Menu, &mut pressed);
        map_btn(XINPUT_GAMEPAD_BACK.0, ControllerControlID::Options, &mut pressed);

        let lt = normalize_trigger(gp.bLeftTrigger);
        let rt = normalize_trigger(gp.bRightTrigger);
        if lt >= 0.18 {
            pressed.insert(ControllerControlID::LeftTrigger);
        }
        if rt >= 0.18 {
            pressed.insert(ControllerControlID::RightTrigger);
        }

        let mut analog = BTreeMap::new();
        analog.insert(ControllerControlID::LeftTrigger, lt);
        analog.insert(ControllerControlID::RightTrigger, rt);

        let mut bat = XINPUT_BATTERY_INFORMATION::default();
        let bat_ok =
            unsafe { XInputGetBatteryInformation(slot, BATTERY_DEVTYPE_GAMEPAD, &mut bat) } == 0;
        let (transport, battery_level) = if bat_ok {
            if bat.BatteryType == BATTERY_TYPE_WIRED {
                (Some("USB / XInput".to_string()), None)
            } else {
                let lvl = match bat.BatteryLevel {
                    x if x == BATTERY_LEVEL_EMPTY => Some(0.10),
                    x if x == BATTERY_LEVEL_LOW => Some(0.35),
                    x if x == BATTERY_LEVEL_MEDIUM => Some(0.70),
                    x if x == BATTERY_LEVEL_FULL => Some(1.00),
                    _ => None,
                };
                (Some("Wireless / XInput".to_string()), lvl)
            }
        } else {
            (Some("XInput".to_string()), None)
        };

        let snapshot = ControllerSnapshot {
            is_connected: true,
            name: Some(format!("Xbox Controller (Slot {})", slot + 1)),
            family: ControllerFamily::Xbox,
            transport,
            battery_level,
            left_stick: StickSnapshot {
                x: normalize_thumb(gp.sThumbLX),
                y: normalize_thumb(gp.sThumbLY),
                pressed: (buttons & XINPUT_GAMEPAD_LEFT_THUMB.0) != 0,
            },
            right_stick: StickSnapshot {
                x: normalize_thumb(gp.sThumbRX),
                y: normalize_thumb(gp.sThumbRY),
                pressed: (buttons & XINPUT_GAMEPAD_RIGHT_THUMB.0) != 0,
            },
            pressed_controls: pressed,
            analog_values: analog,
            event_count,
            share_supported: false,
        };

        return Some(XInputReading { slot, snapshot });
    }
    None
}

/// Sends dual-motor rumble to an XInput controller slot.
pub fn set_xinput_rumble(slot: u32, left: f32, right: f32) -> bool {
    let vibration = XINPUT_VIBRATION {
        wLeftMotorSpeed: (left.clamp(0.0, 1.0) * 65535.0).round() as u16,
        wRightMotorSpeed: (right.clamp(0.0, 1.0) * 65535.0).round() as u16,
    };
    unsafe { XInputSetState(slot, &vibration) == 0 }
}
