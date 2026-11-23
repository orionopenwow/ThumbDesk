use std::sync::Arc;
use tauri::State;

use crate::models::companion::{CompanionEdge, CompanionMode};
use crate::models::control::{ControllerControlID, StickRoleChoice, StickSide};
use crate::models::profile::{ControllerActionMapping, CursorConfiguration};
use crate::models::vibration::{ControllerVibration, HapticsPreferences};
use crate::platform::active_app::RunningAppInfo;
use crate::runtime::{AppRuntime, AppStatePayload};

type SharedRuntime<'a> = State<'a, Arc<AppRuntime>>;

#[tauri::command]
pub fn get_app_state(runtime: SharedRuntime<'_>) -> AppStatePayload {
    runtime.state_payload()
}

#[tauri::command]
pub fn set_runtime_enabled(enabled: bool, runtime: SharedRuntime<'_>) -> AppStatePayload {
    runtime.set_runtime_enabled(enabled)
}

#[tauri::command]
pub fn select_profile(profile_id: String, runtime: SharedRuntime<'_>) -> Result<AppStatePayload, String> {
    runtime.select_profile(profile_id)
}

#[tauri::command]
pub fn update_cursor_config(
    config: CursorConfiguration,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.update_cursor_config(config)
}

#[tauri::command]
pub fn assign_stick_role(
    side: StickSide,
    role: StickRoleChoice,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.assign_stick_role(side, role)
}

#[tauri::command]
pub fn save_mapping(
    control: ControllerControlID,
    modifier_control: Option<ControllerControlID>,
    app_bundle_id: Option<String>,
    mapping: ControllerActionMapping,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.save_mapping(control, modifier_control, app_bundle_id, mapping)
}

#[tauri::command]
pub fn clear_mapping_override(
    control: ControllerControlID,
    modifier_control: Option<ControllerControlID>,
    app_bundle_id: Option<String>,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.clear_mapping_override(control, modifier_control, app_bundle_id)
}

#[tauri::command]
pub fn add_modifier_layer(
    modifier_control: ControllerControlID,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.add_modifier_layer(modifier_control)
}

#[tauri::command]
pub fn remove_modifier_layer(
    modifier_control: ControllerControlID,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.remove_modifier_layer(modifier_control)
}

#[tauri::command]
pub fn list_running_apps(runtime: SharedRuntime<'_>) -> Vec<RunningAppInfo> {
    runtime.running_apps()
}

#[tauri::command]
pub fn add_app_scope(
    bundle_identifier: String,
    display_name: String,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.add_app_scope(bundle_identifier, display_name)
}

#[tauri::command]
pub fn install_codex_starter(runtime: SharedRuntime<'_>) -> Result<AppStatePayload, String> {
    runtime.install_codex_starter()
}

#[tauri::command]
pub fn reset_app_scope(
    bundle_identifier: String,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.reset_app_scope(bundle_identifier)
}

#[tauri::command]
pub fn remove_app_scope(
    bundle_identifier: String,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.remove_app_scope(bundle_identifier)
}

#[tauri::command]
pub fn set_haptics_preferences(
    prefs: HapticsPreferences,
    runtime: SharedRuntime<'_>,
) -> AppStatePayload {
    runtime.set_haptics_preferences(prefs)
}

#[tauri::command]
pub fn preview_vibration(pattern: ControllerVibration, runtime: SharedRuntime<'_>) {
    runtime.preview_vibration(pattern);
}

#[tauri::command]
pub fn apply_suggested_vibrations(runtime: SharedRuntime<'_>) -> Result<AppStatePayload, String> {
    runtime.apply_suggested_vibrations()
}

#[tauri::command]
pub fn set_companion_mode(mode: CompanionMode, runtime: SharedRuntime<'_>) -> AppStatePayload {
    runtime.set_companion_mode(mode)
}

#[tauri::command]
pub fn set_companion_edge(edge: CompanionEdge, runtime: SharedRuntime<'_>) -> AppStatePayload {
    runtime.set_companion_edge(edge)
}

#[tauri::command]
pub fn connect_companion(peer_id: Option<String>, runtime: SharedRuntime<'_>) -> AppStatePayload {
    runtime.connect_companion(peer_id)
}

#[tauri::command]
pub fn disconnect_companion(runtime: SharedRuntime<'_>) -> AppStatePayload {
    runtime.disconnect_companion()
}

#[tauri::command]
pub fn force_companion_handoff(runtime: SharedRuntime<'_>) -> AppStatePayload {
    runtime.force_companion_handoff()
}

#[tauri::command]
pub fn return_companion_local(runtime: SharedRuntime<'_>) -> AppStatePayload {
    runtime.return_companion_local()
}

#[tauri::command]
pub fn trigger_diagnostic_sweep(runtime: SharedRuntime<'_>) {
    runtime.trigger_diagnostic_sweep();
}

#[tauri::command]
pub fn trigger_diagnostic_left_click(runtime: SharedRuntime<'_>) {
    runtime.trigger_diagnostic_left_click();
}

#[tauri::command]
pub fn import_profile_json(
    raw_json: String,
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.import_profile_json(raw_json)
}

#[tauri::command]
pub fn export_active_profile_json(runtime: SharedRuntime<'_>) -> Result<String, String> {
    runtime.export_active_profile_json()
}

#[tauri::command]
pub fn reset_active_profile_to_defaults(
    runtime: SharedRuntime<'_>,
) -> Result<AppStatePayload, String> {
    runtime.reset_active_profile_to_defaults()
}

#[tauri::command]
pub fn get_update_status(runtime: SharedRuntime<'_>) -> crate::services::updater::UpdateStatus {
    runtime.update_status()
}

#[tauri::command]
pub fn check_for_updates(runtime: SharedRuntime<'_>) -> crate::services::updater::UpdateStatus {
    runtime.check_for_updates()
}

#[tauri::command]
pub fn restart_to_update(
    app: tauri::AppHandle,
    runtime: SharedRuntime<'_>,
) -> Result<(), String> {
    runtime.apply_staged_update()?;
    app.exit(0);
    Ok(())
}
