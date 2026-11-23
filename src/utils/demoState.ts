import type { AppStatePayload, ControllerControlID } from "../types/models";

export type DemoPreset = "ps" | "ps-mod" | "xbox" | "sheet" | "updater" | "lab" | "radial";

export function getDemoPreset(): DemoPreset | null {
  if (typeof window === "undefined") return null;
  const p = new URLSearchParams(window.location.search).get("demo");
  if (
    p === "ps" ||
    p === "ps-mod" ||
    p === "xbox" ||
    p === "sheet" ||
    p === "updater" ||
    p === "lab" ||
    p === "radial"
  ) {
    return p;
  }
  return null;
}

export function buildDemoState(preset: DemoPreset): AppStatePayload {
  const isXbox = preset === "xbox";
  const isMod = preset === "ps-mod";

  const pressedControls: ControllerControlID[] = isMod
    ? ["leftShoulder", "dpadRight"]
    : isXbox
    ? ["buttonSouth"]
    : [];

  return {
    document: {
      version: 5,
      activeProfileId: "gabes-defaults",
      profiles: [
        {
          id: "gabes-defaults",
          name: "Gabe's Defaults",
          cursor: {
            primaryStick: "left",
            precisionStick: "right",
            primarySpeed: 1920,
            precisionSpeed: 560,
            deadZone: 0.12,
            responseCurve: 1.8,
            smoothing: 0.25,
            accelerationEnabled: true,
            invertPrimaryX: false,
            invertPrimaryY: false,
            invertPrecisionX: false,
            invertPrecisionY: false,
            horizontalSpeedMultiplier: 1.0,
            verticalSpeedMultiplier: 1.0,
            flickBoostEnabled: true,
            zoomGestureEnabled: true,
          },
          gyro: {
            enabled: true,
            activationMode: "touchpadTouch",
            sensitivity: 1.4,
            deadzone: 0.04,
            smoothing: 0.25,
            invertX: false,
            invertY: false,
          },
          radialMenu: [
            { id: "s1", label: "Terminal", actionType: "runCommand", commandScript: "wt.exe" },
            { id: "s2", label: "Snap Left", actionType: "windowSnapLeft" },
            { id: "s3", label: "Snap Right", actionType: "windowSnapRight" },
            { id: "s4", label: "Next Monitor", actionType: "windowNextMonitor" },
            { id: "s5", label: "Play / Pause", actionType: "mediaPlayPause" },
            { id: "s6", label: "Mute Audio", actionType: "volumeMute" },
            { id: "s7", label: "Voice Mode", actionType: "toggleVoiceMode" },
            { id: "s8", label: "Maximize", actionType: "windowMaximize" },
          ],
          mappings: {
            buttonSouth: { actionType: "leftClick", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
            buttonEast: { actionType: "rightClick", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
            buttonWest: { actionType: "keyboardShortcut", shortcut: { keyCode: 0x0d, modifiers: [] }, triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "none" },
            buttonNorth: { actionType: "keyboardShortcut", shortcut: { keyCode: 0x56, modifiers: ["control"] }, triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
            leftShoulder: { actionType: "keyboardShortcut", shortcut: { keyCode: 0x1b, modifiers: [] }, triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
            rightShoulder: { actionType: "keyboardShortcut", shortcut: { keyCode: 0x09, modifiers: [] }, triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
            leftTrigger: { actionType: "leftMouseHold", triggerMode: "holdWhilePressed", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "grabRelease" },
            rightTrigger: { actionType: "keyboardShortcut", shortcut: { keyCode: 0x48, modifiers: ["win"] }, triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "doubleTap" },
            dpadUp: { actionType: "scrollUp", triggerMode: "repeatWhileHeld", repeatDelay: 0.22, repeatInterval: 0.06, vibration: "none" },
            dpadDown: { actionType: "scrollDown", triggerMode: "repeatWhileHeld", repeatDelay: 0.22, repeatInterval: 0.06, vibration: "none" },
            dpadLeft: { actionType: "switchSpaceLeft", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "leftPulse" },
            dpadRight: { actionType: "switchSpaceRight", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "rightPulse" },
            leftThumbstickButton: { actionType: "toggleCursorSpeeds", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
            rightThumbstickButton: { actionType: "middleClick", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
            menu: { actionType: "runCommand", commandScript: "wt.exe", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
            options: { actionType: "toggleVoiceMode", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "doubleTap" },
            home: { actionType: "keyboardShortcut", shortcut: { keyCode: 0x57, modifiers: ["control"] }, triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "thump" },
            touchpadButton: { actionType: "keyboardShortcut", shortcut: { keyCode: 0x53, modifiers: ["shift", "win"] }, triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "doubleTap" },
            share: { actionType: "keyboardShortcut", shortcut: { keyCode: 0x53, modifiers: ["shift", "win"] }, triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "doubleTap" },
          },
          modifierLayers: [
            {
              modifierControl: "leftShoulder",
              mappings: {
                dpadLeft: { actionType: "crossEdgeLeft", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "leftPulse" },
                dpadRight: { actionType: "crossEdgeRight", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "rightPulse" },
                dpadUp: { actionType: "crossEdgeUp", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "thump" },
                dpadDown: { actionType: "crossEdgeDown", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "thump" },
                buttonNorth: { actionType: "runCommand", commandScript: "code .", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
              },
            },
            {
              modifierControl: "rightShoulder",
              mappings: {
                buttonSouth: { actionType: "doubleClick", triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "doubleTap" },
              },
            },
          ],
          applicationMappings: [
            {
              bundleIdentifier: "codex.exe",
              displayName: "Codex",
              mappings: {
                menu: { actionType: "keyboardShortcut", shortcut: { keyCode: 0x4e, modifiers: ["control"] }, triggerMode: "tap", repeatDelay: 0.35, repeatInterval: 0.08, vibration: "softTap" },
              },
              modifierLayers: [],
            },
          ],
        },
      ],
    },
    runtimeEnabled: true,
    hapticsPreferences: {
      enabled: true,
      strength: 0.75,
      connectionFeedback: true,
    },
    hapticsStatus: {
      canPlay: true,
      backend: isXbox ? "XInput Dual-Motor" : "DualSense Direct HID",
      message: "Hardware rumble ready",
    },
    companionMode: "off",
    companionEdge: "right",
    companionState: { state: "off" },
    companionPeers: [],
    diagnostics: {
      cursorState: isMod ? "moving" : "idle",
      cursorMessage: isMod ? "Moving cursor · 120 Hz" : "Ready · 120 Hz loop",
      lastActionStatus: isMod ? "Cross Edge Right (L1 + →)" : "Ready",
      activeAppId: "code.exe",
      activeAppName: "VS Code",
      isRoutingToCompanion: false,
    },
    snapshot: {
      isConnected: true,
      name: isXbox
        ? "Xbox Series X|S Controller"
        : "Sony DualSense Wireless Controller",
      family: isXbox ? "xbox" : "playStation",
      transport: isXbox ? "USB / XInput" : "USB-C HID",
      batteryLevel: isXbox ? 0.88 : 0.94,
      leftStick: isMod ? { x: 0.72, y: -0.25, pressed: false } : { x: 0, y: 0, pressed: false },
      rightStick: { x: 0, y: 0, pressed: false },
      pressedControls,
      analogValues: {},
      eventCount: 142,
      shareSupported: isXbox,
      hasGyro: !isXbox,
      gyro: [0.02, -0.14, 0.01],
      touchpadTouched: false,
      pollingRateHz: isXbox ? 125 : 250,
    },
    updateStatus:
      preset === "updater"
        ? {
            phase: "readyToRestart",
            currentVersion: "1.1.0",
            latestVersion: "v1.2.0",
            releaseNotes:
              "• Offline Voice Mode with custom command & script execution\n• Anti-Cheat Compliance Guard (Vanguard, EAC, BattlEye) & Steam auto-pause\n• One-click Steam Input & Big Picture activation",
            releaseUrl: "https://github.com/orionopenwow/ThumbDesk/releases/tag/v1.2.0",
            downloadedBytes: 5120000,
            totalBytes: 5120000,
            percent: 100,
            verifiedSha256: "9f29470d804087345f8a702e09f8cfe5c15dc1b33aa97e164566a3542d735432",
            stagedPath: "C:\\Users\\demo\\AppData\\Local\\ThumbDesk\\updates\\ThumbDesk-v1.2.0.exe",
            message: "v1.2.0 preloaded & SHA-256 verified",
          }
        : {
            phase: "upToDate",
            currentVersion: "2.0.0",
            latestVersion: "v2.0.0",
            releaseNotes: null,
            releaseUrl: null,
            downloadedBytes: 0,
            totalBytes: 0,
            percent: 0,
            verifiedSha256: null,
            stagedPath: null,
            message: "Up to date (v2.0.0)",
          },
    voiceState: {
      enabled: true,
      listening: true,
      lastHeard: isMod ? "open terminal" : null,
      lastExecuted: isMod ? "wt.exe" : null,
      statusMessage: "Listening (offline Windows Speech)",
      rules: [
        { id: "r1", phrase: "open terminal", command: "wt.exe", enabled: true },
        { id: "r2", phrase: "open code", command: "code .", enabled: true },
        { id: "r3", phrase: "lock screen", command: "rundll32.exe user32.dll,LockWorkStation", enabled: true },
      ],
    },
    gameGuardPrefs: {
      antiCheatAutoPause: true,
      steamGameAutoPause: true,
    },
    gameGuardStatus: {
      isSuspended: false,
      reason: null,
      detectedAntiCheat: null,
      steamRunning: true,
      steamGameActive: false,
    },
    hardwareFx: {
      leftTriggerMode: "softClick",
      rightTriggerMode: "dualStage",
      lightbarMode: "autoTelemetry",
      customRgb: [56, 189, 248],
    },
    radialMenuOpen: preset === "radial",
    automationApiUrl: "http://127.0.0.1:49152",
  };
}
