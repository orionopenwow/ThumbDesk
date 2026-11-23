import type {
  AppStatePayload,
  CompanionEdge,
  CompanionMode,
  ControllerActionMapping,
  ControllerControlID,
  ControllerFamily,
  ControllerProfile,
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
import { buildDemoState, getDemoPreset } from "../utils/demoState";

let simState: AppStatePayload | null = null;

function ensureState(): AppStatePayload {
  if (!simState) {
    simState = buildDemoState(getDemoPreset() ?? "ps");
  }
  return simState;
}

function cloneState(): AppStatePayload {
  return JSON.parse(JSON.stringify(ensureState())) as AppStatePayload;
}

function activeProfileMut(st: AppStatePayload): ControllerProfile {
  return (
    st.document.profiles.find((p) => p.id === st.document.activeProfileId) ??
    st.document.profiles[0]
  );
}

const STANDARD_BTN_MAP: [number, ControllerControlID][] = [
  [0, "buttonSouth"],
  [1, "buttonEast"],
  [2, "buttonWest"],
  [3, "buttonNorth"],
  [4, "leftShoulder"],
  [5, "rightShoulder"],
  [6, "leftTrigger"],
  [7, "rightTrigger"],
  [8, "options"],
  [9, "menu"],
  [10, "leftThumbstickButton"],
  [11, "rightThumbstickButton"],
  [12, "dpadUp"],
  [13, "dpadDown"],
  [14, "dpadLeft"],
  [15, "dpadRight"],
  [16, "home"],
  [17, "touchpadButton"],
];

export const browserSim = {
  getAppState: async (): Promise<AppStatePayload> => cloneState(),

  setRuntimeEnabled: async (enabled: boolean): Promise<AppStatePayload> => {
    const st = ensureState();
    st.runtimeEnabled = enabled;
    st.diagnostics.cursorState = enabled ? "idle" : "disabled";
    st.diagnostics.cursorMessage = enabled ? "Ready · 120 Hz loop" : "Input is paused";
    return cloneState();
  },

  selectProfile: async (profileId: string): Promise<AppStatePayload> => {
    const st = ensureState();
    if (st.document.profiles.some((p) => p.id === profileId)) {
      st.document.activeProfileId = profileId;
    }
    return cloneState();
  },

  updateCursorConfig: async (config: CursorConfiguration): Promise<AppStatePayload> => {
    const st = ensureState();
    activeProfileMut(st).cursor = { ...config };
    return cloneState();
  },

  assignStickRole: async (side: StickSide, role: StickRoleChoice): Promise<AppStatePayload> => {
    const st = ensureState();
    const c = activeProfileMut(st).cursor;
    const other = side === "left" ? "right" : "left";
    if (role === "primary") {
      if (c.precisionStick === side) c.precisionStick = c.primaryStick === other ? other : "off";
      c.primaryStick = side;
    } else if (role === "precision") {
      if (c.primaryStick === side) c.primaryStick = c.precisionStick === other ? other : "off";
      c.precisionStick = side;
    } else {
      if (c.primaryStick === side) c.primaryStick = "off";
      if (c.precisionStick === side) c.precisionStick = "off";
    }
    return cloneState();
  },

  saveMapping: async (
    control: ControllerControlID,
    modifierControl: ControllerControlID | null,
    appBundleId: string | null,
    mapping: ControllerActionMapping
  ): Promise<AppStatePayload> => {
    const st = ensureState();
    const prof = activeProfileMut(st);
    if (appBundleId) {
      let app = prof.applicationMappings.find((a) => a.bundleIdentifier === appBundleId);
      if (!app) {
        app = { bundleIdentifier: appBundleId, displayName: appBundleId, mappings: {}, modifierLayers: [] };
        prof.applicationMappings.push(app);
      }
      if (modifierControl) {
        let layer = app.modifierLayers.find((l) => l.modifierControl === modifierControl);
        if (!layer) {
          layer = { modifierControl, mappings: {} };
          app.modifierLayers.push(layer);
        }
        layer.mappings[control] = { ...mapping };
      } else {
        app.mappings[control] = { ...mapping };
      }
    } else if (modifierControl) {
      let layer = prof.modifierLayers.find((l) => l.modifierControl === modifierControl);
      if (!layer) {
        layer = { modifierControl, mappings: {} };
        prof.modifierLayers.push(layer);
      }
      layer.mappings[control] = { ...mapping };
    } else {
      prof.mappings[control] = { ...mapping };
    }
    st.diagnostics.lastActionStatus = `Mapped ${control}`;
    return cloneState();
  },

  clearMappingOverride: async (
    control: ControllerControlID,
    modifierControl: ControllerControlID | null,
    appBundleId: string | null
  ): Promise<AppStatePayload> => {
    const st = ensureState();
    const prof = activeProfileMut(st);
    if (appBundleId) {
      const app = prof.applicationMappings.find((a) => a.bundleIdentifier === appBundleId);
      if (app) {
        if (modifierControl) {
          const layer = app.modifierLayers.find((l) => l.modifierControl === modifierControl);
          if (layer) delete layer.mappings[control];
        } else {
          delete app.mappings[control];
        }
      }
    } else if (modifierControl) {
      const layer = prof.modifierLayers.find((l) => l.modifierControl === modifierControl);
      if (layer) delete layer.mappings[control];
    }
    return cloneState();
  },

  addModifierLayer: async (modifierControl: ControllerControlID): Promise<AppStatePayload> => {
    const st = ensureState();
    const prof = activeProfileMut(st);
    if (!prof.modifierLayers.some((l) => l.modifierControl === modifierControl)) {
      prof.modifierLayers.push({ modifierControl, mappings: {} });
    }
    return cloneState();
  },

  removeModifierLayer: async (modifierControl: ControllerControlID): Promise<AppStatePayload> => {
    const st = ensureState();
    const prof = activeProfileMut(st);
    prof.modifierLayers = prof.modifierLayers.filter((l) => l.modifierControl !== modifierControl);
    return cloneState();
  },

  listRunningApps: async (): Promise<RunningAppInfo[]> => [
    { bundleIdentifier: "code.exe", displayName: "VS Code", executablePath: "C:\\Program Files\\Microsoft VS Code\\Code.exe" },
    { bundleIdentifier: "chrome.exe", displayName: "Google Chrome", executablePath: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe" },
    { bundleIdentifier: "discord.exe", displayName: "Discord", executablePath: "C:\\Users\\demo\\AppData\\Local\\Discord\\Discord.exe" },
    { bundleIdentifier: "spotify.exe", displayName: "Spotify", executablePath: "C:\\Users\\demo\\AppData\\Roaming\\Spotify\\Spotify.exe" },
  ],

  addAppScope: async (bundleIdentifier: string, displayName: string): Promise<AppStatePayload> => {
    const st = ensureState();
    const prof = activeProfileMut(st);
    if (!prof.applicationMappings.some((a) => a.bundleIdentifier === bundleIdentifier)) {
      prof.applicationMappings.push({ bundleIdentifier, displayName, mappings: {}, modifierLayers: [] });
    }
    return cloneState();
  },

  installCodexStarter: async (): Promise<AppStatePayload> => cloneState(),

  resetAppScope: async (bundleIdentifier: string): Promise<AppStatePayload> => {
    const st = ensureState();
    const app = activeProfileMut(st).applicationMappings.find((a) => a.bundleIdentifier === bundleIdentifier);
    if (app) {
      app.mappings = {};
      app.modifierLayers = [];
    }
    return cloneState();
  },

  removeAppScope: async (bundleIdentifier: string): Promise<AppStatePayload> => {
    const st = ensureState();
    const prof = activeProfileMut(st);
    prof.applicationMappings = prof.applicationMappings.filter((a) => a.bundleIdentifier !== bundleIdentifier);
    return cloneState();
  },

  setHapticsPreferences: async (prefs: HapticsPreferences): Promise<AppStatePayload> => {
    const st = ensureState();
    st.hapticsPreferences = { ...prefs };
    return cloneState();
  },

  previewVibration: async (_pattern: ControllerVibration): Promise<void> => {
    // Fire real Web Gamepad rumble if a physical controller is connected!
    if (typeof navigator !== "undefined" && navigator.getGamepads) {
      const pads = navigator.getGamepads();
      for (const gp of pads) {
        const act = (gp as Gamepad & { vibrationActuator?: { playEffect?: (t: string, o: Record<string, number>) => Promise<void> } })?.vibrationActuator;
        if (act?.playEffect) {
          act.playEffect("dual-rumble", {
            startDelay: 0,
            duration: 140,
            weakMagnitude: 0.6,
            strongMagnitude: 0.45,
          }).catch(() => {});
          break;
        }
      }
    }
  },

  applySuggestedVibrations: async (): Promise<AppStatePayload> => cloneState(),

  setCompanionMode: async (mode: CompanionMode): Promise<AppStatePayload> => {
    const st = ensureState();
    st.companionMode = mode;
    st.companionState = mode === "off" ? { state: "off" } : mode === "controller" ? { state: "browsing" } : { state: "listening" };
    return cloneState();
  },

  setCompanionEdge: async (edge: CompanionEdge): Promise<AppStatePayload> => {
    ensureState().companionEdge = edge;
    return cloneState();
  },

  connectCompanion: async (): Promise<AppStatePayload> => {
    ensureState().companionState = { state: "connected", detail: "DESKTOP-STUDIO (LAN)" };
    return cloneState();
  },

  disconnectCompanion: async (): Promise<AppStatePayload> => {
    ensureState().companionState = { state: "off" };
    return cloneState();
  },

  forceCompanionHandoff: async (): Promise<AppStatePayload> => {
    ensureState().diagnostics.isRoutingToCompanion = true;
    return cloneState();
  },

  returnCompanionLocal: async (): Promise<AppStatePayload> => {
    ensureState().diagnostics.isRoutingToCompanion = false;
    return cloneState();
  },

  triggerDiagnosticSweep: async (): Promise<void> => {
    ensureState().diagnostics.lastActionStatus = "Diagnostic sweep tested (browser preview)";
  },

  triggerDiagnosticLeftClick: async (): Promise<void> => {
    ensureState().diagnostics.lastActionStatus = "Left click tested (browser preview)";
  },

  importProfileJson: async (rawJson: string): Promise<AppStatePayload> => {
    const parsed = JSON.parse(rawJson) as ControllerProfile;
    const st = ensureState();
    st.document.profiles.push(parsed);
    st.document.activeProfileId = parsed.id;
    return cloneState();
  },

  exportActiveProfileJson: async (): Promise<string> =>
    JSON.stringify(activeProfileMut(ensureState()), null, 2),

  resetActiveProfileToDefaults: async (): Promise<AppStatePayload> => {
    simState = buildDemoState("ps");
    return cloneState();
  },

  getUpdateStatus: async (): Promise<UpdateStatus> => ensureState().updateStatus,

  checkForUpdates: async (): Promise<UpdateStatus> => {
    const st = ensureState();
    st.updateStatus = {
      phase: "readyToRestart",
      currentVersion: "2.0.0",
      latestVersion: "v2.0.0",
      releaseNotes:
        "• 6-Axis Gyro Mouse & DualSense Adaptive Triggers\n• 8-Slice Obsidian Radial Pie Menu & Window Tiling\n• Hardware Drift Calibration Lab & Local Automation API",
      releaseUrl: "https://github.com/orionopenwow/ThumbDesk/releases/tag/v2.0.0",
      downloadedBytes: 5600000,
      totalBytes: 5600000,
      percent: 100,
      verifiedSha256: "0c474975b457d3655532718846fc510ec957e3e5beae5285861e9b882c09f2e7",
      stagedPath: "ThumbDesk-v2.0.0.exe",
      message: "v2.0.0 verified (click to preview installer)",
    };
    return { ...st.updateStatus };
  },

  restartToUpdate: async (): Promise<void> => {
    window.open("https://github.com/orionopenwow/ThumbDesk/releases/latest", "_blank");
  },

  setVoiceEnabled: async (enabled: boolean): Promise<AppStatePayload> => {
    const st = ensureState();
    st.voiceState.enabled = enabled;
    st.voiceState.listening = enabled;
    st.voiceState.statusMessage = enabled ? "Listening (offline Windows Speech)" : "Voice Mode off";
    return cloneState();
  },

  setVoiceRules: async (rules: VoiceCommandRule[]): Promise<AppStatePayload> => {
    ensureState().voiceState.rules = [...rules];
    return cloneState();
  },

  testCommandScript: async (script: string): Promise<void> => {
    ensureState().diagnostics.lastActionStatus = `Simulated cmd: ${script}`;
  },

  setGameGuardPreferences: async (prefs: GameGuardPreferences): Promise<AppStatePayload> => {
    ensureState().gameGuardPrefs = { ...prefs };
    return cloneState();
  },

  openSteamControllerSettings: async (): Promise<void> => {
    ensureState().diagnostics.lastActionStatus = "steam://settings/controller (Windows app)";
  },

  openSteamBigPicture: async (): Promise<void> => {
    ensureState().diagnostics.lastActionStatus = "steam://open/bigpicture (Windows app)";
  },

  updateGyroConfig: async (gyro: GyroConfiguration): Promise<AppStatePayload> => {
    activeProfileMut(ensureState()).gyro = { ...gyro };
    return cloneState();
  },

  setHardwareFx: async (fx: HardwareFxPreferences): Promise<AppStatePayload> => {
    ensureState().hardwareFx = { ...fx };
    return cloneState();
  },

  updateRadialMenu: async (slices: RadialMenuSlice[]): Promise<AppStatePayload> => {
    activeProfileMut(ensureState()).radialMenu = [...slices];
    return cloneState();
  },

  setRadialMenuOpen: async (open: boolean): Promise<AppStatePayload> => {
    ensureState().radialMenuOpen = open;
    return cloneState();
  },

  executeRadialSlice: async (sliceId: string): Promise<AppStatePayload> => {
    const st = ensureState();
    st.radialMenuOpen = false;
    const found = activeProfileMut(st).radialMenu?.find((s) => s.id === sliceId);
    if (found) st.diagnostics.lastActionStatus = `Radial: ${found.label}`;
    return cloneState();
  },

  /** Live Web Gamepad API telemetry loop with change-detection so idle controllers cause zero React re-renders! */
  onTelemetry: async (cb: (snapshot: ControllerSnapshot) => void): Promise<() => void> => {
    let rafId = 0;
    let events = 142;
    let active = true;
    let lastFingerprint = "";

    const tick = () => {
      if (!active) return;
      if (typeof navigator !== "undefined" && navigator.getGamepads) {
        const pads = navigator.getGamepads();
        const gp = pads.find((p) => p && p.connected);
        if (gp) {
          const lx = gp.axes[0] ?? 0;
          const ly = -(gp.axes[1] ?? 0);
          const rx = gp.axes[2] ?? 0;
          const ry = -(gp.axes[3] ?? 0);
          const pressed: ControllerControlID[] = [];
          const analog: Partial<Record<ControllerControlID, number>> = {};
          for (const [idx, ctrl] of STANDARD_BTN_MAP) {
            const btn = gp.buttons[idx];
            if (!btn) continue;
            if (ctrl === "leftTrigger" || ctrl === "rightTrigger") {
              analog[ctrl] = btn.value;
              if (btn.value > 0.18 || btn.pressed) pressed.push(ctrl);
            } else if (btn.pressed) {
              pressed.push(ctrl);
            }
          }
          const fp = `${gp.id}|${lx.toFixed(3)},${ly.toFixed(3)},${rx.toFixed(3)},${ry.toFixed(3)}|${pressed.join(",")}`;
          if (fp !== lastFingerprint) {
            lastFingerprint = fp;
            const lower = gp.id.toLowerCase();
            const family: ControllerFamily =
              lower.includes("dualsense") || lower.includes("dualshock") || lower.includes("054c") || lower.includes("wireless controller")
                ? "playStation"
                : "xbox";
            events += 1;
            const snap: ControllerSnapshot = {
              isConnected: true,
              name: gp.id.split("(")[0].trim() || (family === "playStation" ? "DualSense Controller" : "Xbox Controller"),
              family,
              transport: "Web Gamepad API (Live)",
              batteryLevel: null,
              leftStick: { x: lx, y: ly, pressed: Boolean(gp.buttons[10]?.pressed) },
              rightStick: { x: rx, y: ry, pressed: Boolean(gp.buttons[11]?.pressed) },
              pressedControls: pressed,
              analogValues: analog,
              eventCount: events,
              shareSupported: family === "xbox",
              hasGyro: family === "playStation",
              gyro: [0, 0, 0],
              touchpadTouched: Boolean(gp.buttons[17]?.pressed),
              pollingRateHz: 120,
            };
            if (simState) simState.snapshot = snap;
            cb(snap);
          }
        }
      }
      rafId = window.requestAnimationFrame(tick);
    };
    rafId = window.requestAnimationFrame(tick);

    return () => {
      active = false;
      window.cancelAnimationFrame(rafId);
    };
  },
};
