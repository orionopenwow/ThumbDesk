use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::engines::action_engine::{ActionEngine, ActionSink};
use crate::engines::cursor_math::{CursorMotionState, ZoomDirection};
use crate::engines::scroll_repeater::HeldScrollRepeater;
use crate::hardware::haptics::HapticsEngine;
use crate::hardware::input_relay::{SnapshotRelayFilter, WindowsControllerHub};
use crate::models::companion::{
    CompanionConnectionState, CompanionEdge, CompanionMessage, CompanionMessageType, CompanionMode,
    CompanionMouseButton, CompanionMousePhase, CompanionPeer, CompanionShortcutPhase,
    SpaceSwitchDirection,
};
use crate::models::control::{
    ControllerControlID, ControllerSnapshot, StickAssignment, StickRoleChoice, StickSide,
};
use crate::models::profile::{
    ApplicationMappingOverrides, ControllerActionMapping, ControllerModifierLayer,
    ControllerProfile, CrossEdgeDirection, CursorConfiguration, ProfileDocument,
};
use crate::models::shortcut::{vk, KeyboardModifier, ShortcutDescriptor};
use crate::models::vibration::{
    ControllerVibration, HapticsPreferences, HapticsStatus, VibrationPhase,
};
use crate::platform::active_app::{frontmost_application, RunningAppInfo};
use crate::platform::win_input::{
    get_cursor_pos, local_restore_point, move_mouse_relative, normalized_edge_position,
    remote_entry_point, send_mouse_button, send_scroll, send_shortcut, send_switch_virtual_desktop,
    set_cursor_pos, should_begin_handoff, virtual_desktop_bounds,
};
use crate::services::codex_provisioner::ensure_fork_shortcut;
use crate::services::companion::CompanionService;
use crate::services::profile_store::ProfileStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDiagnostics {
    pub cursor_state: String,
    pub cursor_message: String,
    pub last_action_status: String,
    pub active_app_id: Option<String>,
    pub active_app_name: Option<String>,
    pub is_routing_to_companion: bool,
}

impl Default for RuntimeDiagnostics {
    fn default() -> Self {
        Self {
            cursor_state: "idle".to_string(),
            cursor_message: "Waiting for stick input".to_string(),
            last_action_status: "Ready".to_string(),
            active_app_id: None,
            active_app_name: None,
            is_routing_to_companion: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatePayload {
    pub document: ProfileDocument,
    pub runtime_enabled: bool,
    pub haptics_preferences: HapticsPreferences,
    pub haptics_status: HapticsStatus,
    pub companion_mode: CompanionMode,
    pub companion_edge: CompanionEdge,
    pub companion_state: CompanionConnectionState,
    pub companion_peers: Vec<CompanionPeer>,
    pub diagnostics: RuntimeDiagnostics,
    pub snapshot: ControllerSnapshot,
}

struct SharedState {
    document: ProfileDocument,
    runtime_enabled: bool,
    diagnostics: RuntimeDiagnostics,
    snapshot: ControllerSnapshot,
    cursor_motion: CursorMotionState,
    action_engine: ActionEngine,
    is_routing_to_companion: bool,
    local_restore_pos: Option<(f64, f64)>,
}

pub struct AppRuntime {
    store: ProfileStore,
    state: Arc<Mutex<SharedState>>,
    hub: WindowsControllerHub,
    haptics: HapticsEngine,
    scroll_repeater: Arc<HeldScrollRepeater>,
    companion: CompanionService,
    running: Arc<AtomicBool>,
}

struct LiveActionSink<'a> {
    is_routing_to_companion: bool,
    companion: &'a CompanionService,
    haptics: &'a HapticsEngine,
    scroll_repeater: &'a HeldScrollRepeater,
    cursor_motion: &'a mut CursorMotionState,
    last_status: &'a mut String,
}

impl<'a> ActionSink for LiveActionSink<'a> {
    fn mouse(&mut self, button: CompanionMouseButton, phase: CompanionMousePhase) {
        if self.is_routing_to_companion {
            let _ = self.companion.send(CompanionMessage::mouse(button, phase));
        } else {
            send_mouse_button(button, phase);
        }
    }

    fn scroll(&mut self, vertical: i32, horizontal: i32) {
        if self.is_routing_to_companion {
            let _ = self.companion.send(CompanionMessage::scroll(vertical, horizontal));
        } else {
            send_scroll(vertical, horizontal);
        }
    }

    fn shortcut(&mut self, shortcut: &ShortcutDescriptor, phase: CompanionShortcutPhase) {
        if self.is_routing_to_companion {
            let _ = self
                .companion
                .send(CompanionMessage::shortcut(shortcut.clone(), phase));
        } else {
            send_shortcut(shortcut, phase);
        }
    }

    fn space_switch(&mut self, direction: SpaceSwitchDirection) {
        if self.is_routing_to_companion {
            let _ = self.companion.send(CompanionMessage::space_switch(direction));
        } else {
            send_switch_virtual_desktop(direction);
        }
    }

    fn cross_edge(&mut self, direction: CrossEdgeDirection) {
        self.cursor_motion.start_cross_edge_sweep(direction);
    }

    fn toggle_cursor_speeds(&mut self) {
        self.cursor_motion.speeds_swapped = !self.cursor_motion.speeds_swapped;
    }

    fn start_scroll_repeat(
        &mut self,
        control: ControllerControlID,
        modifier: Option<ControllerControlID>,
        vertical: i32,
        horizontal: i32,
        delay: f64,
        interval: f64,
    ) {
        self.scroll_repeater
            .start(control, modifier, vertical, horizontal, delay, interval);
    }

    fn stop_scroll_repeat(&mut self, control: ControllerControlID) {
        self.scroll_repeater.stop(control);
    }

    fn stop_all_scroll_repeats(&mut self) {
        self.scroll_repeater.stop_all();
    }

    fn vibrate(&mut self, pattern: ControllerVibration, phase: VibrationPhase) {
        self.haptics.play(pattern, phase);
    }

    fn stop_vibration(&mut self) {
        self.haptics.stop();
    }

    fn report_status(&mut self, status: String) {
        *self.last_status = status;
    }
}

impl AppRuntime {
    pub fn new(store: ProfileStore) -> Arc<Self> {
        let document = store
            .load_or_create()
            .unwrap_or_else(|_| ProfileDocument::default_document());

        let hub = WindowsControllerHub::new();
        let haptics = HapticsEngine::new(Arc::new(hub.clone()));

        let state = Arc::new(Mutex::new(SharedState {
            document,
            runtime_enabled: true,
            diagnostics: RuntimeDiagnostics::default(),
            snapshot: ControllerSnapshot::default(),
            cursor_motion: CursorMotionState::new(),
            action_engine: ActionEngine::new(),
            is_routing_to_companion: false,
            local_restore_pos: None,
        }));

        let state_for_companion = Arc::clone(&state);
        let companion = CompanionService::new(move |msg| {
            Self::handle_companion_message(&state_for_companion, msg);
        });

        let companion_for_scroll = companion.clone();
        let state_for_scroll = Arc::clone(&state);
        let scroll_repeater = Arc::new(HeldScrollRepeater::new(move |v, h| {
            let routing = state_for_scroll.lock().unwrap().is_routing_to_companion;
            if routing {
                let _ = companion_for_scroll.send(CompanionMessage::scroll(v, h));
            } else {
                send_scroll(v, h);
            }
        }));

        Arc::new(Self {
            store,
            state,
            hub,
            haptics,
            scroll_repeater,
            companion,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    fn handle_companion_message(state: &Arc<Mutex<SharedState>>, msg: CompanionMessage) {
        match msg.message_type {
            CompanionMessageType::HandoffStart => {
                if let (Some(edge), Some(norm)) = (msg.edge, msg.normalized_position) {
                    let bounds = virtual_desktop_bounds();
                    let (x, y) = remote_entry_point(edge, norm, bounds);
                    set_cursor_pos(x, y);
                }
            }
            CompanionMessageType::PointerDelta => {
                if let (Some(dx), Some(dy)) = (msg.dx, msg.dy) {
                    move_mouse_relative(dx.round() as i32, dy.round() as i32);
                }
            }
            CompanionMessageType::Mouse => {
                if let (Some(btn), Some(phase)) = (msg.button, msg.phase) {
                    send_mouse_button(btn, phase);
                }
            }
            CompanionMessageType::Scroll => {
                send_scroll(msg.vertical.unwrap_or(0), msg.horizontal.unwrap_or(0));
            }
            CompanionMessageType::Shortcut => {
                if let (Some(sc), Some(phase)) = (msg.shortcut, msg.shortcut_phase) {
                    send_shortcut(&sc, phase);
                }
            }
            CompanionMessageType::SpaceSwitch => {
                if let Some(dir) = msg.space_switch_direction {
                    send_switch_virtual_desktop(dir);
                }
            }
            CompanionMessageType::HandoffBack => {
                let restore = {
                    let mut guard = state.lock().unwrap();
                    guard.is_routing_to_companion = false;
                    guard.diagnostics.is_routing_to_companion = false;
                    guard.local_restore_pos.take()
                };
                if let Some((rx, ry)) = restore {
                    set_cursor_pos(rx, ry);
                }
            }
            CompanionMessageType::Hello => {}
        }
    }

    pub fn start_loop(self: &Arc<Self>, app_handle: AppHandle) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let rt = Arc::clone(self);
        thread::Builder::new()
            .name("vibe-realtime-loop".into())
            .spawn(move || {
                let start_epoch = Instant::now();
                let mut last_tick = Instant::now();
                let mut last_app_check = Instant::now() - Duration::from_secs(1);
                let mut was_connected = false;
                let mut relay_filter = SnapshotRelayFilter::default();

                while rt.running.load(Ordering::Relaxed) {
                    let now_inst = Instant::now();
                    let dt = now_inst.duration_since(last_tick).as_secs_f64();
                    last_tick = now_inst;
                    let now_secs = now_inst.duration_since(start_epoch).as_secs_f64();

                    // Poll active foreground app every 250ms
                    if now_inst.duration_since(last_app_check) >= Duration::from_millis(250) {
                        last_app_check = now_inst;
                        let fg = frontmost_application();
                        let mut guard = rt.state.lock().unwrap();
                        guard.diagnostics.active_app_id =
                            fg.as_ref().map(|a| a.bundle_identifier.clone());
                        guard.diagnostics.active_app_name =
                            fg.as_ref().map(|a| a.display_name.clone());
                    }

                    // Poll hardware
                    let snapshot = rt.hub.poll();
                    if snapshot.is_connected && !was_connected {
                        rt.haptics.play_connection_pulse();
                    }
                    was_connected = snapshot.is_connected;

                    rt.scroll_repeater.update_input(&snapshot);
                    let decision = relay_filter.evaluate(&snapshot, now_inst);

                    let mut emit_snap: Option<ControllerSnapshot> = None;
                    {
                        let mut guard = rt.state.lock().unwrap();
                        guard.snapshot = snapshot.clone();
                        let enabled = guard.runtime_enabled;
                        let profile = Self::active_profile_from_doc(&guard.document).clone();
                        let active_app = guard.diagnostics.active_app_id.clone();

                        // Drop companion routing if disconnected
                        if guard.is_routing_to_companion && !rt.companion.state().is_connected() {
                            guard.is_routing_to_companion = false;
                            guard.diagnostics.is_routing_to_companion = false;
                        }

                        if decision.emit_realtime {
                            guard
                                .cursor_motion
                                .update_input(&snapshot, &profile.cursor, now_secs);
                        }

                        // Actions
                        if decision.emit_action {
                            let routing = guard.is_routing_to_companion;
                            let state_ref = &mut *guard;
                            let mut sink = LiveActionSink {
                                is_routing_to_companion: routing,
                                companion: &rt.companion,
                                haptics: &rt.haptics,
                                scroll_repeater: &rt.scroll_repeater,
                                cursor_motion: &mut state_ref.cursor_motion,
                                last_status: &mut state_ref.diagnostics.last_action_status,
                            };
                            state_ref.action_engine.enabled = enabled;
                            state_ref.action_engine.process(
                                &snapshot,
                                &profile,
                                active_app.as_deref(),
                                now_secs,
                                &mut sink,
                            );
                        }

                        // Zoom gesture check
                        let zoom_enabled = enabled
                            && profile.cursor.zoom_gesture_enabled
                            && snapshot
                                .pressed_controls
                                .contains(&ControllerControlID::ButtonSouth);
                        if let Some(dir) = guard.cursor_motion.zoom_repeater.tick(
                            zoom_enabled,
                            snapshot.left_stick.x,
                            snapshot.left_stick.y,
                            now_secs,
                        ) {
                            let sc = match dir {
                                ZoomDirection::ZoomIn => ShortcutDescriptor::new(
                                    vk::OEM_PLUS,
                                    vec![KeyboardModifier::Control],
                                ),
                                ZoomDirection::ZoomOut => ShortcutDescriptor::new(
                                    vk::OEM_MINUS,
                                    vec![KeyboardModifier::Control],
                                ),
                            };
                            send_shortcut(&sc, CompanionShortcutPhase::Tap);
                        }
                        let suppress_left = guard.cursor_motion.zoom_repeater.is_active();

                        // Cursor tick
                        if !enabled {
                            guard.cursor_motion.reset_transient();
                            guard.diagnostics.cursor_state = "disabled".to_string();
                            guard.diagnostics.cursor_message = "Input is paused".to_string();
                        } else if let Some((dx, dy)) = guard.cursor_motion.tick(
                            &snapshot,
                            &profile.cursor,
                            dt,
                            suppress_left,
                        ) {
                            guard.diagnostics.cursor_state = "moving".to_string();
                            guard.diagnostics.cursor_message =
                                format!("Moving ({:+.1}, {:+.1})", dx, dy);

                            let companion_ready = rt.companion.mode() == CompanionMode::Controller
                                && rt.companion.state().is_connected();

                            if companion_ready && !guard.is_routing_to_companion {
                                let loc = get_cursor_pos();
                                let bounds = virtual_desktop_bounds();
                                let edge = rt.companion.edge();
                                if should_begin_handoff(loc, (dx, dy), edge, bounds) {
                                    guard.local_restore_pos =
                                        Some(local_restore_point(loc, edge, bounds));
                                    guard.is_routing_to_companion = true;
                                    guard.diagnostics.is_routing_to_companion = true;
                                    let norm = normalized_edge_position(loc, edge, bounds);
                                    let _ = rt
                                        .companion
                                        .send(CompanionMessage::handoff_start(edge, norm));
                                    let _ = rt
                                        .companion
                                        .send(CompanionMessage::pointer_delta(dx, dy));
                                } else {
                                    let (ix, iy) = guard.cursor_motion.accumulator.accumulate(dx, dy);
                                    move_mouse_relative(ix, iy);
                                }
                            } else if guard.is_routing_to_companion {
                                let _ = rt
                                    .companion
                                    .send(CompanionMessage::pointer_delta(dx, dy));
                            } else {
                                let (ix, iy) = guard.cursor_motion.accumulator.accumulate(dx, dy);
                                move_mouse_relative(ix, iy);
                            }
                        } else {
                            guard.diagnostics.cursor_state = "idle".to_string();
                            guard.diagnostics.cursor_message = if snapshot.is_connected {
                                "Ready for stick input".to_string()
                            } else {
                                "Waiting for controller".to_string()
                            };
                        }

                        if decision.emit_telemetry {
                            emit_snap = Some(snapshot.clone());
                        }
                    }

                    if let Some(snap) = emit_snap {
                        let _ = app_handle.emit("controller-telemetry", &snap);
                    }

                    thread::sleep(Duration::from_micros(8000)); // ~120 Hz
                }
            })
            .ok();
    }

    pub fn active_profile_from_doc(doc: &ProfileDocument) -> &ControllerProfile {
        doc.profiles
            .iter()
            .find(|p| p.id == doc.active_profile_id)
            .unwrap_or(&doc.profiles[0])
    }

    fn active_profile_mut(doc: &mut ProfileDocument) -> &mut ControllerProfile {
        let idx = doc
            .profiles
            .iter()
            .position(|p| p.id == doc.active_profile_id)
            .unwrap_or(0);
        &mut doc.profiles[idx]
    }

    pub fn state_payload(&self) -> AppStatePayload {
        let guard = self.state.lock().unwrap();
        AppStatePayload {
            document: guard.document.clone(),
            runtime_enabled: guard.runtime_enabled,
            haptics_preferences: self.haptics.preferences(),
            haptics_status: self.haptics.status(),
            companion_mode: self.companion.mode(),
            companion_edge: self.companion.edge(),
            companion_state: self.companion.state(),
            companion_peers: self.companion.peers(),
            diagnostics: guard.diagnostics.clone(),
            snapshot: guard.snapshot.clone(),
        }
    }

    fn persist_doc(&self, doc: &ProfileDocument) -> Result<(), String> {
        self.store.save(doc)
    }

    pub fn set_runtime_enabled(&self, enabled: bool) -> AppStatePayload {
        {
            let mut guard = self.state.lock().unwrap();
            guard.runtime_enabled = enabled;
            if !enabled {
                guard.cursor_motion.reset_transient();
                self.scroll_repeater.stop_all();
                self.haptics.stop();
            }
        }
        self.state_payload()
    }

    pub fn select_profile(&self, profile_id: String) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            if guard.document.profiles.iter().any(|p| p.id == profile_id) {
                guard.document.active_profile_id = profile_id;
                self.persist_doc(&guard.document)?;
            }
        }
        Ok(self.state_payload())
    }

    pub fn update_cursor_config(&self, config: CursorConfiguration) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            Self::active_profile_mut(&mut guard.document).cursor = config;
            self.persist_doc(&guard.document)?;
        }
        Ok(self.state_payload())
    }

    pub fn assign_stick_role(
        &self,
        side: StickSide,
        role: StickRoleChoice,
    ) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let cursor = &mut Self::active_profile_mut(&mut guard.document).cursor;
            let this_assign = StickAssignment::from_side(side);
            let other_assign = StickAssignment::from_side(side.opposite());
            match role {
                StickRoleChoice::Primary => {
                    if cursor.precision_stick == this_assign {
                        cursor.precision_stick = if cursor.primary_stick == other_assign {
                            other_assign
                        } else {
                            StickAssignment::Off
                        };
                    }
                    cursor.primary_stick = this_assign;
                }
                StickRoleChoice::Precision => {
                    if cursor.primary_stick == this_assign {
                        cursor.primary_stick = if cursor.precision_stick == other_assign {
                            other_assign
                        } else {
                            StickAssignment::Off
                        };
                    }
                    cursor.precision_stick = this_assign;
                }
                StickRoleChoice::Off => {
                    if cursor.primary_stick == this_assign {
                        cursor.primary_stick = StickAssignment::Off;
                    }
                    if cursor.precision_stick == this_assign {
                        cursor.precision_stick = StickAssignment::Off;
                    }
                }
            }
            self.persist_doc(&guard.document)?;
        }
        Ok(self.state_payload())
    }

    pub fn save_mapping(
        &self,
        control: ControllerControlID,
        modifier_control: Option<ControllerControlID>,
        app_bundle_id: Option<String>,
        mapping: ControllerActionMapping,
    ) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let profile = Self::active_profile_mut(&mut guard.document);
            if let Some(app_id) = app_bundle_id {
                if let Some(app) = profile
                    .application_mappings
                    .iter_mut()
                    .find(|a| a.bundle_identifier.eq_ignore_ascii_case(&app_id))
                {
                    if let Some(mod_ctrl) = modifier_control {
                        if let Some(layer) = app
                            .modifier_layers
                            .iter_mut()
                            .find(|l| l.modifier_control == mod_ctrl)
                        {
                            layer.mappings.insert(control, mapping);
                        } else {
                            let mut m = std::collections::BTreeMap::new();
                            m.insert(control, mapping);
                            app.modifier_layers.push(ControllerModifierLayer {
                                modifier_control: mod_ctrl,
                                mappings: m,
                            });
                        }
                    } else {
                        app.mappings.insert(control, mapping);
                    }
                }
            } else if let Some(mod_ctrl) = modifier_control {
                if let Some(layer) = profile
                    .modifier_layers
                    .iter_mut()
                    .find(|l| l.modifier_control == mod_ctrl)
                {
                    layer.mappings.insert(control, mapping);
                } else {
                    let mut m = std::collections::BTreeMap::new();
                    m.insert(control, mapping);
                    profile.modifier_layers.push(ControllerModifierLayer {
                        modifier_control: mod_ctrl,
                        mappings: m,
                    });
                }
            } else {
                profile.mappings.insert(control, mapping);
            }
            self.persist_doc(&guard.document)?;
        }
        Ok(self.state_payload())
    }

    pub fn clear_mapping_override(
        &self,
        control: ControllerControlID,
        modifier_control: Option<ControllerControlID>,
        app_bundle_id: Option<String>,
    ) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let profile = Self::active_profile_mut(&mut guard.document);
            if let Some(app_id) = app_bundle_id {
                if let Some(app) = profile
                    .application_mappings
                    .iter_mut()
                    .find(|a| a.bundle_identifier.eq_ignore_ascii_case(&app_id))
                {
                    if let Some(mod_ctrl) = modifier_control {
                        if let Some(layer) = app
                            .modifier_layers
                            .iter_mut()
                            .find(|l| l.modifier_control == mod_ctrl)
                        {
                            layer.mappings.remove(&control);
                        }
                    } else {
                        app.mappings.remove(&control);
                    }
                }
            } else if let Some(mod_ctrl) = modifier_control {
                if let Some(layer) = profile
                    .modifier_layers
                    .iter_mut()
                    .find(|l| l.modifier_control == mod_ctrl)
                {
                    layer.mappings.remove(&control);
                }
            }
            self.persist_doc(&guard.document)?;
        }
        Ok(self.state_payload())
    }

    pub fn add_modifier_layer(
        &self,
        modifier_control: ControllerControlID,
    ) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let profile = Self::active_profile_mut(&mut guard.document);
            if !profile
                .modifier_layers
                .iter()
                .any(|l| l.modifier_control == modifier_control)
            {
                profile.modifier_layers.push(ControllerModifierLayer {
                    modifier_control,
                    mappings: Default::default(),
                });
                self.persist_doc(&guard.document)?;
            }
        }
        Ok(self.state_payload())
    }

    pub fn remove_modifier_layer(
        &self,
        modifier_control: ControllerControlID,
    ) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let profile = Self::active_profile_mut(&mut guard.document);
            profile
                .modifier_layers
                .retain(|l| l.modifier_control != modifier_control);
            for app in &mut profile.application_mappings {
                app.modifier_layers
                    .retain(|l| l.modifier_control != modifier_control);
            }
            self.persist_doc(&guard.document)?;
        }
        Ok(self.state_payload())
    }

    pub fn add_app_scope(
        &self,
        bundle_identifier: String,
        display_name: String,
    ) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let profile = Self::active_profile_mut(&mut guard.document);
            if !profile
                .application_mappings
                .iter()
                .any(|a| a.bundle_identifier.eq_ignore_ascii_case(&bundle_identifier))
            {
                profile.application_mappings.push(ApplicationMappingOverrides {
                    bundle_identifier,
                    display_name,
                    mappings: Default::default(),
                    modifier_layers: vec![],
                });
                self.persist_doc(&guard.document)?;
            }
        }
        Ok(self.state_payload())
    }

    pub fn install_codex_starter(&self) -> Result<AppStatePayload, String> {
        if let Some(home) = dirs::home_dir() {
            let _ = ensure_fork_shortcut(&home);
        }
        {
            let mut guard = self.state.lock().unwrap();
            let profile = Self::active_profile_mut(&mut guard.document);
            let starter = ControllerProfile::codex_starter_application_mappings();
            if let Some(existing) = profile.application_mappings.iter_mut().find(|a| {
                a.bundle_identifier == ApplicationMappingOverrides::CODEX_BUNDLE_IDENTIFIER
            }) {
                *existing = starter;
            } else {
                profile.application_mappings.push(starter);
            }
            self.persist_doc(&guard.document)?;
        }
        Ok(self.state_payload())
    }

    pub fn reset_app_scope(&self, bundle_identifier: String) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let profile = Self::active_profile_mut(&mut guard.document);
            if let Some(app) = profile
                .application_mappings
                .iter_mut()
                .find(|a| a.bundle_identifier.eq_ignore_ascii_case(&bundle_identifier))
            {
                app.mappings.clear();
                app.modifier_layers.clear();
                self.persist_doc(&guard.document)?;
            }
        }
        Ok(self.state_payload())
    }

    pub fn remove_app_scope(&self, bundle_identifier: String) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let profile = Self::active_profile_mut(&mut guard.document);
            profile
                .application_mappings
                .retain(|a| !a.bundle_identifier.eq_ignore_ascii_case(&bundle_identifier));
            self.persist_doc(&guard.document)?;
        }
        Ok(self.state_payload())
    }

    pub fn set_haptics_preferences(&self, prefs: HapticsPreferences) -> AppStatePayload {
        self.haptics.set_preferences(prefs);
        self.state_payload()
    }

    pub fn preview_vibration(&self, pattern: ControllerVibration) {
        self.haptics.preview(pattern);
    }

    pub fn apply_suggested_vibrations(&self) -> Result<AppStatePayload, String> {
        let mut prefs = self.haptics.preferences();
        prefs.enabled = true;
        self.haptics.set_preferences(prefs);
        {
            let mut guard = self.state.lock().unwrap();
            Self::active_profile_mut(&mut guard.document).apply_suggested_vibrations();
            self.persist_doc(&guard.document)?;
        }
        Ok(self.state_payload())
    }

    pub fn set_companion_mode(&self, mode: CompanionMode) -> AppStatePayload {
        self.companion.set_mode(mode);
        {
            let mut guard = self.state.lock().unwrap();
            guard.is_routing_to_companion = false;
            guard.diagnostics.is_routing_to_companion = false;
        }
        self.state_payload()
    }

    pub fn set_companion_edge(&self, edge: CompanionEdge) -> AppStatePayload {
        self.companion.set_edge(edge);
        self.state_payload()
    }

    pub fn connect_companion(&self, peer_id: Option<String>) -> AppStatePayload {
        self.companion.connect_to_peer(peer_id.as_deref());
        self.state_payload()
    }

    pub fn disconnect_companion(&self) -> AppStatePayload {
        self.companion.disconnect();
        {
            let mut guard = self.state.lock().unwrap();
            guard.is_routing_to_companion = false;
            guard.diagnostics.is_routing_to_companion = false;
        }
        self.state_payload()
    }

    pub fn force_companion_handoff(&self) -> AppStatePayload {
        if self.companion.state().is_connected() {
            let edge = self.companion.edge();
            let _ = self.companion.send(CompanionMessage::handoff_start(edge, 0.5));
            let mut guard = self.state.lock().unwrap();
            guard.is_routing_to_companion = true;
            guard.diagnostics.is_routing_to_companion = true;
        }
        self.state_payload()
    }

    pub fn return_companion_local(&self) -> AppStatePayload {
        let _ = self.companion.send(CompanionMessage::handoff_back());
        {
            let mut guard = self.state.lock().unwrap();
            guard.is_routing_to_companion = false;
            guard.diagnostics.is_routing_to_companion = false;
        }
        self.state_payload()
    }

    pub fn trigger_diagnostic_sweep(&self) {
        let mut guard = self.state.lock().unwrap();
        guard
            .cursor_motion
            .start_cross_edge_sweep(CrossEdgeDirection::Right);
        guard.diagnostics.last_action_status = "Diagnostic Cross-Edge Sweep".to_string();
    }

    pub fn trigger_diagnostic_left_click(&self) {
        send_mouse_button(CompanionMouseButton::Left, CompanionMousePhase::Click);
        let mut guard = self.state.lock().unwrap();
        guard.diagnostics.last_action_status = "Diagnostic Left Click".to_string();
    }

    pub fn import_profile_json(&self, raw_json: String) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let merged = ProfileStore::import_profile_str(&raw_json, &guard.document)?;
            self.persist_doc(&merged)?;
            guard.document = merged;
        }
        Ok(self.state_payload())
    }

    pub fn export_active_profile_json(&self) -> Result<String, String> {
        let guard = self.state.lock().unwrap();
        let active = Self::active_profile_from_doc(&guard.document);
        serde_json::to_string_pretty(active).map_err(|e| e.to_string())
    }

    pub fn reset_active_profile_to_defaults(&self) -> Result<AppStatePayload, String> {
        {
            let mut guard = self.state.lock().unwrap();
            let active = Self::active_profile_mut(&mut guard.document);
            let keep_id = active.id.clone();
            let keep_name = active.name.clone();
            *active = ControllerProfile::gabes_defaults();
            active.id = keep_id;
            active.name = keep_name;
            self.persist_doc(&guard.document)?;
        }
        Ok(self.state_payload())
    }

    pub fn running_apps(&self) -> Vec<RunningAppInfo> {
        crate::platform::active_app::list_running_apps()
    }
}
