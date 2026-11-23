use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::engines::action_engine::TRIGGER_ACTUATION_THRESHOLD;
use crate::hardware::hid_controller::HidControllerHandle;
use crate::hardware::xinput::{poll_xinput, set_xinput_rumble};
use crate::models::control::{ControllerControlID, ControllerSnapshot};
use crate::models::vibration::HapticsStatus;

use super::haptics::RumbleSink;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerActionState {
    pub is_connected: bool,
    pub pressed_controls: BTreeSet<ControllerControlID>,
    pub left_trigger_actuated: bool,
    pub right_trigger_actuated: bool,
}

impl ControllerActionState {
    pub fn from_snapshot(s: &ControllerSnapshot) -> Self {
        Self {
            is_connected: s.is_connected,
            pressed_controls: s.pressed_controls.clone(),
            left_trigger_actuated: s.value_for(ControllerControlID::LeftTrigger)
                >= TRIGGER_ACTUATION_THRESHOLD,
            right_trigger_actuated: s.value_for(ControllerControlID::RightTrigger)
                >= TRIGGER_ACTUATION_THRESHOLD,
        }
    }
}

pub const TELEMETRY_INTERVAL_SECS: f64 = 1.0 / 15.0;

#[derive(Debug)]
pub struct SnapshotRelayFilter {
    last_snapshot: Option<ControllerSnapshot>,
    last_action_state: Option<ControllerActionState>,
    last_telemetry_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayDecision {
    pub emit_realtime: bool,
    pub emit_action: bool,
    pub emit_telemetry: bool,
}

impl Default for SnapshotRelayFilter {
    fn default() -> Self {
        Self {
            last_snapshot: None,
            last_action_state: None,
            last_telemetry_at: None,
        }
    }
}

impl SnapshotRelayFilter {
    pub fn evaluate(&mut self, snapshot: &ControllerSnapshot, now: Instant) -> RelayDecision {
        let same_payload = self
            .last_snapshot
            .as_ref()
            .map(|prev| prev.has_same_input_payload(snapshot))
            .unwrap_or(false);

        let action_state = ControllerActionState::from_snapshot(snapshot);
        let action_changed = self
            .last_action_state
            .as_ref()
            .map(|prev| *prev != action_state)
            .unwrap_or(true);

        let telemetry_due = self
            .last_telemetry_at
            .map(|t| now.duration_since(t).as_secs_f64() >= TELEMETRY_INTERVAL_SECS)
            .unwrap_or(true);

        let emit_realtime = !same_payload || action_changed;
        let emit_action = action_changed;
        let emit_telemetry = action_changed || (!same_payload && telemetry_due);

        if emit_realtime {
            self.last_snapshot = Some(snapshot.clone());
        }
        if emit_action {
            self.last_action_state = Some(action_state);
        }
        if emit_telemetry {
            self.last_telemetry_at = Some(now);
        }

        RelayDecision {
            emit_realtime,
            emit_action,
            emit_telemetry,
        }
    }
}

#[derive(Default)]
struct ActiveHardware {
    hid: Option<HidControllerHandle>,
    xinput_slot: Option<u32>,
    last_hid_snapshot: Option<ControllerSnapshot>,
    last_hid_scan: Option<Instant>,
    event_count: u64,
}

/// Manages unified Windows controller polling (HID Direct PlayStation > XInput Xbox)
/// and implements `RumbleSink` for active hardware.
#[derive(Clone, Default)]
pub struct WindowsControllerHub {
    inner: Arc<Mutex<ActiveHardware>>,
}

impl WindowsControllerHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn poll(&self) -> ControllerSnapshot {
        let mut guard = self.inner.lock().unwrap();
        let next_event = guard.event_count.wrapping_add(1);
        let now = Instant::now();
        let should_scan_hid = guard
            .last_hid_scan
            .map(|t| now.duration_since(t).as_secs_f64() >= 0.5)
            .unwrap_or(true);

        // 1. Direct HID (PlayStation DualSense / DualShock 4) has highest precedence
        if should_scan_hid {
            guard.last_hid_scan = Some(now);
            if guard.hid.is_none() {
                if let Some(handle) = HidControllerHandle::try_open_any() {
                    let init_snap = handle.initial_snapshot(next_event);
                    guard.event_count = next_event;
                    guard.xinput_slot = None;
                    guard.last_hid_snapshot = Some(init_snap);
                    guard.hid = Some(handle);
                }
            } else if !HidControllerHandle::is_sony_hid_present() {
                guard.hid = None;
                guard.last_hid_snapshot = None;
            }
        }

        if let Some(hid) = &guard.hid {
            match hid.poll(next_event) {
                Ok(Some(snap)) => {
                    guard.event_count = next_event;
                    guard.xinput_slot = None;
                    guard.last_hid_snapshot = Some(snap.clone());
                    return snap;
                }
                Ok(None) => {
                    if let Some(prev) = &guard.last_hid_snapshot {
                        return prev.clone();
                    }
                }
                Err(()) => {
                    guard.hid = None;
                    guard.last_hid_snapshot = None;
                }
            }
        }

        // 2. XInput (Xbox 360 / Xbox One / Xbox Series X|S)
        if let Some(reading) = poll_xinput(next_event) {
            guard.event_count = next_event;
            guard.xinput_slot = Some(reading.slot);
            return reading.snapshot;
        }

        guard.xinput_slot = None;
        ControllerSnapshot::default()
    }
}

impl RumbleSink for WindowsControllerHub {
    fn set_motors(&self, left: f32, right: f32) -> bool {
        let guard = self.inner.lock().unwrap();
        if let Some(hid) = &guard.hid {
            return hid.set_rumble(left, right);
        }
        if let Some(slot) = guard.xinput_slot {
            return set_xinput_rumble(slot, left, right);
        }
        false
    }

    fn status(&self) -> HapticsStatus {
        let guard = self.inner.lock().unwrap();
        if guard.hid.is_some() {
            return HapticsStatus {
                can_play: true,
                backend: "hid".to_string(),
                message: "Ready via Direct USB HID (PlayStation).".to_string(),
            };
        }
        if let Some(slot) = guard.xinput_slot {
            return HapticsStatus {
                can_play: true,
                backend: "xinput".to_string(),
                message: format!("Ready via XInput dual-motor rumble (Slot {}).", slot + 1),
            };
        }
        HapticsStatus::default()
    }
}
