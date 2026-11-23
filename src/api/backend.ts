import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppStatePayload,
  CompanionEdge,
  CompanionMode,
  ControllerActionMapping,
  ControllerControlID,
  ControllerSnapshot,
  ControllerVibration,
  CursorConfiguration,
  GameGuardPreferences,
  GyroConfiguration,
  HapticsPreferences,
  HardwareFxPreferences,
  RadialMenuSlice,
  RunningAppInfo,
  StickRoleChoice,
  StickSide,
  TriageItem,
  TriageKind,
  TriageStatus,
  UpdateStatus,
  VoiceCommandRule,
} from "../types/models";
import { browserSim } from "./browserSim";

export function isTauriEnv(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const backend = {
  getAppState: () =>
    isTauriEnv() ? invoke<AppStatePayload>("get_app_state") : browserSim.getAppState(),
  setRuntimeEnabled: (enabled: boolean) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("set_runtime_enabled", { enabled })
      : browserSim.setRuntimeEnabled(enabled),
  selectProfile: (profileId: string) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("select_profile", { profileId })
      : browserSim.selectProfile(profileId),
  updateCursorConfig: (config: CursorConfiguration) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("update_cursor_config", { config })
      : browserSim.updateCursorConfig(config),
  assignStickRole: (side: StickSide, role: StickRoleChoice) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("assign_stick_role", { side, role })
      : browserSim.assignStickRole(side, role),
  saveMapping: (
    control: ControllerControlID,
    modifierControl: ControllerControlID | null,
    appBundleId: string | null,
    mapping: ControllerActionMapping
  ) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("save_mapping", {
          control,
          modifierControl,
          appBundleId,
          mapping,
        })
      : browserSim.saveMapping(control, modifierControl, appBundleId, mapping),
  clearMappingOverride: (
    control: ControllerControlID,
    modifierControl: ControllerControlID | null,
    appBundleId: string | null
  ) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("clear_mapping_override", {
          control,
          modifierControl,
          appBundleId,
        })
      : browserSim.clearMappingOverride(control, modifierControl, appBundleId),
  addModifierLayer: (modifierControl: ControllerControlID) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("add_modifier_layer", { modifierControl })
      : browserSim.addModifierLayer(modifierControl),
  removeModifierLayer: (modifierControl: ControllerControlID) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("remove_modifier_layer", { modifierControl })
      : browserSim.removeModifierLayer(modifierControl),
  listRunningApps: () =>
    isTauriEnv() ? invoke<RunningAppInfo[]>("list_running_apps") : browserSim.listRunningApps(),
  addAppScope: (bundleIdentifier: string, displayName: string) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("add_app_scope", { bundleIdentifier, displayName })
      : browserSim.addAppScope(bundleIdentifier, displayName),
  installCodexStarter: () =>
    isTauriEnv()
      ? invoke<AppStatePayload>("install_codex_starter")
      : browserSim.installCodexStarter(),
  resetAppScope: (bundleIdentifier: string) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("reset_app_scope", { bundleIdentifier })
      : browserSim.resetAppScope(bundleIdentifier),
  removeAppScope: (bundleIdentifier: string) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("remove_app_scope", { bundleIdentifier })
      : browserSim.removeAppScope(bundleIdentifier),
  setHapticsPreferences: (prefs: HapticsPreferences) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("set_haptics_preferences", { prefs })
      : browserSim.setHapticsPreferences(prefs),
  previewVibration: (pattern: ControllerVibration) =>
    isTauriEnv()
      ? invoke<void>("preview_vibration", { pattern })
      : browserSim.previewVibration(pattern),
  applySuggestedVibrations: () =>
    isTauriEnv()
      ? invoke<AppStatePayload>("apply_suggested_vibrations")
      : browserSim.applySuggestedVibrations(),
  setCompanionMode: (mode: CompanionMode) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("set_companion_mode", { mode })
      : browserSim.setCompanionMode(mode),
  setCompanionEdge: (edge: CompanionEdge) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("set_companion_edge", { edge })
      : browserSim.setCompanionEdge(edge),
  connectCompanion: (peerId: string | null) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("connect_companion", { peerId })
      : browserSim.connectCompanion(),
  disconnectCompanion: () =>
    isTauriEnv()
      ? invoke<AppStatePayload>("disconnect_companion")
      : browserSim.disconnectCompanion(),
  forceCompanionHandoff: () =>
    isTauriEnv()
      ? invoke<AppStatePayload>("force_companion_handoff")
      : browserSim.forceCompanionHandoff(),
  returnCompanionLocal: () =>
    isTauriEnv()
      ? invoke<AppStatePayload>("return_companion_local")
      : browserSim.returnCompanionLocal(),
  triggerDiagnosticSweep: () =>
    isTauriEnv()
      ? invoke<void>("trigger_diagnostic_sweep")
      : browserSim.triggerDiagnosticSweep(),
  triggerDiagnosticLeftClick: () =>
    isTauriEnv()
      ? invoke<void>("trigger_diagnostic_left_click")
      : browserSim.triggerDiagnosticLeftClick(),
  importProfileJson: (rawJson: string) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("import_profile_json", { rawJson })
      : browserSim.importProfileJson(rawJson),
  exportActiveProfileJson: () =>
    isTauriEnv()
      ? invoke<string>("export_active_profile_json")
      : browserSim.exportActiveProfileJson(),
  resetActiveProfileToDefaults: () =>
    isTauriEnv()
      ? invoke<AppStatePayload>("reset_active_profile_to_defaults")
      : browserSim.resetActiveProfileToDefaults(),
  getUpdateStatus: () =>
    isTauriEnv() ? invoke<UpdateStatus>("get_update_status") : browserSim.getUpdateStatus(),
  checkForUpdates: () =>
    isTauriEnv() ? invoke<UpdateStatus>("check_for_updates") : browserSim.checkForUpdates(),
  restartToUpdate: () =>
    isTauriEnv() ? invoke<void>("restart_to_update") : browserSim.restartToUpdate(),
  setVoiceEnabled: (enabled: boolean) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("set_voice_enabled", { enabled })
      : browserSim.setVoiceEnabled(enabled),
  setVoiceRules: (rules: VoiceCommandRule[]) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("set_voice_rules", { rules })
      : browserSim.setVoiceRules(rules),
  testCommandScript: (script: string) =>
    isTauriEnv()
      ? invoke<void>("test_command_script", { script })
      : browserSim.testCommandScript(script),
  setGameGuardPreferences: (prefs: GameGuardPreferences) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("set_game_guard_preferences", { prefs })
      : browserSim.setGameGuardPreferences(prefs),
  openSteamControllerSettings: () =>
    isTauriEnv()
      ? invoke<void>("open_steam_controller_settings")
      : browserSim.openSteamControllerSettings(),
  openSteamBigPicture: () =>
    isTauriEnv()
      ? invoke<void>("open_steam_big_picture")
      : browserSim.openSteamBigPicture(),
  updateGyroConfig: (gyro: GyroConfiguration) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("update_gyro_config", { gyro })
      : browserSim.updateGyroConfig(gyro),
  setHardwareFx: (fx: HardwareFxPreferences) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("set_hardware_fx", { fx })
      : browserSim.setHardwareFx(fx),
  updateRadialMenu: (slices: RadialMenuSlice[]) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("update_radial_menu", { slices })
      : browserSim.updateRadialMenu(slices),
  setRadialMenuOpen: (open: boolean) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("set_radial_menu_open", { open })
      : browserSim.setRadialMenuOpen(open),
  executeRadialSlice: (sliceId: string) =>
    isTauriEnv()
      ? invoke<AppStatePayload>("execute_radial_slice", { sliceId })
      : browserSim.executeRadialSlice(sliceId),
  listTriageItems: () =>
    isTauriEnv() ? invoke<TriageItem[]>("list_triage_items") : browserSim.listTriageItems(),
  submitTriageItem: (kind: TriageKind, title: string, details: string) =>
    isTauriEnv()
      ? invoke<TriageItem[]>("submit_triage_item", { kind, title, details })
      : browserSim.submitTriageItem(kind, title, details),
  setTriageStatus: (id: string, status: TriageStatus) =>
    isTauriEnv()
      ? invoke<TriageItem[]>("set_triage_status", { id, status })
      : browserSim.setTriageStatus(id, status),
  deleteTriageItem: (id: string) =>
    isTauriEnv()
      ? invoke<TriageItem[]>("delete_triage_item", { id })
      : browserSim.deleteTriageItem(id),
  exportAdeBatchPrompt: () =>
    isTauriEnv()
      ? invoke<string>("export_ade_batch_prompt")
      : browserSim.exportAdeBatchPrompt(),
  onTelemetry: (cb: (snapshot: ControllerSnapshot) => void): Promise<UnlistenFn> =>
    isTauriEnv()
      ? listen<ControllerSnapshot>("controller-telemetry", (event) => cb(event.payload))
      : browserSim.onTelemetry(cb),
};
