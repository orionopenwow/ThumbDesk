use std::collections::{BTreeMap, BTreeSet};

use hidapi::{HidApi, HidDevice};

use crate::models::control::{
    ControllerControlID, ControllerFamily, ControllerSnapshot, StickSnapshot,
};

pub const SONY_VID: u16 = 0x054C;
pub const DUALSENSE_PID: u16 = 0x0CE6;
pub const DUALSENSE_EDGE_PID: u16 = 0x0DF2;
pub const DUALSHOCK4_V1_PID: u16 = 0x05C4;
pub const DUALSHOCK4_V2_PID: u16 = 0x09CC;

pub const MICROSOFT_VID: u16 = 0x045E;
pub const XBOX360_WIRED_PID: u16 = 0x028E;
pub const XBOX_SERIES_USB_PID: u16 = 0x0B12;

fn norm_u8_stick(v: u8) -> f64 {
    ((v as f64 - 127.5) / 127.5).clamp(-1.0, 1.0)
}

fn apply_dpad_hat(hat: u8, pressed: &mut BTreeSet<ControllerControlID>) {
    match hat & 0x0F {
        0 => {
            pressed.insert(ControllerControlID::DpadUp);
        }
        1 => {
            pressed.insert(ControllerControlID::DpadUp);
            pressed.insert(ControllerControlID::DpadRight);
        }
        2 => {
            pressed.insert(ControllerControlID::DpadRight);
        }
        3 => {
            pressed.insert(ControllerControlID::DpadRight);
            pressed.insert(ControllerControlID::DpadDown);
        }
        4 => {
            pressed.insert(ControllerControlID::DpadDown);
        }
        5 => {
            pressed.insert(ControllerControlID::DpadDown);
            pressed.insert(ControllerControlID::DpadLeft);
        }
        6 => {
            pressed.insert(ControllerControlID::DpadLeft);
        }
        7 => {
            pressed.insert(ControllerControlID::DpadLeft);
            pressed.insert(ControllerControlID::DpadUp);
        }
        _ => {}
    }
}

/// Parses a Sony DualSense USB (report ID 0x01) or Bluetooth (report ID 0x31) HID input report.
pub fn parse_dualsense_report(data: &[u8], event_count: u64) -> Option<ControllerSnapshot> {
    let (offset, is_bt) = match data.first().copied()? {
        0x01 if data.len() >= 11 => (1usize, false),
        0x31 if data.len() >= 12 => (2usize, true),
        _ => return None,
    };
    let lx = norm_u8_stick(data[offset]);
    let ly = -norm_u8_stick(data[offset + 1]); // invert Y so up is +1
    let rx = norm_u8_stick(data[offset + 2]);
    let ry = -norm_u8_stick(data[offset + 3]);
    let lt = data[offset + 4] as f64 / 255.0;
    let rt = data[offset + 5] as f64 / 255.0;
    let b0 = data[offset + 7];
    let b1 = data[offset + 8];
    let b2 = data[offset + 9];

    let mut pressed = BTreeSet::new();
    apply_dpad_hat(b0 & 0x0F, &mut pressed);
    if (b0 & 0x10) != 0 {
        pressed.insert(ControllerControlID::ButtonWest); // Square
    }
    if (b0 & 0x20) != 0 {
        pressed.insert(ControllerControlID::ButtonSouth); // Cross
    }
    if (b0 & 0x40) != 0 {
        pressed.insert(ControllerControlID::ButtonEast); // Circle
    }
    if (b0 & 0x80) != 0 {
        pressed.insert(ControllerControlID::ButtonNorth); // Triangle
    }
    if (b1 & 0x01) != 0 {
        pressed.insert(ControllerControlID::LeftShoulder); // L1
    }
    if (b1 & 0x02) != 0 {
        pressed.insert(ControllerControlID::RightShoulder); // R1
    }
    if (b1 & 0x04) != 0 || lt >= 0.18 {
        pressed.insert(ControllerControlID::LeftTrigger);
    }
    if (b1 & 0x08) != 0 || rt >= 0.18 {
        pressed.insert(ControllerControlID::RightTrigger);
    }
    if (b1 & 0x10) != 0 {
        pressed.insert(ControllerControlID::Options); // Create
    }
    if (b1 & 0x20) != 0 {
        pressed.insert(ControllerControlID::Menu); // Options
    }
    let l3 = (b1 & 0x40) != 0;
    let r3 = (b1 & 0x80) != 0;
    if l3 {
        pressed.insert(ControllerControlID::LeftThumbstickButton);
    }
    if r3 {
        pressed.insert(ControllerControlID::RightThumbstickButton);
    }
    if (b2 & 0x01) != 0 {
        pressed.insert(ControllerControlID::Home); // PS button
    }
    if (b2 & 0x02) != 0 {
        pressed.insert(ControllerControlID::TouchpadButton);
    }

    let mut analog = BTreeMap::new();
    analog.insert(ControllerControlID::LeftTrigger, lt);
    analog.insert(ControllerControlID::RightTrigger, rt);

    Some(ControllerSnapshot {
        is_connected: true,
        name: Some("DualSense Wireless Controller".to_string()),
        family: ControllerFamily::PlayStation,
        transport: Some(if is_bt { "Bluetooth HID" } else { "USB HID" }.to_string()),
        battery_level: None,
        left_stick: StickSnapshot {
            x: lx,
            y: ly,
            pressed: l3,
        },
        right_stick: StickSnapshot {
            x: rx,
            y: ry,
            pressed: r3,
        },
        pressed_controls: pressed,
        analog_values: analog,
        event_count,
        share_supported: false,
    })
}

/// Parses a Sony DualShock 4 USB (report ID 0x01) or Bluetooth (report ID 0x11) HID input report.
pub fn parse_dualshock4_report(data: &[u8], event_count: u64) -> Option<ControllerSnapshot> {
    let (offset, is_bt) = match data.first().copied()? {
        0x01 if data.len() >= 10 => (1usize, false),
        0x11 if data.len() >= 12 => (3usize, true),
        _ => return None,
    };
    let lx = norm_u8_stick(data[offset]);
    let ly = -norm_u8_stick(data[offset + 1]);
    let rx = norm_u8_stick(data[offset + 2]);
    let ry = -norm_u8_stick(data[offset + 3]);
    let b0 = data[offset + 4];
    let b1 = data[offset + 5];
    let b2 = data[offset + 6];
    let lt = data[offset + 7] as f64 / 255.0;
    let rt = data[offset + 8] as f64 / 255.0;

    let mut pressed = BTreeSet::new();
    apply_dpad_hat(b0 & 0x0F, &mut pressed);
    if (b0 & 0x10) != 0 {
        pressed.insert(ControllerControlID::ButtonWest);
    }
    if (b0 & 0x20) != 0 {
        pressed.insert(ControllerControlID::ButtonSouth);
    }
    if (b0 & 0x40) != 0 {
        pressed.insert(ControllerControlID::ButtonEast);
    }
    if (b0 & 0x80) != 0 {
        pressed.insert(ControllerControlID::ButtonNorth);
    }
    if (b1 & 0x01) != 0 {
        pressed.insert(ControllerControlID::LeftShoulder);
    }
    if (b1 & 0x02) != 0 {
        pressed.insert(ControllerControlID::RightShoulder);
    }
    if (b1 & 0x04) != 0 || lt >= 0.18 {
        pressed.insert(ControllerControlID::LeftTrigger);
    }
    if (b1 & 0x08) != 0 || rt >= 0.18 {
        pressed.insert(ControllerControlID::RightTrigger);
    }
    if (b1 & 0x10) != 0 {
        pressed.insert(ControllerControlID::Options);
    }
    if (b1 & 0x20) != 0 {
        pressed.insert(ControllerControlID::Menu);
    }
    let l3 = (b1 & 0x40) != 0;
    let r3 = (b1 & 0x80) != 0;
    if l3 {
        pressed.insert(ControllerControlID::LeftThumbstickButton);
    }
    if r3 {
        pressed.insert(ControllerControlID::RightThumbstickButton);
    }
    if (b2 & 0x01) != 0 {
        pressed.insert(ControllerControlID::Home);
    }
    if (b2 & 0x02) != 0 {
        pressed.insert(ControllerControlID::TouchpadButton);
    }

    let mut analog = BTreeMap::new();
    analog.insert(ControllerControlID::LeftTrigger, lt);
    analog.insert(ControllerControlID::RightTrigger, rt);

    Some(ControllerSnapshot {
        is_connected: true,
        name: Some("DualShock 4 Controller".to_string()),
        family: ControllerFamily::PlayStation,
        transport: Some(if is_bt { "Bluetooth HID" } else { "USB HID" }.to_string()),
        battery_level: None,
        left_stick: StickSnapshot {
            x: lx,
            y: ly,
            pressed: l3,
        },
        right_stick: StickSnapshot {
            x: rx,
            y: ry,
            pressed: r3,
        },
        pressed_controls: pressed,
        analog_values: analog,
        event_count,
        share_supported: false,
    })
}

/// Parses a genuine wired Xbox 360 20-byte USB report (0x00, 0x14, ...).
pub fn parse_xbox360_usb_report(data: &[u8], event_count: u64) -> Option<ControllerSnapshot> {
    if data.len() < 14 || data[0] != 0x00 || data[1] != 0x14 {
        return None;
    }
    let b0 = data[2];
    let b1 = data[3];
    let lt = data[4] as f64 / 255.0;
    let rt = data[5] as f64 / 255.0;
    let lx_raw = i16::from_le_bytes([data[6], data[7]]);
    let ly_raw = i16::from_le_bytes([data[8], data[9]]);
    let rx_raw = i16::from_le_bytes([data[10], data[11]]);
    let ry_raw = i16::from_le_bytes([data[12], data[13]]);

    let norm_i16 = |v: i16| {
        if v >= 0 {
            v as f64 / 32767.0
        } else {
            v as f64 / 32768.0
        }
    };

    let mut pressed = BTreeSet::new();
    if (b0 & 0x01) != 0 {
        pressed.insert(ControllerControlID::DpadUp);
    }
    if (b0 & 0x02) != 0 {
        pressed.insert(ControllerControlID::DpadDown);
    }
    if (b0 & 0x04) != 0 {
        pressed.insert(ControllerControlID::DpadLeft);
    }
    if (b0 & 0x08) != 0 {
        pressed.insert(ControllerControlID::DpadRight);
    }
    if (b0 & 0x10) != 0 {
        pressed.insert(ControllerControlID::Menu); // Start
    }
    if (b0 & 0x20) != 0 {
        pressed.insert(ControllerControlID::Options); // Back
    }
    let l3 = (b0 & 0x40) != 0;
    let r3 = (b0 & 0x80) != 0;
    if l3 {
        pressed.insert(ControllerControlID::LeftThumbstickButton);
    }
    if r3 {
        pressed.insert(ControllerControlID::RightThumbstickButton);
    }
    if (b1 & 0x01) != 0 {
        pressed.insert(ControllerControlID::LeftShoulder);
    }
    if (b1 & 0x02) != 0 {
        pressed.insert(ControllerControlID::RightShoulder);
    }
    if (b1 & 0x04) != 0 {
        pressed.insert(ControllerControlID::Home); // Guide
    }
    if (b1 & 0x10) != 0 {
        pressed.insert(ControllerControlID::ButtonSouth);
    }
    if (b1 & 0x20) != 0 {
        pressed.insert(ControllerControlID::ButtonEast);
    }
    if (b1 & 0x40) != 0 {
        pressed.insert(ControllerControlID::ButtonWest);
    }
    if (b1 & 0x80) != 0 {
        pressed.insert(ControllerControlID::ButtonNorth);
    }
    if lt >= 0.18 {
        pressed.insert(ControllerControlID::LeftTrigger);
    }
    if rt >= 0.18 {
        pressed.insert(ControllerControlID::RightTrigger);
    }

    let mut analog = BTreeMap::new();
    analog.insert(ControllerControlID::LeftTrigger, lt);
    analog.insert(ControllerControlID::RightTrigger, rt);

    Some(ControllerSnapshot {
        is_connected: true,
        name: Some("Xbox 360 Wired Controller".to_string()),
        family: ControllerFamily::Xbox,
        transport: Some("USB Direct".to_string()),
        battery_level: None,
        left_stick: StickSnapshot {
            x: norm_i16(lx_raw),
            y: norm_i16(ly_raw),
            pressed: l3,
        },
        right_stick: StickSnapshot {
            x: norm_i16(rx_raw),
            y: norm_i16(ry_raw),
            pressed: r3,
        },
        pressed_controls: pressed,
        analog_values: analog,
        event_count,
        share_supported: false,
    })
}

/// Builds a 48-byte Sony DualSense USB output report (report ID 0x02) for dual-motor rumble.
pub fn build_dualsense_usb_rumble(left: f32, right: f32) -> [u8; 48] {
    let mut buf = [0u8; 48];
    buf[0] = 0x02;
    buf[1] = 0x03; // Enable rumble emulation
    buf[3] = (right.clamp(0.0, 1.0) * 255.0).round() as u8;
    buf[4] = (left.clamp(0.0, 1.0) * 255.0).round() as u8;
    buf
}

/// Builds a 32-byte Sony DualShock 4 USB output report (report ID 0x05) for dual-motor rumble.
pub fn build_ds4_usb_rumble(left: f32, right: f32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0] = 0x05;
    buf[1] = 0x01; // Enable rumble
    buf[4] = (right.clamp(0.0, 1.0) * 255.0).round() as u8;
    buf[5] = (left.clamp(0.0, 1.0) * 255.0).round() as u8;
    buf
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidControllerKind {
    DualSense,
    DualShock4,
}

pub struct HidControllerHandle {
    pub kind: HidControllerKind,
    device: HidDevice,
}

impl HidControllerHandle {
    pub fn try_open_any() -> Option<Self> {
        let api = HidApi::new().ok()?;
        for info in api.device_list() {
            if info.vendor_id() != SONY_VID {
                continue;
            }
            let kind = match info.product_id() {
                DUALSENSE_PID | DUALSENSE_EDGE_PID => HidControllerKind::DualSense,
                DUALSHOCK4_V1_PID | DUALSHOCK4_V2_PID => HidControllerKind::DualShock4,
                _ => continue,
            };
            if let Ok(device) = info.open_device(&api) {
                let _ = device.set_blocking_mode(false);
                return Some(Self { kind, device });
            }
        }
        None
    }

    pub fn initial_snapshot(&self, event_count: u64) -> ControllerSnapshot {
        ControllerSnapshot {
            is_connected: true,
            name: Some(match self.kind {
                HidControllerKind::DualSense => "DualSense Wireless Controller".to_string(),
                HidControllerKind::DualShock4 => "DualShock 4 Controller".to_string(),
            }),
            family: ControllerFamily::PlayStation,
            transport: Some("USB HID".to_string()),
            event_count,
            share_supported: false,
            ..Default::default()
        }
    }

    pub fn is_sony_hid_present() -> bool {
        let Ok(api) = HidApi::new() else {
            return false;
        };
        let present = api.device_list().any(|info| {
            info.vendor_id() == SONY_VID
                && matches!(
                    info.product_id(),
                    DUALSENSE_PID | DUALSENSE_EDGE_PID | DUALSHOCK4_V1_PID | DUALSHOCK4_V2_PID
                )
        });
        present
    }

    pub fn poll(&self, event_count: u64) -> Result<Option<ControllerSnapshot>, ()> {
        let mut buf = [0u8; 128];
        let mut last_len = 0usize;
        // Drain queued reports to always get the freshest state
        loop {
            match self.device.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => last_len = n,
                Err(_) => return Err(()),
            }
        }
        if last_len == 0 {
            return Ok(None);
        }
        let slice = &buf[..last_len];
        let snap = match self.kind {
            HidControllerKind::DualSense => parse_dualsense_report(slice, event_count),
            HidControllerKind::DualShock4 => parse_dualshock4_report(slice, event_count),
        };
        Ok(snap)
    }

    pub fn set_rumble(&self, left: f32, right: f32) -> bool {
        match self.kind {
            HidControllerKind::DualSense => {
                let report = build_dualsense_usb_rumble(left, right);
                self.device.write(&report).is_ok()
            }
            HidControllerKind::DualShock4 => {
                let report = build_ds4_usb_rumble(left, right);
                self.device.write(&report).is_ok()
            }
        }
    }
}
