pub mod commands;
pub mod engines;
pub mod hardware;
pub mod models;
pub mod platform;
pub mod runtime;
pub mod services;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

pub fn run() {
    let runtime = runtime::AppRuntime::new(services::profile_store::ProfileStore::default_store());
    let runtime_for_setup = std::sync::Arc::clone(&runtime);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(runtime)
        .setup(move |app| {
            runtime_for_setup.start_loop(app.handle().clone());

            let logo_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))
                .ok()
                .or_else(|| app.default_window_icon().cloned());

            if let (Some(win), Some(icon)) = (app.get_webview_window("main"), logo_icon.clone()) {
                let _ = win.set_icon(icon);
            }

            let show_item = MenuItem::with_id(app, "show", "Show ThumbDesk", true, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "Toggle Input Enabled", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &toggle_item, &quit_item])?;

            let rt_menu = std::sync::Arc::clone(&runtime_for_setup);
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("ThumbDesk");
            if let Some(icon) = logo_icon {
                tray_builder = tray_builder.icon(icon);
            }
            let _tray = tray_builder
                .on_menu_event(move |app_handle, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app_handle.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "toggle" => {
                        let cur = rt_menu.state_payload().runtime_enabled;
                        let _ = rt_menu.set_runtime_enabled(!cur);
                    }
                    "quit" => {
                        app_handle.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(win) = tray.app_handle().get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::set_runtime_enabled,
            commands::select_profile,
            commands::update_cursor_config,
            commands::assign_stick_role,
            commands::save_mapping,
            commands::clear_mapping_override,
            commands::add_modifier_layer,
            commands::remove_modifier_layer,
            commands::list_running_apps,
            commands::add_app_scope,
            commands::install_codex_starter,
            commands::reset_app_scope,
            commands::remove_app_scope,
            commands::set_haptics_preferences,
            commands::preview_vibration,
            commands::apply_suggested_vibrations,
            commands::set_companion_mode,
            commands::set_companion_edge,
            commands::connect_companion,
            commands::disconnect_companion,
            commands::force_companion_handoff,
            commands::return_companion_local,
            commands::trigger_diagnostic_sweep,
            commands::trigger_diagnostic_left_click,
            commands::import_profile_json,
            commands::export_active_profile_json,
            commands::reset_active_profile_to_defaults,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vibe Controller");
}

#[cfg(test)]
mod wave1_tests {
    use crate::models::companion::{
        CompanionEdge, CompanionMessage, CompanionMessageType, COMPANION_PROTOCOL_VERSION,
    };
    use crate::models::control::{ControllerControlID, ControllerFamily};
    use crate::models::profile::{ActionType, ControllerProfile, ProfileDocument, TriggerMode};
    use crate::models::shortcut::{vk, KeyboardModifier, ShortcutDescriptor};
    use crate::models::vibration::{ControllerVibration, VibrationLocality, VibrationPhase};

    #[test]
    fn control_family_inference_and_mapping_lists() {
        assert_eq!(
            ControllerFamily::inferred(Some("Sony DualSense Wireless Controller")),
            ControllerFamily::PlayStation
        );
        assert_eq!(
            ControllerFamily::inferred(Some("Xbox Series X Controller")),
            ControllerFamily::Xbox
        );
        assert_eq!(ControllerFamily::inferred(None), ControllerFamily::Generic);

        let xbox_controls = ControllerControlID::mapping_controls_for(ControllerFamily::Xbox);
        assert!(xbox_controls.contains(&ControllerControlID::Share));
        assert!(!xbox_controls.contains(&ControllerControlID::TouchpadButton));
        assert_eq!(xbox_controls.len(), 18);

        let ps_controls = ControllerControlID::mapping_controls_for(ControllerFamily::PlayStation);
        assert!(!ps_controls.contains(&ControllerControlID::Share));
        assert!(ps_controls.contains(&ControllerControlID::TouchpadButton));
        assert_eq!(ps_controls.len(), 18);
    }

    #[test]
    fn vibration_pulses_and_localities_match_spec() {
        assert_eq!(
            ControllerVibration::LeftPulse.locality(),
            VibrationLocality::Left
        );
        assert_eq!(
            ControllerVibration::RightPulse.locality(),
            VibrationLocality::Right
        );
        assert_eq!(
            ControllerVibration::DoubleTap
                .pulses(VibrationPhase::Press)
                .len(),
            2
        );
        assert_eq!(
            ControllerVibration::GrabRelease
                .pulses(VibrationPhase::Press)
                .len(),
            1
        );
        assert_eq!(
            ControllerVibration::GrabRelease
                .pulses(VibrationPhase::Release)
                .len(),
            1
        );
        assert!(ControllerVibration::None
            .pulses(VibrationPhase::Press)
            .is_empty());
    }

    #[test]
    fn shortcut_formatting_chords_and_macos_conversion() {
        let paste = ShortcutDescriptor::new(vk::V, vec![KeyboardModifier::Control]);
        assert_eq!(paste.display_string(), "Ctrl+V");
        assert!(paste.is_assignable());

        let chord = ShortcutDescriptor::left_right_modifier_chord(KeyboardModifier::Control);
        assert!(chord.is_supported_modifier_chord());
        assert_eq!(chord.display_string(), "LCtrl + RCtrl");
        assert!(chord.is_assignable());

        let alt_only = ShortcutDescriptor::new(0xA5, vec![]);
        assert!(!alt_only.is_assignable());
        assert!(alt_only.is_assignable_for(TriggerMode::HoldWhilePressed));

        let win_dict = ShortcutDescriptor::from_macos_shortcut(63, &[]);
        assert_eq!(win_dict.display_string(), "Win+H");
        let win_snip = ShortcutDescriptor::from_macos_shortcut(
            21,
            &[
                KeyboardModifier::Control,
                KeyboardModifier::Shift,
                KeyboardModifier::Win,
            ],
        );
        assert_eq!(win_snip.display_string(), "Shift+Win+S");
        let win_ocr = ShortcutDescriptor::from_macos_shortcut(
            19,
            &[KeyboardModifier::Shift, KeyboardModifier::Win],
        );
        assert_eq!(win_ocr.display_string(), "Shift+Win+T");
    }

    #[test]
    fn gabes_defaults_and_gape_json_deserialization() {
        let doc = ProfileDocument::default_document();
        assert_eq!(doc.version, 5);
        assert_eq!(doc.profiles.len(), 1);
        let p = &doc.profiles[0];
        assert_eq!(p.id, "gabes-defaults");
        assert_eq!(
            p.mappings[&ControllerControlID::Home].vibration,
            ControllerVibration::Thump
        );
        assert_eq!(
            p.mappings[&ControllerControlID::LeftThumbstickButton].vibration,
            ControllerVibration::SoftTap
        );
        assert_eq!(
            p.mappings[&ControllerControlID::LeftTrigger].vibration,
            ControllerVibration::GrabRelease
        );
        assert_eq!(
            p.effective_mapping(
                ControllerControlID::DpadLeft,
                Some(ControllerControlID::LeftShoulder),
                None
            )
            .action_type,
            ActionType::CrossEdgeLeft
        );
        let codex_menu = p.effective_mapping(ControllerControlID::Menu, None, Some("codex.exe"));
        assert_eq!(
            codex_menu.shortcut.as_ref().unwrap().display_string(),
            "Ctrl+N"
        );

        let gape_raw = include_str!("../../profiles/GAPE.json");
        let gape: ControllerProfile =
            serde_json::from_str(gape_raw).expect("macOS GAPE.json should deserialize cleanly");
        assert_eq!(gape.id, "gabes-defaults");
        assert_eq!(gape.modifier_layers.len(), 2);
        assert_eq!(gape.application_mappings.len(), 1);
    }

    #[test]
    fn companion_ndjson_protocol_roundtrip() {
        let msg = CompanionMessage::hello("DESKTOP-WIN".into(), "1.0.0".into());
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: CompanionMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.message_type, CompanionMessageType::Hello);
        assert_eq!(decoded.protocol_version, Some(COMPANION_PROTOCOL_VERSION));

        let handoff = CompanionMessage::handoff_start(CompanionEdge::Right, 0.42);
        let h_json = serde_json::to_string(&handoff).unwrap();
        let h_dec: CompanionMessage = serde_json::from_str(&h_json).unwrap();
        assert_eq!(h_dec.edge, Some(CompanionEdge::Right));
        assert_eq!(CompanionEdge::Right.opposite(), CompanionEdge::Left);
    }
}

#[cfg(test)]
mod wave2_tests {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use crate::engines::action_engine::{ActionEngine, ActionSink};
    use crate::engines::cursor_math::{
        adjusted_vector, decayed_flick_boost, zoom_repeat_interval, CursorMotionState,
        FlickBoostTracker, StickZoomRepeater, SubpixelAccumulator, ZoomDirection,
        CROSS_EDGE_SWEEP_DISTANCE,
    };
    use crate::engines::scroll_repeater::HeldScrollRepeater;
    use crate::models::companion::{
        CompanionMouseButton, CompanionMousePhase, CompanionShortcutPhase, SpaceSwitchDirection,
    };
    use crate::models::control::{ControllerControlID, ControllerSnapshot, StickSnapshot};
    use crate::models::profile::{
        ActionType, ApplicationMappingOverrides, ControllerActionMapping, ControllerProfile,
        CrossEdgeDirection, CursorConfiguration, ProfileDocument,
    };
    use crate::models::shortcut::ShortcutDescriptor;
    use crate::models::vibration::{ControllerVibration, VibrationPhase};
    use crate::services::codex_provisioner::{ensure_fork_shortcut, CodexKeybindingProvisionResult};
    use crate::services::profile_store::ProfileStore;

    #[derive(Default)]
    struct SpySink {
        events: Vec<String>,
        vibrations: Vec<(ControllerVibration, VibrationPhase)>,
    }

    impl ActionSink for SpySink {
        fn mouse(&mut self, button: CompanionMouseButton, phase: CompanionMousePhase) {
            self.events.push(format!("mouse:{:?}:{:?}", button, phase));
        }
        fn scroll(&mut self, vertical: i32, horizontal: i32) {
            self.events.push(format!("scroll:{},{}", vertical, horizontal));
        }
        fn shortcut(&mut self, shortcut: &ShortcutDescriptor, phase: CompanionShortcutPhase) {
            self.events
                .push(format!("shortcut:{}:{:?}", shortcut.display_string(), phase));
        }
        fn space_switch(&mut self, direction: SpaceSwitchDirection) {
            self.events.push(format!("space:{:?}", direction));
        }
        fn cross_edge(&mut self, direction: CrossEdgeDirection) {
            self.events.push(format!("cross:{:?}", direction));
        }
        fn toggle_cursor_speeds(&mut self) {
            self.events.push("toggleSpeeds".into());
        }
        fn start_scroll_repeat(
            &mut self,
            control: ControllerControlID,
            _modifier: Option<ControllerControlID>,
            v: i32,
            h: i32,
            _delay: f64,
            _interval: f64,
        ) {
            self.events
                .push(format!("startScroll:{:?}:{},{}", control, v, h));
        }
        fn stop_scroll_repeat(&mut self, control: ControllerControlID) {
            self.events.push(format!("stopScroll:{:?}", control));
        }
        fn stop_all_scroll_repeats(&mut self) {
            self.events.push("stopAllScroll".into());
        }
        fn vibrate(&mut self, pattern: ControllerVibration, phase: VibrationPhase) {
            if pattern != ControllerVibration::None {
                self.vibrations.push((pattern, phase));
            }
        }
        fn stop_vibration(&mut self) {
            self.events.push("stopVib".into());
        }
        fn report_status(&mut self, _status: String) {}
    }

    #[test]
    fn cursor_math_deadzone_curve_flick_zoom_and_sweep() {
        assert_eq!(adjusted_vector(0.08, 0.05, 0.12, 1.8), (0.0, 0.0));
        let (ax, ay) = adjusted_vector(1.0, 0.0, 0.12, 1.8);
        assert!((ax - 1.0).abs() < 1e-6 && ay.abs() < 1e-6);

        // Flick boost state machine
        let mut tracker = FlickBoostTracker::default();
        assert_eq!(tracker.sample(0.0, 0.0, 0.0), 1.0); // arms
        assert_eq!(tracker.sample(0.5, 0.0, 0.01), 1.0); // tracking
        let boost = tracker.sample(0.96, 0.0, 0.045); // 35ms < 40ms -> full 2.0x
        assert!((boost - 2.0).abs() < 1e-6);
        assert!((decayed_flick_boost(2.0, 1.0) - 1.5).abs() < 1e-6);
        assert!((decayed_flick_boost(2.0, 3.0) - 1.0).abs() < 1e-6);

        // Zoom gesture hysteresis and interval
        assert!((zoom_repeat_interval(0.55) - 0.28).abs() < 1e-6);
        assert!((zoom_repeat_interval(1.0) - 0.075).abs() < 1e-6);
        let mut zoom = StickZoomRepeater::default();
        assert_eq!(zoom.tick(true, 0.0, 0.50, 0.0), None);
        assert_eq!(zoom.tick(true, 0.0, 0.60, 0.1), Some(ZoomDirection::ZoomIn));
        assert_eq!(zoom.tick(true, 0.0, 0.45, 0.2), None); // stays active above 0.42
        assert!(zoom.is_active());
        assert_eq!(zoom.tick(true, 0.0, 0.30, 0.3), None); // releases below 0.42
        assert!(!zoom.is_active());

        // Subpixel accumulator
        let mut acc = SubpixelAccumulator::default();
        assert_eq!(acc.accumulate(0.4, -0.4), (0, 0));
        assert_eq!(acc.accumulate(0.4, -0.4), (0, 0));
        assert_eq!(acc.accumulate(0.4, -0.4), (1, -1));

        // Cross-edge sweep travels exact 1500 px over 0.25s
        let mut motion = CursorMotionState::new();
        motion.start_cross_edge_sweep(CrossEdgeDirection::Right);
        let cfg = CursorConfiguration::default();
        let snap = ControllerSnapshot::default();
        let mut total_x = 0.0;
        for _ in 0..40 {
            if let Some((dx, _)) = motion.tick(&snap, &cfg, 0.01, false) {
                total_x += dx;
            }
        }
        assert!((total_x - CROSS_EDGE_SWEEP_DISTANCE).abs() < 1e-6);
    }

    #[test]
    fn action_engine_modifier_tap_vs_hold_chords_and_drag() {
        let profile = ControllerProfile::gabes_defaults();
        let mut engine = ActionEngine::new();
        let mut sink = SpySink::default();

        let mut snap = ControllerSnapshot {
            is_connected: true,
            ..Default::default()
        };

        // 1. Tap LB alone -> silent on press, fires Escape + softTap on release
        snap.pressed_controls.insert(ControllerControlID::LeftShoulder);
        engine.process(&snap, &profile, None, 1.0, &mut sink);
        assert!(sink.events.is_empty());
        assert!(sink.vibrations.is_empty());

        snap.pressed_controls.remove(&ControllerControlID::LeftShoulder);
        engine.process(&snap, &profile, None, 1.1, &mut sink);
        assert_eq!(sink.events, vec!["shortcut:Esc:Tap"]);
        assert_eq!(
            sink.vibrations,
            vec![(ControllerVibration::SoftTap, VibrationPhase::Press)]
        );

        // 2. Hold LB + press DpadLeft -> CrossEdgeLeft + LeftPulse; releasing LB does NOT fire Escape
        sink.events.clear();
        sink.vibrations.clear();
        snap.pressed_controls.insert(ControllerControlID::LeftShoulder);
        engine.process(&snap, &profile, None, 2.0, &mut sink);
        snap.pressed_controls.insert(ControllerControlID::DpadLeft);
        engine.process(&snap, &profile, None, 2.05, &mut sink);
        snap.pressed_controls.remove(&ControllerControlID::DpadLeft);
        engine.process(&snap, &profile, None, 2.1, &mut sink);
        snap.pressed_controls.remove(&ControllerControlID::LeftShoulder);
        engine.process(&snap, &profile, None, 2.15, &mut sink);
        assert_eq!(sink.events, vec!["cross:Left"]);
        assert_eq!(
            sink.vibrations,
            vec![(ControllerVibration::LeftPulse, VibrationPhase::Press)]
        );

        // 3. Chorded modifiers: Hold LB + press RB -> LCtrl + RCtrl
        sink.events.clear();
        snap.pressed_controls.insert(ControllerControlID::LeftShoulder);
        engine.process(&snap, &profile, None, 3.0, &mut sink);
        snap.pressed_controls.insert(ControllerControlID::RightShoulder);
        engine.process(&snap, &profile, None, 3.05, &mut sink);
        snap.pressed_controls.clear();
        engine.process(&snap, &profile, None, 3.1, &mut sink);
        assert_eq!(sink.events, vec!["shortcut:LCtrl + RCtrl:Tap"]);

        // 4. LT drag -> GrabRelease press & release
        sink.events.clear();
        sink.vibrations.clear();
        snap.analog_values.insert(ControllerControlID::LeftTrigger, 0.5);
        engine.process(&snap, &profile, None, 4.0, &mut sink);
        snap.analog_values.insert(ControllerControlID::LeftTrigger, 0.0);
        engine.process(&snap, &profile, None, 4.2, &mut sink);
        assert_eq!(sink.events, vec!["mouse:Left:Down", "mouse:Left:Up"]);
        assert_eq!(
            sink.vibrations,
            vec![
                (ControllerVibration::GrabRelease, VibrationPhase::Press),
                (ControllerVibration::GrabRelease, VibrationPhase::Release),
            ]
        );
    }

    #[test]
    fn action_engine_app_none_suppression_and_disconnect_cleanup() {
        let mut profile = ControllerProfile::gabes_defaults();
        let mut notepad = ApplicationMappingOverrides {
            bundle_identifier: "notepad.exe".into(),
            display_name: "Notepad".into(),
            mappings: Default::default(),
            modifier_layers: vec![],
        };
        notepad.mappings.insert(
            ControllerControlID::ButtonSouth,
            ControllerActionMapping::new(ActionType::None),
        );
        profile.application_mappings.push(notepad);

        let mut engine = ActionEngine::new();
        let mut sink = SpySink::default();
        let mut snap = ControllerSnapshot {
            is_connected: true,
            ..Default::default()
        };
        snap.pressed_controls.insert(ControllerControlID::ButtonSouth);
        engine.process(&snap, &profile, Some("notepad.exe"), 1.0, &mut sink);
        assert!(sink.events.is_empty(), "App .none override must suppress base action");

        // Start drag then disconnect -> must release left mouse button
        snap.pressed_controls.clear();
        snap.analog_values.insert(ControllerControlID::LeftTrigger, 0.9);
        engine.process(&snap, &profile, None, 2.0, &mut sink);
        assert!(sink.events.contains(&"mouse:Left:Down".to_string()));
        snap.is_connected = false;
        engine.process(&snap, &profile, None, 2.1, &mut sink);
        assert!(sink.events.contains(&"mouse:Left:Up".to_string()));
    }

    #[test]
    fn held_scroll_repeater_emits_and_stops_on_release() {
        let ticks = Arc::new(Mutex::new(Vec::new()));
        let ticks_clone = Arc::clone(&ticks);
        let repeater = HeldScrollRepeater::new(move |v, h| {
            ticks_clone.lock().unwrap().push((v, h));
        });

        let mut snap = ControllerSnapshot {
            is_connected: true,
            ..Default::default()
        };
        snap.pressed_controls.insert(ControllerControlID::DpadDown);
        repeater.update_input(&snap);
        repeater.start(ControllerControlID::DpadDown, None, 1, 0, 0.02, 0.02);
        thread::sleep(Duration::from_millis(75));

        snap.pressed_controls.remove(&ControllerControlID::DpadDown);
        repeater.update_input(&snap);
        let count_at_release = ticks.lock().unwrap().len();
        assert!(count_at_release >= 1, "Expected at least 1 repeated scroll tick");

        thread::sleep(Duration::from_millis(60));
        assert_eq!(
            ticks.lock().unwrap().len(),
            count_at_release,
            "Scroll repeater must stop immediately on release"
        );
    }

    #[test]
    fn profile_store_migrations_and_macos_import_dedupe() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ProfileStore::with_directory(tmp.path().to_path_buf());
        let doc = store.load_or_create().unwrap();
        assert_eq!(doc.version, 5);
        assert!(store.file_path().exists());

        // Test v2 -> v5 migration adds Codex starter
        let mut legacy = ProfileDocument::default_document();
        legacy.version = 2;
        legacy.profiles[0].application_mappings.clear();
        let migrated = ProfileStore::normalize(legacy);
        assert_eq!(migrated.version, 5);
        assert_eq!(migrated.profiles[0].application_mappings.len(), 1);

        // Import macOS GAPE.json -> auto-converts shortcuts & dedupes ID/name
        let gape_raw = include_str!("../../profiles/GAPE.json");
        let merged = ProfileStore::import_profile_str(gape_raw, &doc).unwrap();
        assert_eq!(merged.profiles.len(), 2);
        let imported = &merged.profiles[1];
        assert_eq!(imported.id, "gabes-defaults-2");
        assert_eq!(imported.name, "Gabe's Defaults 2");
        assert_eq!(
            imported.mappings[&ControllerControlID::ButtonNorth]
                .shortcut
                .as_ref()
                .unwrap()
                .display_string(),
            "Ctrl+V"
        );
        assert_eq!(
            imported.mappings[&ControllerControlID::RightTrigger]
                .shortcut
                .as_ref()
                .unwrap()
                .display_string(),
            "Win+H"
        );
    }

    #[test]
    fn codex_keybinding_provisioner_non_destructive() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(
            ensure_fork_shortcut(tmp.path()).unwrap(),
            CodexKeybindingProvisionResult::CodexConfigurationUnavailable
        );

        let codex_dir = tmp.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("keybindings.json"),
            r#"[{"command":"existing","key":"Ctrl+K"}]"#,
        )
        .unwrap();

        assert_eq!(
            ensure_fork_shortcut(tmp.path()).unwrap(),
            CodexKeybindingProvisionResult::Installed
        );
        assert_eq!(
            ensure_fork_shortcut(tmp.path()).unwrap(),
            CodexKeybindingProvisionResult::AlreadyConfigured
        );
        let content = std::fs::read_to_string(codex_dir.join("keybindings.json")).unwrap();
        assert!(content.contains("existing"));
        assert!(content.contains("forkThread"));
        assert!(content.contains("Ctrl+Alt+Shift+F"));
    }

    #[test]
    fn stick_motion_velocity_and_acceleration() {
        let mut motion = CursorMotionState::new();
        let cfg = CursorConfiguration {
            smoothing: 0.0,
            acceleration_enabled: true,
            ..Default::default()
        };
        let snap = ControllerSnapshot {
            is_connected: true,
            left_stick: StickSnapshot {
                x: 1.0,
                y: 0.0,
                pressed: false,
            },
            ..Default::default()
        };
        let (dx, dy) = motion.tick(&snap, &cfg, 0.01, false).unwrap();
        assert!(dx > 20.0 && dy.abs() < 1e-6);
    }
}

#[cfg(test)]
mod wave3_tests {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use crate::hardware::haptics::{motor_levels, HapticsEngine, RumbleSink};
    use crate::hardware::hid_controller::{
        build_ds4_usb_rumble, build_dualsense_usb_rumble, parse_dualsense_report,
        parse_dualshock4_report, parse_xbox360_usb_report,
    };
    use crate::hardware::input_relay::SnapshotRelayFilter;
    use crate::models::control::{ControllerControlID, ControllerFamily, ControllerSnapshot};
    use crate::models::vibration::{
        ControllerVibration, HapticsPreferences, HapticsStatus, VibrationLocality, VibrationPhase,
    };

    #[derive(Default)]
    struct SpyRumble {
        calls: Mutex<Vec<(f32, f32)>>,
    }

    impl RumbleSink for SpyRumble {
        fn set_motors(&self, left: f32, right: f32) -> bool {
            self.calls.lock().unwrap().push((left, right));
            true
        }
        fn status(&self) -> HapticsStatus {
            HapticsStatus {
                can_play: true,
                backend: "spy".into(),
                message: "Ready".into(),
            }
        }
    }

    #[test]
    fn dualsense_and_ds4_hid_parsers_handle_buttons_sticks_and_touchpad() {
        // DualSense USB report: report ID 0x01, LX=255 (+1), LY=0 (+1), RX=128, RY=128, LT=255, RT=0
        let mut ds_usb = [0u8; 16];
        ds_usb[0] = 0x01;
        ds_usb[1] = 255; // LX -> +1
        ds_usb[2] = 0;   // LY -> +1 (up)
        ds_usb[3] = 128;
        ds_usb[4] = 128;
        ds_usb[5] = 255; // L2 full
        ds_usb[6] = 0;   // R2 0
        ds_usb[8] = 0x20 | 0x00; // Cross + DpadUp
        ds_usb[9] = 0x20; // Options (Menu)
        ds_usb[10] = 0x02; // Touchpad click

        let snap = parse_dualsense_report(&ds_usb, 1).unwrap();
        assert_eq!(snap.family, ControllerFamily::PlayStation);
        assert!(snap.left_stick.x > 0.99);
        assert!(snap.left_stick.y > 0.99);
        assert!(snap.pressed_controls.contains(&ControllerControlID::ButtonSouth));
        assert!(snap.pressed_controls.contains(&ControllerControlID::DpadUp));
        assert!(snap.pressed_controls.contains(&ControllerControlID::Menu));
        assert!(snap.pressed_controls.contains(&ControllerControlID::TouchpadButton));
        assert!(snap.value_for(ControllerControlID::LeftTrigger) > 0.99);

        // DualShock 4 USB report
        let mut ds4 = [0u8; 12];
        ds4[0] = 0x01;
        ds4[1] = 128;
        ds4[2] = 128;
        ds4[3] = 128;
        ds4[4] = 128;
        ds4[5] = 0x40 | 0x06; // Circle + DpadLeft
        ds4[6] = 0x01; // L1
        ds4[7] = 0x03; // PS + Touchpad
        ds4[8] = 0;
        ds4[9] = 200; // R2
        let snap4 = parse_dualshock4_report(&ds4, 2).unwrap();
        assert!(snap4.pressed_controls.contains(&ControllerControlID::ButtonEast));
        assert!(snap4.pressed_controls.contains(&ControllerControlID::DpadLeft));
        assert!(snap4.pressed_controls.contains(&ControllerControlID::LeftShoulder));
        assert!(snap4.pressed_controls.contains(&ControllerControlID::Home));
        assert!(snap4.pressed_controls.contains(&ControllerControlID::TouchpadButton));

        // DualShock 4 Bluetooth report (report ID 0x11, offset 3)
        let mut ds4_bt = [0u8; 14];
        ds4_bt[0] = 0x11;
        ds4_bt[3] = 128;
        ds4_bt[4] = 128;
        ds4_bt[5] = 128;
        ds4_bt[6] = 128;
        ds4_bt[7] = 0x20 | 0x00; // Cross + DpadUp
        ds4_bt[8] = 0x02; // R1
        ds4_bt[9] = 0x02; // Touchpad
        let snap4_bt = parse_dualshock4_report(&ds4_bt, 3).unwrap();
        assert_eq!(snap4_bt.transport.as_deref(), Some("Bluetooth HID"));
        assert!(snap4_bt.pressed_controls.contains(&ControllerControlID::ButtonSouth));
        assert!(snap4_bt.pressed_controls.contains(&ControllerControlID::RightShoulder));
        assert!(snap4_bt.pressed_controls.contains(&ControllerControlID::TouchpadButton));

        // Rumble packets
        let r_ds = build_dualsense_usb_rumble(1.0, 0.5);
        assert_eq!(r_ds[0], 0x02);
        assert_eq!(r_ds[4], 255);
        assert_eq!(r_ds[3], 128);
        let r_ds4 = build_ds4_usb_rumble(0.5, 1.0);
        assert_eq!(r_ds4[0], 0x05);
        assert_eq!(r_ds4[5], 128);
        assert_eq!(r_ds4[4], 255);
    }

    #[test]
    fn xbox360_usb_report_parser_maps_start_back_guide() {
        let mut pkt = [0u8; 20];
        pkt[0] = 0x00;
        pkt[1] = 0x14;
        pkt[2] = 0x10 | 0x20 | 0x01; // Start (Menu) + Back (Options) + DpadUp
        pkt[3] = 0x04 | 0x10; // Guide (Home) + A
        pkt[4] = 128; // LT ~0.5
        pkt[5] = 0;
        let snap = parse_xbox360_usb_report(&pkt, 7).unwrap();
        assert_eq!(snap.family, ControllerFamily::Xbox);
        assert!(snap.pressed_controls.contains(&ControllerControlID::Menu));
        assert!(snap.pressed_controls.contains(&ControllerControlID::Options));
        assert!(snap.pressed_controls.contains(&ControllerControlID::Home));
        assert!(snap.pressed_controls.contains(&ControllerControlID::ButtonSouth));
        assert!(snap.pressed_controls.contains(&ControllerControlID::LeftTrigger));
    }

    #[test]
    fn haptics_locality_routing_and_pulse_execution() {
        assert_eq!(motor_levels(VibrationLocality::Left, 0.8, 0.5), (0.4, 0.0));
        assert_eq!(motor_levels(VibrationLocality::Right, 0.8, 0.5), (0.0, 0.4));
        assert_eq!(motor_levels(VibrationLocality::Handles, 0.8, 0.5), (0.4, 0.4));

        let spy = Arc::new(SpyRumble::default());
        let engine = HapticsEngine::new(spy.clone());
        engine.set_preferences(HapticsPreferences {
            enabled: true,
            strength: 0.5,
            connection_feedback: true,
        });
        engine.play(ControllerVibration::LeftPulse, VibrationPhase::Press);
        thread::sleep(Duration::from_millis(180));
        let calls = spy.calls.lock().unwrap().clone();
        assert!(calls.len() >= 2, "Expected motor start and stop");
        assert!(calls[0].0 > 0.3 && calls[0].1 == 0.0);
        assert_eq!(*calls.last().unwrap(), (0.0, 0.0));
    }

    #[test]
    fn snapshot_relay_filter_dedupes_and_throttles_telemetry() {
        let mut filter = SnapshotRelayFilter::default();
        let t0 = Instant::now();
        let mut snap = ControllerSnapshot {
            is_connected: true,
            ..Default::default()
        };
        let d1 = filter.evaluate(&snap, t0);
        assert!(d1.emit_realtime && d1.emit_action && d1.emit_telemetry);

        // Identical payload immediately after -> nothing emitted
        let d2 = filter.evaluate(&snap, t0 + Duration::from_millis(5));
        assert!(!d2.emit_realtime && !d2.emit_action && !d2.emit_telemetry);

        // Stick change within 15Hz window -> realtime true, telemetry false
        snap.left_stick.x = 0.5;
        let d3 = filter.evaluate(&snap, t0 + Duration::from_millis(10));
        assert!(d3.emit_realtime && !d3.emit_action && !d3.emit_telemetry);

        // Button press -> always forces action & telemetry immediately
        snap.pressed_controls.insert(ControllerControlID::ButtonSouth);
        let d4 = filter.evaluate(&snap, t0 + Duration::from_millis(15));
        assert!(d4.emit_realtime && d4.emit_action && d4.emit_telemetry);
    }
}

#[cfg(test)]
mod wave4_tests {
    use crate::models::companion::CompanionEdge;
    use crate::models::control::{ControllerControlID, StickAssignment, StickRoleChoice, StickSide};
    use crate::models::profile::{ActionType, ControllerActionMapping};
    use crate::platform::win_input::{
        local_restore_point, normalized_edge_position, remote_entry_point, should_begin_handoff,
        DesktopBounds,
    };
    use crate::runtime::AppRuntime;
    use crate::services::profile_store::ProfileStore;

    #[test]
    fn desktop_handoff_edge_geometry_matches_spec() {
        let bounds = DesktopBounds {
            min_x: 0.0,
            min_y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        assert!(should_begin_handoff(
            (1919.0, 540.0),
            (5.0, 0.0),
            CompanionEdge::Right,
            bounds
        ));
        assert!(!should_begin_handoff(
            (1919.0, 540.0),
            (-5.0, 0.0),
            CompanionEdge::Right,
            bounds
        ));
        assert!(should_begin_handoff(
            (1.0, 540.0),
            (-3.0, 0.0),
            CompanionEdge::Left,
            bounds
        ));

        let norm = normalized_edge_position((1919.0, 540.0), CompanionEdge::Right, bounds);
        assert!((norm - 0.5).abs() < 1e-6);

        let restore = local_restore_point((1919.0, 540.0), CompanionEdge::Right, bounds);
        assert_eq!(restore, (1908.0, 540.0));

        let entry = remote_entry_point(CompanionEdge::Right, 0.5, bounds);
        assert_eq!(entry, (8.0, 540.0));
    }

    #[test]
    fn app_runtime_state_mutations_and_persistence() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ProfileStore::with_directory(tmp.path().to_path_buf());
        let rt = AppRuntime::new(store);

        // Assign right stick as primary -> swaps left to precision
        let st = rt
            .assign_stick_role(StickSide::Right, StickRoleChoice::Primary)
            .unwrap();
        assert_eq!(st.document.profiles[0].cursor.primary_stick, StickAssignment::Right);
        assert_eq!(st.document.profiles[0].cursor.precision_stick, StickAssignment::Left);

        // Add app scope & override ButtonSouth
        let _ = rt
            .add_app_scope("code.exe".into(), "VS Code".into())
            .unwrap();
        let st2 = rt
            .save_mapping(
                ControllerControlID::ButtonSouth,
                None,
                Some("code.exe".into()),
                ControllerActionMapping::new(ActionType::DoubleClick),
            )
            .unwrap();
        let effective = st2.document.profiles[0].effective_mapping(
            ControllerControlID::ButtonSouth,
            None,
            Some("code.exe"),
        );
        assert_eq!(effective.action_type, ActionType::DoubleClick);

        // Clear override -> inherits All Apps LeftClick
        let st3 = rt
            .clear_mapping_override(
                ControllerControlID::ButtonSouth,
                None,
                Some("code.exe".into()),
            )
            .unwrap();
        let inherited = st3.document.profiles[0].effective_mapping(
            ControllerControlID::ButtonSouth,
            None,
            Some("code.exe"),
        );
        assert_eq!(inherited.action_type, ActionType::LeftClick);

        // Export & re-import roundtrip
        let exported = rt.export_active_profile_json().unwrap();
        let st4 = rt.import_profile_json(exported).unwrap();
        assert_eq!(st4.document.profiles.len(), 2);
    }
}
