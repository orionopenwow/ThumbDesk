import type {
  ActionType,
  ControllerActionMapping,
  ControllerVibration,
  KeyboardModifier,
  ShortcutDescriptor,
  TriggerMode,
} from "../types/models";

const MOD_ORDER: KeyboardModifier[] = ["control", "alt", "shift", "win"];

const MOD_LABELS: Record<KeyboardModifier, string> = {
  control: "Ctrl",
  alt: "Alt",
  shift: "Shift",
  win: "Win",
};

const VK_NAMES: Record<number, string> = {
  0x08: "Backspace",
  0x09: "Tab",
  0x0d: "Enter",
  0x14: "Caps Lock",
  0x1b: "Esc",
  0x20: "Space",
  0x21: "PgUp",
  0x22: "PgDn",
  0x23: "End",
  0x24: "Home",
  0x25: "Left",
  0x26: "Up",
  0x27: "Right",
  0x28: "Down",
  0x2e: "Del",
  0x5b: "LWin",
  0x5c: "RWin",
  0xa0: "LShift",
  0xa1: "RShift",
  0xa2: "LCtrl",
  0xa3: "RCtrl",
  0xa4: "LAlt",
  0xa5: "RAlt",
  0x70: "F1",
  0x71: "F2",
  0x72: "F3",
  0x73: "F4",
  0x74: "F5",
  0x75: "F6",
  0x76: "F7",
  0x77: "F8",
  0x78: "F9",
  0x79: "F10",
  0x7a: "F11",
  0x7b: "F12",
  0xba: ";",
  0xbb: "=",
  0xbc: ",",
  0xbd: "-",
  0xbe: ".",
  0xbf: "/",
  0xc0: "`",
  0xdb: "[",
  0xdc: "\\",
  0xdd: "]",
  0xde: "'",
};

export function formatVkName(vk: number): string {
  if (VK_NAMES[vk]) return VK_NAMES[vk];
  if ((vk >= 0x30 && vk <= 0x39) || (vk >= 0x41 && vk <= 0x5a)) {
    return String.fromCharCode(vk);
  }
  return `Key 0x${vk.toString(16).toUpperCase()}`;
}

export function formatShortcut(shortcut: ShortcutDescriptor | null | undefined): string {
  if (!shortcut) return "Set shortcut";
  const mods = MOD_ORDER.filter((m) => shortcut.modifiers.includes(m));
  if (shortcut.keyCode === 0xa3 && mods.includes("control")) return "LCtrl + RCtrl";
  if (shortcut.keyCode === 0xa5 && mods.includes("alt")) return "LAlt + RAlt";
  if (shortcut.keyCode === 0xa1 && mods.includes("shift")) return "LShift + RShift";
  if (shortcut.keyCode === 0x5c && mods.includes("win")) return "LWin + RWin";
  const key = formatVkName(shortcut.keyCode);
  if (mods.length === 0) return key;
  return `${mods.map((m) => MOD_LABELS[m]).join("+")}+${key}`;
}

export const ACTION_TYPE_OPTIONS: { value: ActionType; label: string }[] = [
  { value: "none", label: "None" },
  { value: "keyboardShortcut", label: "Keyboard Shortcut" },
  { value: "leftClick", label: "Left Click" },
  { value: "rightClick", label: "Right Click" },
  { value: "middleClick", label: "Middle Click" },
  { value: "leftMouseHold", label: "Left Mouse Hold (drag)" },
  { value: "doubleClick", label: "Double Click" },
  { value: "scrollUp", label: "Scroll Up" },
  { value: "scrollDown", label: "Scroll Down" },
  { value: "scrollLeft", label: "Scroll Left" },
  { value: "scrollRight", label: "Scroll Right" },
  { value: "switchSpaceLeft", label: "Switch Desktop Left" },
  { value: "switchSpaceRight", label: "Switch Desktop Right" },
  { value: "crossEdgeLeft", label: "Cross Edge Left" },
  { value: "crossEdgeRight", label: "Cross Edge Right" },
  { value: "crossEdgeUp", label: "Cross Edge Up" },
  { value: "crossEdgeDown", label: "Cross Edge Down" },
  { value: "toggleCursorSpeeds", label: "Toggle Cursor Speeds" },
  { value: "runCommand", label: "Run Command / Script" },
  { value: "toggleVoiceMode", label: "Toggle Voice Mode" },
  { value: "openRadialMenu", label: "Open Radial Pie Menu" },
  { value: "windowSnapLeft", label: "Window Snap Left" },
  { value: "windowSnapRight", label: "Window Snap Right" },
  { value: "windowMaximize", label: "Window Maximize" },
  { value: "windowMinimize", label: "Window Minimize" },
  { value: "windowNextMonitor", label: "Move Window to Next Monitor" },
  { value: "mediaPlayPause", label: "Media Play / Pause" },
  { value: "mediaNext", label: "Media Next Track" },
  { value: "mediaPrev", label: "Media Previous Track" },
  { value: "volumeUp", label: "Volume Up" },
  { value: "volumeDown", label: "Volume Down" },
  { value: "volumeMute", label: "Volume Mute" },
];

export const TRIGGER_MODE_LABELS: Record<TriggerMode, string> = {
  tap: "Tap",
  holdWhilePressed: "Hold while pressed",
  repeatWhileHeld: "Repeat while held",
  toggle: "Toggle",
};

export function defaultTriggerModeForAction(actionType: ActionType): TriggerMode {
  if (actionType === "leftMouseHold") return "holdWhilePressed";
  if (
    actionType === "scrollUp" ||
    actionType === "scrollDown" ||
    actionType === "scrollLeft" ||
    actionType === "scrollRight"
  ) {
    return "repeatWhileHeld";
  }
  return "tap";
}

export function supportedTriggerModesForAction(actionType: ActionType): TriggerMode[] {
  switch (actionType) {
    case "none":
    case "crossEdgeLeft":
    case "crossEdgeRight":
    case "crossEdgeUp":
    case "crossEdgeDown":
    case "toggleCursorSpeeds":
    case "runCommand":
    case "toggleVoiceMode":
    case "openRadialMenu":
    case "windowSnapLeft":
    case "windowSnapRight":
    case "windowMaximize":
    case "windowMinimize":
    case "windowNextMonitor":
    case "mediaPlayPause":
    case "mediaNext":
    case "mediaPrev":
    case "volumeMute":
      return ["tap"];
    case "keyboardShortcut":
      return ["tap", "holdWhilePressed", "repeatWhileHeld"];
    case "leftMouseHold":
      return ["holdWhilePressed", "toggle"];
    default:
      return ["tap", "repeatWhileHeld"];
  }
}

export const VIBRATION_OPTIONS: {
  value: ControllerVibration;
  label: string;
  detail: string;
}[] = [
  { value: "none", label: "None", detail: "No vibration feedback." },
  { value: "softTap", label: "Soft tap", detail: "Light 75ms acknowledgment tap." },
  { value: "doubleTap", label: "Double tap", detail: "Two crisp taps for capture or connect." },
  { value: "thump", label: "Thump", detail: "Firm 120ms pulse for vertical edge or close." },
  { value: "leftPulse", label: "Left pulse", detail: "Left-handle pulse for crossing left edge." },
  { value: "rightPulse", label: "Right pulse", detail: "Right-handle pulse for crossing right edge." },
  { value: "grabRelease", label: "Grab & release", detail: "Start tap on press, lighter tap on release." },
];

export function mappingSummary(mapping: ControllerActionMapping): string {
  if (mapping.actionType === "none") return "No action";
  if (mapping.actionType === "keyboardShortcut") {
    return formatShortcut(mapping.shortcut);
  }
  if (mapping.actionType === "runCommand") {
    const cmd = mapping.commandScript?.trim();
    if (!cmd) return "Run Command";
    return cmd.length > 22 ? `Run: ${cmd.slice(0, 19)}…` : `Run: ${cmd}`;
  }
  return (
    ACTION_TYPE_OPTIONS.find((o) => o.value === mapping.actionType)?.label ??
    mapping.actionType
  );
}

export function browserKeyEventToShortcut(e: KeyboardEvent): ShortcutDescriptor | null {
  if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) {
    return null;
  }
  const mods: KeyboardModifier[] = [];
  if (e.ctrlKey) mods.push("control");
  if (e.altKey) mods.push("alt");
  if (e.shiftKey) mods.push("shift");
  if (e.metaKey) mods.push("win");

  const codeMap: Record<string, number> = {
    Backspace: 0x08,
    Tab: 0x09,
    Enter: 0x0d,
    Escape: 0x1b,
    Space: 0x20,
    PageUp: 0x21,
    PageDown: 0x22,
    End: 0x23,
    Home: 0x24,
    ArrowLeft: 0x25,
    ArrowUp: 0x26,
    ArrowRight: 0x27,
    ArrowDown: 0x28,
    Delete: 0x2e,
    Semicolon: 0xba,
    Equal: 0xbb,
    Comma: 0xbc,
    Minus: 0xbd,
    Period: 0xbe,
    Slash: 0xbf,
    Backquote: 0xc0,
    BracketLeft: 0xdb,
    Backslash: 0xdc,
    BracketRight: 0xdd,
    Quote: 0xde,
  };

  let vk = codeMap[e.code];
  if (!vk && /^Key([A-Z])$/.test(e.code)) {
    vk = e.code.charCodeAt(3);
  } else if (!vk && /^Digit([0-9])$/.test(e.code)) {
    vk = e.code.charCodeAt(5);
  } else if (!vk && /^F([1-9]|1[0-2])$/.test(e.code)) {
    vk = 0x6f + Number(e.code.slice(1));
  }
  if (!vk) return null;
  return { keyCode: vk, modifiers: mods };
}
