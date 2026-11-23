use std::collections::{BTreeMap, BTreeSet};

use crate::engines::cursor_math::{zoom_gesture_sample, ZOOM_GESTURE_ACTIVATION_THRESHOLD};
use crate::models::companion::{
    CompanionMouseButton, CompanionMousePhase, CompanionShortcutPhase, SpaceSwitchDirection,
};
use crate::models::control::{ControllerControlID, ControllerSnapshot};
use crate::models::profile::{
    ActionType, ControllerActionMapping, ControllerProfile, CrossEdgeDirection, TriggerMode,
};
use crate::models::shortcut::ShortcutDescriptor;
use crate::models::vibration::{ControllerVibration, VibrationPhase};

pub const TRIGGER_ACTUATION_THRESHOLD: f64 = 0.18;
pub const DUPLICATE_TAP_INTERVAL: f64 = 0.08;

/// Abstraction for dispatching side effects from ActionEngine (Win32, Companion, Haptics, or test spy).
pub trait ActionSink: Send {
    fn mouse(&mut self, button: CompanionMouseButton, phase: CompanionMousePhase);
    fn scroll(&mut self, vertical: i32, horizontal: i32);
    fn shortcut(&mut self, shortcut: &ShortcutDescriptor, phase: CompanionShortcutPhase);
    fn space_switch(&mut self, direction: SpaceSwitchDirection);
    fn cross_edge(&mut self, direction: CrossEdgeDirection);
    fn toggle_cursor_speeds(&mut self);
    fn start_scroll_repeat(
        &mut self,
        control: ControllerControlID,
        modifier: Option<ControllerControlID>,
        vertical: i32,
        horizontal: i32,
        delay: f64,
        interval: f64,
    );
    fn stop_scroll_repeat(&mut self, control: ControllerControlID);
    fn stop_all_scroll_repeats(&mut self);
    fn vibrate(&mut self, pattern: ControllerVibration, phase: VibrationPhase);
    fn stop_vibration(&mut self);
    fn report_status(&mut self, status: String);
}

#[derive(Debug, Clone)]
struct ActiveControlState {
    mapping: ControllerActionMapping,
    source_modifier: Option<ControllerControlID>,
    is_holding_shortcut: bool,
    is_dragging: bool,
    is_repeating_scroll: bool,
    is_toggled_on: bool,
}

#[derive(Debug, Clone)]
struct RecentTap {
    mapping: ControllerActionMapping,
    source_modifier: Option<ControllerControlID>,
    timestamp: f64,
}

#[derive(Debug, Default)]
pub struct ActionEngine {
    previous_pressed: BTreeSet<ControllerControlID>,
    armed_modifiers: BTreeSet<ControllerControlID>,
    consumed_modifiers: BTreeSet<ControllerControlID>,
    modifier_press_order: Vec<ControllerControlID>,
    active_states: BTreeMap<ControllerControlID, ActiveControlState>,
    recent_taps: BTreeMap<ControllerControlID, RecentTap>,
    pub enabled: bool,
    pub suspended: bool,
}

impl ActionEngine {
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    pub fn actuated_controls(snapshot: &ControllerSnapshot) -> BTreeSet<ControllerControlID> {
        let mut set = BTreeSet::new();
        if !snapshot.is_connected {
            return set;
        }
        for &control in ControllerControlID::MAPPING_CONTROLS {
            let actuated = match control {
                ControllerControlID::LeftTrigger | ControllerControlID::RightTrigger => {
                    snapshot.value_for(control) >= TRIGGER_ACTUATION_THRESHOLD
                }
                _ => snapshot.pressed_controls.contains(&control),
            };
            if actuated {
                set.insert(control);
            }
        }
        set
    }

    pub fn active_modifier_control(
        &self,
        actuated: &BTreeSet<ControllerControlID>,
    ) -> Option<ControllerControlID> {
        self.modifier_press_order
            .iter()
            .rev()
            .copied()
            .find(|c| self.armed_modifiers.contains(c) && actuated.contains(c))
    }

    pub fn current_held_modifier(&self) -> Option<ControllerControlID> {
        self.modifier_press_order
            .iter()
            .rev()
            .copied()
            .find(|c| self.armed_modifiers.contains(c))
    }

    pub fn reset_action_state(&mut self, sink: &mut dyn ActionSink) {
        sink.stop_all_scroll_repeats();
        let keys: Vec<ControllerControlID> = self.active_states.keys().copied().collect();
        for control in keys {
            self.finish_active_state(control, false, sink);
        }
        sink.stop_vibration();
        self.armed_modifiers.clear();
        self.consumed_modifiers.clear();
        self.modifier_press_order.clear();
        self.previous_pressed.clear();
        self.recent_taps.clear();
    }

    pub fn process(
        &mut self,
        snapshot: &ControllerSnapshot,
        profile: &ControllerProfile,
        active_app_id: Option<&str>,
        now: f64,
        sink: &mut dyn ActionSink,
    ) {
        let actuated = Self::actuated_controls(snapshot);
        if !self.enabled || self.suspended || !snapshot.is_connected {
            if !self.active_states.is_empty() || !self.armed_modifiers.is_empty() {
                self.reset_action_state(sink);
            }
            self.previous_pressed = actuated;
            return;
        }

        // 1. Releases first
        let newly_released: Vec<ControllerControlID> = self
            .previous_pressed
            .difference(&actuated)
            .copied()
            .collect();
        for control in newly_released {
            if self.armed_modifiers.contains(&control) {
                self.release_modifier(control, profile, active_app_id, now, sink);
            } else {
                self.handle_release(control, sink);
            }
        }

        // 2. Modifier presses second
        let newly_pressed: BTreeSet<ControllerControlID> = actuated
            .difference(&self.previous_pressed)
            .copied()
            .collect();

        let mut modifier_controls: Vec<ControllerControlID> = profile
            .modifier_layers
            .iter()
            .map(|l| l.modifier_control)
            .collect();
        if profile.cursor.zoom_gesture_enabled
            && !modifier_controls.contains(&ControllerControlID::ButtonSouth)
        {
            modifier_controls.push(ControllerControlID::ButtonSouth);
        }

        let mut handled_presses = BTreeSet::new();
        for mod_control in modifier_controls {
            if newly_pressed.contains(&mod_control) {
                if let Some(active_mod) = self.active_modifier_control(&actuated) {
                    if active_mod != mod_control {
                        self.press_control(
                            mod_control,
                            Some(active_mod),
                            profile,
                            active_app_id,
                            now,
                            sink,
                        );
                        handled_presses.insert(mod_control);
                        continue;
                    }
                }
                self.armed_modifiers.insert(mod_control);
                self.modifier_press_order.retain(|c| *c != mod_control);
                self.modifier_press_order.push(mod_control);
                handled_presses.insert(mod_control);
            }
        }

        // 3. Zoom gesture consumption check
        if profile.cursor.zoom_gesture_enabled
            && actuated.contains(&ControllerControlID::ButtonSouth)
            && zoom_gesture_sample(
                snapshot.left_stick.x,
                snapshot.left_stick.y,
                ZOOM_GESTURE_ACTIVATION_THRESHOLD,
            )
            .is_some()
        {
            self.consumed_modifiers
                .insert(ControllerControlID::ButtonSouth);
        }

        // 4. Remaining control presses
        let remaining: Vec<ControllerControlID> = newly_pressed
            .difference(&handled_presses)
            .copied()
            .collect();
        for control in remaining {
            let active_mod = self.active_modifier_control(&actuated);
            self.press_control(control, active_mod, profile, active_app_id, now, sink);
        }

        self.previous_pressed = actuated;
    }

    fn press_control(
        &mut self,
        control: ControllerControlID,
        modifier: Option<ControllerControlID>,
        profile: &ControllerProfile,
        active_app_id: Option<&str>,
        now: f64,
        sink: &mut dyn ActionSink,
    ) {
        if let Some(m) = modifier {
            self.consumed_modifiers.insert(m);
        }

        let mapping = profile.effective_mapping(control, modifier, active_app_id);
        if mapping.action_type == ActionType::None {
            return;
        }

        // Duplicate tap debounce
        if mapping.trigger_mode == TriggerMode::Tap {
            if let Some(prev) = self.recent_taps.get(&control) {
                if prev.mapping == mapping
                    && prev.source_modifier == modifier
                    && (now - prev.timestamp) < DUPLICATE_TAP_INTERVAL
                {
                    self.recent_taps.insert(
                        control,
                        RecentTap {
                            mapping,
                            source_modifier: modifier,
                            timestamp: now,
                        },
                    );
                    return;
                }
            }
            self.recent_taps.insert(
                control,
                RecentTap {
                    mapping: mapping.clone(),
                    source_modifier: modifier,
                    timestamp: now,
                },
            );
        }

        match mapping.trigger_mode {
            TriggerMode::Tap => {
                self.fire_discrete_action(&mapping, sink);
                sink.vibrate(mapping.vibration, VibrationPhase::Press);
            }
            TriggerMode::HoldWhilePressed => match mapping.action_type {
                ActionType::KeyboardShortcut => {
                    if let Some(shortcut) = &mapping.shortcut {
                        sink.shortcut(shortcut, CompanionShortcutPhase::Down);
                        sink.vibrate(mapping.vibration, VibrationPhase::Press);
                        sink.report_status(format!("Holding {}", shortcut.display_string()));
                        self.active_states.insert(
                            control,
                            ActiveControlState {
                                mapping,
                                source_modifier: modifier,
                                is_holding_shortcut: true,
                                is_dragging: false,
                                is_repeating_scroll: false,
                                is_toggled_on: false,
                            },
                        );
                    }
                }
                ActionType::LeftMouseHold => {
                    sink.mouse(CompanionMouseButton::Left, CompanionMousePhase::Down);
                    sink.vibrate(mapping.vibration, VibrationPhase::Press);
                    sink.report_status("Dragging (left mouse hold)".to_string());
                    self.active_states.insert(
                        control,
                        ActiveControlState {
                            mapping,
                            source_modifier: modifier,
                            is_holding_shortcut: false,
                            is_dragging: true,
                            is_repeating_scroll: false,
                            is_toggled_on: false,
                        },
                    );
                }
                _ => {
                    self.fire_discrete_action(&mapping, sink);
                    sink.vibrate(mapping.vibration, VibrationPhase::Press);
                }
            },
            TriggerMode::RepeatWhileHeld => {
                if let Some((v, h)) = mapping.action_type.scroll_step() {
                    sink.scroll(v, h);
                    sink.start_scroll_repeat(
                        control,
                        modifier,
                        v,
                        h,
                        mapping.repeat_delay,
                        mapping.repeat_interval,
                    );
                    sink.vibrate(mapping.vibration, VibrationPhase::Press);
                    sink.report_status(mapping.summary());
                    self.active_states.insert(
                        control,
                        ActiveControlState {
                            mapping,
                            source_modifier: modifier,
                            is_holding_shortcut: false,
                            is_dragging: false,
                            is_repeating_scroll: true,
                            is_toggled_on: false,
                        },
                    );
                } else {
                    self.fire_discrete_action(&mapping, sink);
                    sink.vibrate(mapping.vibration, VibrationPhase::Press);
                }
            }
            TriggerMode::Toggle => {
                if let Some(existing) = self.active_states.get(&control) {
                    if existing.is_toggled_on {
                        self.finish_active_state(control, true, sink);
                        return;
                    }
                }
                match mapping.action_type {
                    ActionType::LeftMouseHold => {
                        sink.mouse(CompanionMouseButton::Left, CompanionMousePhase::Down);
                        sink.vibrate(mapping.vibration, VibrationPhase::Press);
                        sink.report_status("Drag toggled on".to_string());
                        self.active_states.insert(
                            control,
                            ActiveControlState {
                                mapping,
                                source_modifier: modifier,
                                is_holding_shortcut: false,
                                is_dragging: true,
                                is_repeating_scroll: false,
                                is_toggled_on: true,
                            },
                        );
                    }
                    ActionType::KeyboardShortcut => {
                        if let Some(shortcut) = &mapping.shortcut {
                            sink.shortcut(shortcut, CompanionShortcutPhase::Down);
                            sink.vibrate(mapping.vibration, VibrationPhase::Press);
                            sink.report_status(format!("Toggled {} on", shortcut.display_string()));
                            self.active_states.insert(
                                control,
                                ActiveControlState {
                                    mapping,
                                    source_modifier: modifier,
                                    is_holding_shortcut: true,
                                    is_dragging: false,
                                    is_repeating_scroll: false,
                                    is_toggled_on: true,
                                },
                            );
                        }
                    }
                    _ => {
                        self.fire_discrete_action(&mapping, sink);
                        sink.vibrate(mapping.vibration, VibrationPhase::Press);
                    }
                }
            }
        }
    }

    fn release_modifier(
        &mut self,
        control: ControllerControlID,
        profile: &ControllerProfile,
        active_app_id: Option<&str>,
        _now: f64,
        sink: &mut dyn ActionSink,
    ) {
        self.armed_modifiers.remove(&control);
        self.modifier_press_order.retain(|c| *c != control);
        let was_consumed = self.consumed_modifiers.remove(&control);

        if was_consumed {
            let children: Vec<ControllerControlID> = self
                .active_states
                .iter()
                .filter(|(_, st)| st.source_modifier == Some(control) && !st.is_toggled_on)
                .map(|(&k, _)| k)
                .collect();
            for child in children {
                self.finish_active_state(child, true, sink);
            }
        } else {
            let base = profile.effective_mapping(control, None, active_app_id);
            if base.action_type == ActionType::None {
                return;
            }
            if base.action_type == ActionType::LeftMouseHold {
                sink.mouse(CompanionMouseButton::Left, CompanionMousePhase::Click);
                sink.report_status("Left Click".to_string());
            } else {
                self.fire_discrete_action(&base, sink);
            }
            sink.vibrate(base.vibration, VibrationPhase::Press);
        }
    }

    fn handle_release(&mut self, control: ControllerControlID, sink: &mut dyn ActionSink) {
        if let Some(state) = self.active_states.get(&control) {
            if !state.is_toggled_on {
                self.finish_active_state(control, true, sink);
            }
        }
    }

    fn finish_active_state(
        &mut self,
        control: ControllerControlID,
        with_feedback: bool,
        sink: &mut dyn ActionSink,
    ) {
        let Some(state) = self.active_states.remove(&control) else {
            return;
        };
        if state.is_holding_shortcut {
            if let Some(shortcut) = &state.mapping.shortcut {
                sink.shortcut(shortcut, CompanionShortcutPhase::Up);
            }
        }
        if state.is_dragging {
            sink.mouse(CompanionMouseButton::Left, CompanionMousePhase::Up);
        }
        if state.is_repeating_scroll {
            sink.stop_scroll_repeat(control);
        }
        if with_feedback {
            sink.vibrate(state.mapping.vibration, VibrationPhase::Release);
        }
    }

    fn fire_discrete_action(&self, mapping: &ControllerActionMapping, sink: &mut dyn ActionSink) {
        match mapping.action_type {
            ActionType::None => return,
            ActionType::KeyboardShortcut => {
                if let Some(shortcut) = &mapping.shortcut {
                    sink.shortcut(shortcut, CompanionShortcutPhase::Tap);
                }
            }
            ActionType::LeftClick => {
                sink.mouse(CompanionMouseButton::Left, CompanionMousePhase::Click);
            }
            ActionType::RightClick => {
                sink.mouse(CompanionMouseButton::Right, CompanionMousePhase::Click);
            }
            ActionType::MiddleClick => {
                sink.mouse(CompanionMouseButton::Middle, CompanionMousePhase::Click);
            }
            ActionType::DoubleClick => {
                sink.mouse(CompanionMouseButton::Left, CompanionMousePhase::DoubleClick);
            }
            ActionType::LeftMouseHold => {
                sink.mouse(CompanionMouseButton::Left, CompanionMousePhase::Click);
            }
            ActionType::ScrollUp => sink.scroll(-1, 0),
            ActionType::ScrollDown => sink.scroll(1, 0),
            ActionType::ScrollLeft => sink.scroll(0, -1),
            ActionType::ScrollRight => sink.scroll(0, 1),
            ActionType::SwitchSpaceLeft => sink.space_switch(SpaceSwitchDirection::Left),
            ActionType::SwitchSpaceRight => sink.space_switch(SpaceSwitchDirection::Right),
            ActionType::CrossEdgeLeft => sink.cross_edge(CrossEdgeDirection::Left),
            ActionType::CrossEdgeRight => sink.cross_edge(CrossEdgeDirection::Right),
            ActionType::CrossEdgeUp => sink.cross_edge(CrossEdgeDirection::Up),
            ActionType::CrossEdgeDown => sink.cross_edge(CrossEdgeDirection::Down),
            ActionType::ToggleCursorSpeeds => sink.toggle_cursor_speeds(),
        }
        sink.report_status(mapping.summary());
    }
}
