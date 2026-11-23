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
  | "toggleCursorSpeeds";

export interface ControllerActionMapping {
  actionType: ActionType;
  shortcut?: ShortcutDescriptor | null;
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
}

export interface ControllerProfile {
  id: string;
  name: string;
  cursor: CursorConfiguration;
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
}
