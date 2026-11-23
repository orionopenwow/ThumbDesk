import type {
  ControllerActionMapping,
  ControllerControlID,
  ControllerFamily,
  ControllerProfile,
  ControllerSnapshot,
  StickSide,
} from "../types/models";

export const ARTWORK_DESIGN_WIDTH = 1000;
export const ARTWORK_DESIGN_HEIGHT = 660;
export const ARTWORK_ASPECT = ARTWORK_DESIGN_HEIGHT / ARTWORK_DESIGN_WIDTH;

export interface HotspotRegion {
  control: ControllerControlID;
  x: number;
  y: number;
  w: number;
  h: number;
  r: number;
}

export const XBOX_HOTSPOTS: HotspotRegion[] = [
  { control: "leftTrigger", x: 278, y: 72, w: 96, h: 42, r: 16 },
  { control: "rightTrigger", x: 722, y: 72, w: 96, h: 42, r: 16 },
  { control: "leftShoulder", x: 295, y: 118, w: 138, h: 36, r: 16 },
  { control: "rightShoulder", x: 705, y: 118, w: 138, h: 36, r: 16 },
  { control: "leftThumbstickButton", x: 287, y: 236, w: 112, h: 112, r: 56 },
  { control: "rightThumbstickButton", x: 608, y: 358, w: 112, h: 112, r: 56 },
  { control: "home", x: 500, y: 152, w: 58, h: 58, r: 29 },
  { control: "options", x: 441, y: 232, w: 42, h: 42, r: 21 },
  { control: "menu", x: 559, y: 232, w: 42, h: 42, r: 21 },
  { control: "buttonNorth", x: 710, y: 178, w: 54, h: 54, r: 27 },
  { control: "buttonEast", x: 769, y: 236, w: 54, h: 54, r: 27 },
  { control: "buttonSouth", x: 710, y: 294, w: 54, h: 54, r: 27 },
  { control: "buttonWest", x: 652, y: 236, w: 54, h: 54, r: 27 },
  { control: "dpadUp", x: 392, y: 326, w: 38, h: 40, r: 8 },
  { control: "dpadDown", x: 392, y: 404, w: 38, h: 40, r: 8 },
  { control: "dpadLeft", x: 352, y: 365, w: 40, h: 34, r: 8 },
  { control: "dpadRight", x: 432, y: 365, w: 40, h: 34, r: 8 },
  { control: "share", x: 500, y: 280, w: 46, h: 34, r: 14 },
];

export const PLAYSTATION_HOTSPOTS: HotspotRegion[] = [
  { control: "leftTrigger", x: 248, y: 108, w: 108, h: 36, r: 16 },
  { control: "rightTrigger", x: 752, y: 108, w: 108, h: 36, r: 16 },
  { control: "leftShoulder", x: 248, y: 132, w: 120, h: 32, r: 14 },
  { control: "rightShoulder", x: 752, y: 132, w: 120, h: 32, r: 14 },
  { control: "leftThumbstickButton", x: 369, y: 355, w: 108, h: 108, r: 54 },
  { control: "rightThumbstickButton", x: 631, y: 355, w: 108, h: 108, r: 54 },
  { control: "home", x: 500, y: 354, w: 48, h: 48, r: 24 },
  { control: "options", x: 309, y: 159, w: 34, h: 42, r: 12 },
  { control: "menu", x: 692, y: 159, w: 34, h: 42, r: 12 },
  { control: "buttonNorth", x: 756, y: 187, w: 54, h: 54, r: 27 },
  { control: "buttonEast", x: 816, y: 246, w: 54, h: 54, r: 27 },
  { control: "buttonSouth", x: 756, y: 306, w: 54, h: 54, r: 27 },
  { control: "buttonWest", x: 696, y: 246, w: 54, h: 54, r: 27 },
  { control: "dpadUp", x: 245, y: 206, w: 40, h: 42, r: 8 },
  { control: "dpadDown", x: 245, y: 286, w: 40, h: 42, r: 8 },
  { control: "dpadLeft", x: 202, y: 246, w: 42, h: 36, r: 8 },
  { control: "dpadRight", x: 288, y: 246, w: 42, h: 36, r: 8 },
  { control: "touchpadButton", x: 500, y: 195, w: 240, h: 124, r: 20 },
];

export function hotspotsForFamily(family: ControllerFamily): HotspotRegion[] {
  return family === "playStation" ? PLAYSTATION_HOTSPOTS : XBOX_HOTSPOTS;
}

const XBOX_LEFT_LANE: ControllerControlID[] = [
  "leftTrigger",
  "leftShoulder",
  "home",
  "leftThumbstickButton",
  "options",
  "dpadUp",
  "dpadLeft",
  "dpadRight",
  "dpadDown",
];

const PS_LEFT_LANE: ControllerControlID[] = [
  "leftTrigger",
  "leftShoulder",
  "options",
  "touchpadButton",
  "dpadUp",
  "dpadLeft",
  "dpadRight",
  "dpadDown",
  "leftThumbstickButton",
];

export interface CalloutItem {
  control: ControllerControlID;
  side: "left" | "right";
  labelX: number;
  labelY: number;
  labelW: number;
  labelH: number;
  originX: number;
  originY: number;
  elbowX: number;
  elbowY: number;
  anchorX: number;
  anchorY: number;
}

export interface CalloutLayoutResult {
  artworkX: number;
  artworkY: number;
  artworkW: number;
  artworkH: number;
  items: CalloutItem[];
}

export function computeCalloutLayout(
  family: ControllerFamily,
  width: number,
  height: number
): CalloutLayoutResult {
  const regions = hotspotsForFamily(family);
  const byId = new Map(regions.map((r) => [r.control, r]));
  const leftIds = family === "playStation" ? PS_LEFT_LANE : XBOX_LEFT_LANE;
  const leftSet = new Set(leftIds);
  const rightRegions = regions
    .filter((r) => !leftSet.has(r.control))
    .sort((a, b) => (a.y !== b.y ? a.y - b.y : a.x - b.x));

  const labelW = Math.min(160, width * 0.185);
  const availableW = Math.max(1, width - 2 * (labelW + 24));
  const artworkW = Math.min(availableW, (height - 4) / ARTWORK_ASPECT);
  const artworkH = artworkW * ARTWORK_ASPECT;
  const artworkX = (width - artworkW) / 2;
  const artworkY = (height - artworkH) / 2;

  const rowCount = Math.max(leftIds.length, rightRegions.length);
  const rowH = (height - 12) / rowCount;
  const items: CalloutItem[] = [];

  const pushSide = (controls: ControllerControlID[], side: "left" | "right") => {
    controls.forEach((ctrl, idx) => {
      const region = byId.get(ctrl);
      if (!region) return;
      const cy = 6 + rowH * (idx + 0.5);
      const labelH = 34;
      const labelX = side === "left" ? 4 : width - labelW - 4;
      const labelY = cy - labelH / 2;
      const originX = side === "left" ? labelX + labelW + 2 : labelX - 2;
      const anchorX = artworkX + (region.x * artworkW) / ARTWORK_DESIGN_WIDTH;
      const anchorY = artworkY + (region.y * artworkH) / ARTWORK_DESIGN_HEIGHT;
      const elbowRun = Math.max(18, Math.min(44, Math.abs(anchorX - originX) * 0.38));
      const elbowX = originX + (side === "left" ? elbowRun : -elbowRun);
      items.push({
        control: ctrl,
        side,
        labelX,
        labelY,
        labelW,
        labelH,
        originX,
        originY: cy,
        elbowX,
        elbowY: cy,
        anchorX,
        anchorY,
      });
    });
  };

  pushSide(leftIds, "left");
  pushSide(
    rightRegions.map((r) => r.control),
    "right"
  );

  return { artworkX, artworkY, artworkW, artworkH, items };
}

export function controlDisplayName(
  control: ControllerControlID,
  family: ControllerFamily
): string {
  if (family === "playStation") {
    const ps: Partial<Record<ControllerControlID, string>> = {
      buttonSouth: "Cross",
      buttonEast: "Circle",
      buttonWest: "Square",
      buttonNorth: "Triangle",
      leftShoulder: "L1",
      rightShoulder: "R1",
      leftTrigger: "L2",
      rightTrigger: "R2",
      leftThumbstickButton: "L3",
      rightThumbstickButton: "R3",
      menu: "Options",
      options: "Create",
      home: "PS",
    };
    if (ps[control]) return ps[control]!;
  }
  const xbox: Record<ControllerControlID, string> = {
    buttonSouth: "A",
    buttonEast: "B",
    buttonWest: "X",
    buttonNorth: "Y",
    leftShoulder: "LB",
    rightShoulder: "RB",
    leftTrigger: "LT",
    rightTrigger: "RT",
    dpadUp: "D-Pad Up",
    dpadDown: "D-Pad Down",
    dpadLeft: "D-Pad Left",
    dpadRight: "D-Pad Right",
    leftThumbstick: "Left Stick",
    rightThumbstick: "Right Stick",
    leftThumbstickButton: "L3",
    rightThumbstickButton: "R3",
    menu: "Menu",
    options: "View",
    home: "Home",
    share: "Share",
    touchpadButton: "Touchpad",
  };
  return xbox[control];
}

export function controlGlyph(
  control: ControllerControlID,
  family: ControllerFamily
): string {
  if (family === "playStation") {
    const ps: Partial<Record<ControllerControlID, string>> = {
      buttonSouth: "✕",
      buttonEast: "○",
      buttonWest: "□",
      buttonNorth: "△",
    };
    if (ps[control]) return ps[control]!;
  }
  const glyphs: Partial<Record<ControllerControlID, string>> = {
    dpadUp: "▲",
    dpadDown: "▼",
    dpadLeft: "◀",
    dpadRight: "▶",
  };
  return glyphs[control] ?? controlDisplayName(control, family);
}

export function getControlAmount(
  snapshot: ControllerSnapshot,
  control: ControllerControlID
): number {
  const analog = snapshot.analogValues[control];
  if (typeof analog === "number") {
    return Math.max(0, Math.min(1, analog));
  }
  if (control === "leftThumbstickButton") {
    return snapshot.leftStick.pressed || snapshot.pressedControls.includes(control) ? 1 : 0;
  }
  if (control === "rightThumbstickButton") {
    return snapshot.rightStick.pressed || snapshot.pressedControls.includes(control) ? 1 : 0;
  }
  return snapshot.pressedControls.includes(control) ? 1 : 0;
}

export function isControlActive(
  snapshot: ControllerSnapshot,
  control: ControllerControlID
): boolean {
  return getControlAmount(snapshot, control) > 0.02;
}

export function getStickVector(
  snapshot: ControllerSnapshot,
  side: StickSide
): { dx: number; dy: number } {
  const stick = side === "left" ? snapshot.leftStick : snapshot.rightStick;
  const x = Math.max(-1, Math.min(1, stick.x));
  const y = Math.max(-1, Math.min(1, stick.y));
  const mag = Math.hypot(x, y);
  const div = Math.max(1, mag);
  return { dx: x / div, dy: -(y / div) };
}

export function resolveEffectiveMapping(
  profile: ControllerProfile,
  control: ControllerControlID,
  modifierControl: ControllerControlID | null,
  appBundleId: string | null
): ControllerActionMapping {
  if (appBundleId) {
    const app = profile.applicationMappings.find(
      (a) => a.bundleIdentifier.toLowerCase() === appBundleId.toLowerCase()
    );
    if (app) {
      if (modifierControl) {
        const layer = app.modifierLayers.find((l) => l.modifierControl === modifierControl);
        if (layer?.mappings[control]) return layer.mappings[control]!;
      } else if (app.mappings[control]) {
        return app.mappings[control]!;
      }
    }
  }
  if (modifierControl) {
    const layer = profile.modifierLayers.find((l) => l.modifierControl === modifierControl);
    if (layer?.mappings[control]) return layer.mappings[control]!;
  }
  return (
    profile.mappings[control] ?? {
      actionType: "none",
      shortcut: null,
      triggerMode: "tap",
      repeatDelay: 0.35,
      repeatInterval: 0.08,
      vibration: "none",
    }
  );
}

export function hasExplicitOverride(
  profile: ControllerProfile,
  control: ControllerControlID,
  modifierControl: ControllerControlID | null,
  appBundleId: string | null
): boolean {
  if (appBundleId) {
    const app = profile.applicationMappings.find(
      (a) => a.bundleIdentifier.toLowerCase() === appBundleId.toLowerCase()
    );
    if (!app) return false;
    if (modifierControl) {
      const layer = app.modifierLayers.find((l) => l.modifierControl === modifierControl);
      return Boolean(layer?.mappings[control]);
    }
    return Boolean(app.mappings[control]);
  }
  if (modifierControl) {
    const layer = profile.modifierLayers.find((l) => l.modifierControl === modifierControl);
    return Boolean(layer?.mappings[control]);
  }
  return false;
}
