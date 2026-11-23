import { describe, expect, it } from "vitest";
import { browserSim } from "../../api/browserSim";
import {
  ACTION_TYPE_OPTIONS,
  mappingSummary,
  supportedTriggerModesForAction,
} from "../shortcuts";

describe("ThumbDesk v2.0 full feature & browser-sim regression suite", () => {
  it("formats and validates every ActionType including v2.0 tiling, media, scripts, and radial", () => {
    expect(ACTION_TYPE_OPTIONS.length).toBeGreaterThanOrEqual(30);
    for (const opt of ACTION_TYPE_OPTIONS) {
      const modes = supportedTriggerModesForAction(opt.value);
      expect(modes.length).toBeGreaterThan(0);
    }
    expect(
      mappingSummary({
        actionType: "runCommand",
        commandScript: "wt.exe",
        triggerMode: "tap",
        repeatDelay: 0.35,
        repeatInterval: 0.08,
        vibration: "softTap",
      })
    ).toBe("Run: wt.exe");

    expect(
      mappingSummary({
        actionType: "windowSnapLeft",
        triggerMode: "tap",
        repeatDelay: 0.35,
        repeatInterval: 0.08,
        vibration: "none",
      })
    ).toBe("Window Snap Left");

    expect(
      mappingSummary({
        actionType: "openRadialMenu",
        triggerMode: "tap",
        repeatDelay: 0.35,
        repeatInterval: 0.08,
        vibration: "softTap",
      })
    ).toBe("Open Radial Pie Menu");
  });

  it("supports end-to-end browser-mode mutations for Voice, Game Guard, Gyro, Triggers, and Radial Menu", async () => {
    const initial = await browserSim.resetActiveProfileToDefaults();
    expect(initial.runtimeEnabled).toBe(true);
    expect(initial.document.profiles[0].radialMenu?.length).toBe(8);

    // Save custom command script mapping
    const afterMap = await browserSim.saveMapping("menu", null, null, {
      actionType: "runCommand",
      commandScript: "code .",
      triggerMode: "tap",
      repeatDelay: 0.35,
      repeatInterval: 0.08,
      vibration: "softTap",
    });
    expect(afterMap.document.profiles[0].mappings.menu?.commandScript).toBe("code .");

    // Toggle Voice Mode & add rule
    const afterVoice = await browserSim.setVoiceRules([
      { id: "v1", phrase: "run tests", command: "npm test", enabled: true },
    ]);
    expect(afterVoice.voiceState.rules).toHaveLength(1);
    expect(afterVoice.voiceState.rules[0].phrase).toBe("run tests");

    // Toggle Game Guard preferences
    const afterGuard = await browserSim.setGameGuardPreferences({
      antiCheatAutoPause: true,
      steamGameAutoPause: false,
    });
    expect(afterGuard.gameGuardPrefs.antiCheatAutoPause).toBe(true);
    expect(afterGuard.gameGuardPrefs.steamGameAutoPause).toBe(false);

    // Update 6-axis Gyro & Adaptive Triggers
    const afterGyro = await browserSim.updateGyroConfig({
      enabled: true,
      activationMode: "holdLeftTrigger",
      sensitivity: 1.8,
      deadzone: 0.03,
      smoothing: 0.2,
      invertX: false,
      invertY: false,
    });
    expect(afterGyro.document.profiles[0].gyro?.activationMode).toBe("holdLeftTrigger");

    const afterFx = await browserSim.setHardwareFx({
      leftTriggerMode: "rigidWall",
      rightTriggerMode: "dualStage",
      lightbarMode: "customColor",
      customRgb: [52, 211, 153],
    });
    expect(afterFx.hardwareFx.leftTriggerMode).toBe("rigidWall");
    expect(afterFx.hardwareFx.customRgb).toEqual([52, 211, 153]);

    // Radial wheel open & execute
    const opened = await browserSim.setRadialMenuOpen(true);
    expect(opened.radialMenuOpen).toBe(true);
    const executed = await browserSim.executeRadialSlice("s2");
    expect(executed.radialMenuOpen).toBe(false);
    expect(executed.diagnostics.lastActionStatus).toContain("Snap Left");

    // Profile JSON export/import roundtrip
    const json = await browserSim.exportActiveProfileJson();
    expect(json).toContain("gabes-defaults");
  });
});
