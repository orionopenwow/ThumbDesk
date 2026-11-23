export type ControllerFamily = "xbox" | "playStation" | "generic";
export type StickSide = "left" | "right";
export type StickAssignment = "off" | "left" | "right";
export type StickRoleChoice = "primary" | "precision" | "off";

export type ControllerControlID =
  | "buttonSouth"
  | "buttonEast"
  | "buttonWest"
  | "buttonNorth"
  | "leftShoulder"
  | "rightShoulder"
  | "leftTrigger"
  | "rightTrigger"
  | "dpadUp"
  | "dpadDown"
  | "dpadLeft"
  | "dpadRight"
  | "leftThumbstick"
  | "rightThumbstick"
  | "leftThumbstickButton"
  | "rightThumbstickButton"
  | "menu"
  | "options"
  | "home"
  | "share"
  | "touchpadButton";

export type ControllerVibration =
  | "none"
  | "softTap"
  | "doubleTap"
  | "thump"
  | "leftPulse"
  | "rightPulse"
  | "grabRelease";

export type KeyboardModifier = "control" | "alt" | "shift" | "win";

export interface ShortcutDescriptor {
  keyCode: number;
  modifiers: KeyboardModifier[];
}

export type TriggerMode =
  | "tap"
  | "holdWhilePressed"
  | "repeatWhileHeld"
  | "toggle";

export type ActionType =
  | "none"
  | "keyboardShortcut"
  | "leftClick"
  | "rightClick"
  | "middleClick"
  | "leftMouseHold"
  | "doubleClick"
  | "scrollUp"
  | "scrollDown"
  | "scrollLeft"
  | "scrollRight"
  | "switchSpaceLeft"
  | "switchSpaceRight"
  | "crossEdgeLeft"
  | "crossEdgeRight"
  | "crossEdgeUp"
  | "crossEdgeDown"
  | "toggleCursorSpeeds"
  | "runCommand"
  | "toggleVoiceMode"
  | "windowSnapLeft"
  | "windowSnapRight"
  | "windowMaximize"
  | "windowMinimize"
  | "windowNextMonitor"
  | "mediaPlayPause"
  | "mediaNext"
  | "mediaPrev"
  | "volumeUp"
  | "volumeDown"
  | "volumeMute"
  | "openRadialMenu";

export interface ControllerActionMapping {
  actionType: ActionType;
  shortcut?: ShortcutDescriptor | null;
  commandScript?: string | null;
  triggerMode: TriggerMode;
  repeatDelay: number;
  repeatInterval: number;
  vibration: ControllerVibration;
}

export interface ControllerModifierLayer {
  modifierControl: ControllerControlID;
  mappings: Partial<Record<ControllerControlID, ControllerActionMapping>>;
}

export interface ApplicationMappingOverrides {
  bundleIdentifier: string;
  displayName: string;
  mappings: Partial<Record<ControllerControlID, ControllerActionMapping>>;
  modifierLayers: ControllerModifierLayer[];
}

export interface CursorConfiguration {
  primaryStick: StickAssignment;
  precisionStick: StickAssignment;
  primarySpeed: number;
  precisionSpeed: number;
  deadZone: number;
  responseCurve: number;
  smoothing: number;
  accelerationEnabled: boolean;
  invertPrimaryX: boolean;
  invertPrimaryY: boolean;
  invertPrecisionX: boolean;
  invertPrecisionY: boolean;
  horizontalSpeedMultiplier: number;
  verticalSpeedMultiplier: number;
  flickBoostEnabled: boolean;
  zoomGestureEnabled: boolean;
  magicSelectEnabled?: boolean;
  magicSnapRadius?: number;
}

export type GyroActivationMode =
  | "touchpadTouch"
  | "always"
  | "holdLeftTrigger"
  | "holdRightShoulder";

export interface GyroConfiguration {
  enabled: boolean;
  activationMode: GyroActivationMode;
  sensitivity: number;
  deadzone: number;
  smoothing: number;
  invertX: boolean;
  invertY: boolean;
}

export type AdaptiveTriggerMode =
  | "off"
  | "softClick"
  | "mediumDetent"
  | "rigidWall"
  | "dualStage";

export type LightbarMode = "autoTelemetry" | "customColor" | "off";

export interface HardwareFxPreferences {
  leftTriggerMode: AdaptiveTriggerMode;
  rightTriggerMode: AdaptiveTriggerMode;
  lightbarMode: LightbarMode;
  customRgb: [number, number, number];
}

export interface RadialMenuSlice {
  id: string;
  label: string;
  actionType: ActionType;
  commandScript?: string | null;
}

export interface ControllerProfile {
  id: string;
  name: string;
  cursor: CursorConfiguration;
  gyro?: GyroConfiguration;
  radialMenu?: RadialMenuSlice[];
  mappings: Partial<Record<ControllerControlID, ControllerActionMapping>>;
  modifierLayers: ControllerModifierLayer[];
  applicationMappings: ApplicationMappingOverrides[];
}

export interface ProfileDocument {
  version: number;
  profiles: ControllerProfile[];
  activeProfileId: string;
}

export interface StickSnapshot {
  x: number;
  y: number;
  pressed: boolean;
}

export interface ControllerSnapshot {
  isConnected: boolean;
  name: string | null;
  family: ControllerFamily;
  transport: string | null;
  batteryLevel: number | null;
  leftStick: StickSnapshot;
  rightStick: StickSnapshot;
  pressedControls: ControllerControlID[];
  analogValues: Partial<Record<ControllerControlID, number>>;
  eventCount: number;
  shareSupported: boolean;
  hasGyro?: boolean;
  gyro?: [number, number, number];
  touchpadTouched?: boolean;
  pollingRateHz?: number;
}

export interface HapticsPreferences {
  enabled: boolean;
  strength: number;
  connectionFeedback: boolean;
}

export interface HapticsStatus {
  canPlay: boolean;
  backend: string;
  message: string;
}

export type CompanionMode = "off" | "controller" | "receiver";
export type CompanionEdge = "left" | "right" | "top" | "bottom";

export interface CompanionPeer {
  id: string;
  name: string;
  address: string;
  port: number;
}

export type CompanionConnectionState =
  | { state: "off" }
  | { state: "browsing" }
  | { state: "listening" }
  | { state: "connecting"; detail: string }
  | { state: "connected"; detail: string }
  | { state: "error"; detail: string };

export interface RunningAppInfo {
  bundleIdentifier: string;
  displayName: string;
  executablePath: string;
}

export interface RuntimeDiagnostics {
  cursorState: "disabled" | "idle" | "moving";
  cursorMessage: string;
  lastActionStatus: string;
  activeAppId: string | null;
  activeAppName: string | null;
  isRoutingToCompanion: boolean;
}

export type UpdatePhase =
  | "idle"
  | "checking"
  | "upToDate"
  | "preloading"
  | "verifying"
  | "readyToRestart"
  | "applying"
  | "error";

export interface UpdateStatus {
  phase: UpdatePhase;
  currentVersion: string;
  latestVersion: string | null;
  releaseNotes: string | null;
  releaseUrl: string | null;
  downloadedBytes: number;
  totalBytes: number;
  percent: number;
  verifiedSha256: string | null;
  stagedPath: string | null;
  message: string;
}

export interface VoiceCommandRule {
  id: string;
  phrase: string;
  command: string;
  enabled: boolean;
}

export interface VoiceModeState {
  enabled: boolean;
  listening: boolean;
  lastHeard: string | null;
  lastExecuted: string | null;
  statusMessage: string;
  rules: VoiceCommandRule[];
}

export interface GameGuardPreferences {
  antiCheatAutoPause: boolean;
  steamGameAutoPause: boolean;
}

export interface GameGuardStatus {
  isSuspended: boolean;
  reason: string | null;
  detectedAntiCheat: string | null;
  steamRunning: boolean;
  steamGameActive: boolean;
}

export interface AppStatePayload {
  document: ProfileDocument;
  runtimeEnabled: boolean;
  hapticsPreferences: HapticsPreferences;
  hapticsStatus: HapticsStatus;
  companionMode: CompanionMode;
  companionEdge: CompanionEdge;
  companionState: CompanionConnectionState;
  companionPeers: CompanionPeer[];
  diagnostics: RuntimeDiagnostics;
  snapshot: ControllerSnapshot;
  updateStatus: UpdateStatus;
  voiceState: VoiceModeState;
  gameGuardPrefs: GameGuardPreferences;
  gameGuardStatus: GameGuardStatus;
  hardwareFx: HardwareFxPreferences;
  radialMenuOpen: boolean;
  automationApiUrl: string;
}
