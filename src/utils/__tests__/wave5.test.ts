import { describe, expect, it } from "vitest";
import {
  computeCalloutLayout,
  getStickVector,
  hotspotsForFamily,
  resolveEffectiveMapping,
} from "../controllerLayout";
import {
  getShellColorSpec,
  loadSavedShellColor,
  recolorControllerSvg,
  saveShellColor,
} from "../shellColors";
import { formatShortcut } from "../shortcuts";
import type { ControllerProfile, ControllerSnapshot } from "../../types/models";

describe("Wave 5: Controller Layout, Shell Colors & Shortcuts", () => {
  it("provides 18 hotspots and balanced 9+9 callout lanes for both Xbox and PlayStation", () => {
    expect(hotspotsForFamily("xbox")).toHaveLength(18);
    expect(hotspotsForFamily("playStation")).toHaveLength(18);

    const xboxLayout = computeCalloutLayout("xbox", 1000, 500);
    expect(xboxLayout.items.filter((i) => i.side === "left")).toHaveLength(9);
    expect(xboxLayout.items.filter((i) => i.side === "right")).toHaveLength(9);

    const psLayout = computeCalloutLayout("playStation", 1000, 500);
    expect(psLayout.items.filter((i) => i.side === "left")).toHaveLength(9);
    expect(psLayout.items.filter((i) => i.side === "right")).toHaveLength(9);
  });

  it("recolors SVG shell gradients and persists Xbox and PlayStation colors separately", () => {
    const mockSvg = `<svg xmlns="http://www.w3.org/2000/svg"><defs><linearGradient id="shell"><stop stop-color="#111"/><stop stop-color="#222"/><stop stop-color="#333"/></linearGradient></defs><path id="shell-rim" fill="#000"/></svg>`;
    const blue = recolorControllerSvg(mockSvg, "xbox", "blue");
    expect(blue).toContain("#6CAFFF");
    expect(blue).toContain("#2874D4");
    expect(blue).toContain("#154580");

    saveShellColor("xbox", "green");
    saveShellColor("playStation", "pink");
    expect(loadSavedShellColor("xbox")).toBe("green");
    expect(loadSavedShellColor("playStation")).toBe("pink");
    expect(getShellColorSpec("original", "playStation").usesDarkCheckmark).toBe(true);
    expect(getShellColorSpec("original", "xbox").usesDarkCheckmark).toBe(false);
  });

  it("normalizes live stick vectors and formats Windows shortcuts", () => {
    const snap: ControllerSnapshot = {
      isConnected: true,
      name: "Xbox",
      family: "xbox",
      transport: "XInput",
      batteryLevel: null,
      leftStick: { x: 1, y: 1, pressed: false },
      rightStick: { x: 0, y: 0, pressed: false },
      pressedControls: [],
      analogValues: {},
      eventCount: 1,
      shareSupported: false,
    };
    const vec = getStickVector(snap, "left");
    expect(Math.hypot(vec.dx, vec.dy)).toBeCloseTo(1.0, 5);
    expect(vec.dy).toBeLessThan(0); // +Y stick maps to -Y screen

    expect(formatShortcut({ keyCode: 0x56, modifiers: ["control"] })).toBe("Ctrl+V");
    expect(formatShortcut({ keyCode: 0xa3, modifiers: ["control"] })).toBe("LCtrl + RCtrl");

    const profile: ControllerProfile = {
      id: "p1",
      name: "Test",
      cursor: {
        primaryStick: "left",
        precisionStick: "right",
        primarySpeed: 2200,
        precisionSpeed: 560,
        deadZone: 0.12,
        responseCurve: 1.8,
        smoothing: 0.5,
        accelerationEnabled: true,
        invertPrimaryX: false,
        invertPrimaryY: false,
        invertPrecisionX: false,
        invertPrecisionY: false,
        horizontalSpeedMultiplier: 1,
        verticalSpeedMultiplier: 1,
        flickBoostEnabled: false,
        zoomGestureEnabled: false,
      },
      mappings: {
        buttonSouth: {
          actionType: "leftClick",
          triggerMode: "tap",
          repeatDelay: 0.35,
          repeatInterval: 0.08,
          vibration: "none",
        },
      },
      modifierLayers: [],
      applicationMappings: [],
    };
    expect(resolveEffectiveMapping(profile, "buttonSouth", null, null).actionType).toBe(
      "leftClick"
    );
  });
});
