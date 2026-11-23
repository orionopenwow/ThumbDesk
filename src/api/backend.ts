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
  UpdateStatus,
  VoiceCommandRule,
} from "../types/models";

export function isTauriEnv(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const backend = {
  getAppState: () => invoke<AppStatePayload>("get_app_state"),
  setRuntimeEnabled: (enabled: boolean) =>
    invoke<AppStatePayload>("set_runtime_enabled", { enabled }),
  selectProfile: (profileId: string) =>
    invoke<AppStatePayload>("select_profile", { profileId }),
  updateCursorConfig: (config: CursorConfiguration) =>
    invoke<AppStatePayload>("update_cursor_config", { config }),
  assignStickRole: (side: StickSide, role: StickRoleChoice) =>
    invoke<AppStatePayload>("assign_stick_role", { side, role }),
  saveMapping: (
    control: ControllerControlID,
    modifierControl: ControllerControlID | null,
    appBundleId: string | null,
    mapping: ControllerActionMapping
  ) =>
    invoke<AppStatePayload>("save_mapping", {
      control,
      modifierControl,
      appBundleId,
      mapping,
    }),
  clearMappingOverride: (
    control: ControllerControlID,
    modifierControl: ControllerControlID | null,
    appBundleId: string | null
  ) =>
    invoke<AppStatePayload>("clear_mapping_override", {
      control,
      modifierControl,
      appBundleId,
    }),
  addModifierLayer: (modifierControl: ControllerControlID) =>
    invoke<AppStatePayload>("add_modifier_layer", { modifierControl }),
  removeModifierLayer: (modifierControl: ControllerControlID) =>
    invoke<AppStatePayload>("remove_modifier_layer", { modifierControl }),
  listRunningApps: () => invoke<RunningAppInfo[]>("list_running_apps"),
  addAppScope: (bundleIdentifier: string, displayName: string) =>
    invoke<AppStatePayload>("add_app_scope", { bundleIdentifier, displayName }),
  installCodexStarter: () => invoke<AppStatePayload>("install_codex_starter"),
  resetAppScope: (bundleIdentifier: string) =>
    invoke<AppStatePayload>("reset_app_scope", { bundleIdentifier }),
  removeAppScope: (bundleIdentifier: string) =>
    invoke<AppStatePayload>("remove_app_scope", { bundleIdentifier }),
  setHapticsPreferences: (prefs: HapticsPreferences) =>
    invoke<AppStatePayload>("set_haptics_preferences", { prefs }),
  previewVibration: (pattern: ControllerVibration) =>
    invoke<void>("preview_vibration", { pattern }),
  applySuggestedVibrations: () =>
    invoke<AppStatePayload>("apply_suggested_vibrations"),
  setCompanionMode: (mode: CompanionMode) =>
    invoke<AppStatePayload>("set_companion_mode", { mode }),
  setCompanionEdge: (edge: CompanionEdge) =>
    invoke<AppStatePayload>("set_companion_edge", { edge }),
  connectCompanion: (peerId: string | null) =>
    invoke<AppStatePayload>("connect_companion", { peerId }),
  disconnectCompanion: () => invoke<AppStatePayload>("disconnect_companion"),
  forceCompanionHandoff: () => invoke<AppStatePayload>("force_companion_handoff"),
  returnCompanionLocal: () => invoke<AppStatePayload>("return_companion_local"),
  triggerDiagnosticSweep: () => invoke<void>("trigger_diagnostic_sweep"),
  triggerDiagnosticLeftClick: () => invoke<void>("trigger_diagnostic_left_click"),
  importProfileJson: (rawJson: string) =>
    invoke<AppStatePayload>("import_profile_json", { rawJson }),
  exportActiveProfileJson: () => invoke<string>("export_active_profile_json"),
  resetActiveProfileToDefaults: () =>
    invoke<AppStatePayload>("reset_active_profile_to_defaults"),
  getUpdateStatus: () => invoke<UpdateStatus>("get_update_status"),
  checkForUpdates: () => invoke<UpdateStatus>("check_for_updates"),
  restartToUpdate: () => invoke<void>("restart_to_update"),
  setVoiceEnabled: (enabled: boolean) =>
    invoke<AppStatePayload>("set_voice_enabled", { enabled }),
  setVoiceRules: (rules: VoiceCommandRule[]) =>
    invoke<AppStatePayload>("set_voice_rules", { rules }),
  testCommandScript: (script: string) =>
    invoke<void>("test_command_script", { script }),
  setGameGuardPreferences: (prefs: GameGuardPreferences) =>
    invoke<AppStatePayload>("set_game_guard_preferences", { prefs }),
  openSteamControllerSettings: () => invoke<void>("open_steam_controller_settings"),
  openSteamBigPicture: () => invoke<void>("open_steam_big_picture"),
  updateGyroConfig: (gyro: GyroConfiguration) =>
    invoke<AppStatePayload>("update_gyro_config", { gyro }),
  setHardwareFx: (fx: HardwareFxPreferences) =>
    invoke<AppStatePayload>("set_hardware_fx", { fx }),
  updateRadialMenu: (slices: RadialMenuSlice[]) =>
    invoke<AppStatePayload>("update_radial_menu", { slices }),
  setRadialMenuOpen: (open: boolean) =>
    invoke<AppStatePayload>("set_radial_menu_open", { open }),
  executeRadialSlice: (sliceId: string) =>
    invoke<AppStatePayload>("execute_radial_slice", { sliceId }),
  onTelemetry: (cb: (snapshot: ControllerSnapshot) => void): Promise<UnlistenFn> =>
    listen<ControllerSnapshot>("controller-telemetry", (event) => cb(event.payload)),
};
