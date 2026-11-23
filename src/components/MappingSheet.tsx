import React, { useEffect, useState } from "react";
import { Layers, Play, Sliders, X } from "lucide-react";
import type {
  ActionType,
  ControllerActionMapping,
  ControllerControlID,
  ControllerFamily,
  ControllerVibration,
  HapticsPreferences,
  HapticsStatus,
  ShortcutDescriptor,
} from "../types/models";
import { controlDisplayName } from "../utils/controllerLayout";
import {
  ACTION_TYPE_OPTIONS,
  browserKeyEventToShortcut,
  defaultTriggerModeForAction,
  mappingSummary,
  supportedTriggerModesForAction,
  TRIGGER_MODE_LABELS,
  VIBRATION_OPTIONS,
} from "../utils/shortcuts";
import { KeycapBadge } from "./KeycapBadge";
import { StudioSlider } from "./StudioSlider";

interface MappingSheetProps {
  control: ControllerControlID;
  family: ControllerFamily;
  modifierControl: ControllerControlID | null;
  appName: string | null;
  initialMapping: ControllerActionMapping;
  hasOverride: boolean;
  hapticsPrefs: HapticsPreferences;
  hapticsStatus: HapticsStatus;
  onPreviewVibration: (pattern: ControllerVibration) => void;
  onEnableHaptics: () => void;
  onClearOverride: () => void;
  onSave: (mapping: ControllerActionMapping) => void;
  onClose: () => void;
}

const SYSTEM_QUICK_KEYS: { label: string; shortcut: ShortcutDescriptor | null }[] = [
  { label: "Enter", shortcut: { keyCode: 0x0d, modifiers: [] } },
  { label: "Esc", shortcut: { keyCode: 0x1b, modifiers: [] } },
  { label: "Win+H", shortcut: { keyCode: 0x48, modifiers: ["win"] } },
  { label: "Win+Shift+S", shortcut: { keyCode: 0x53, modifiers: ["shift", "win"] } },
  { label: "Win+Shift+T", shortcut: { keyCode: 0x54, modifiers: ["shift", "win"] } },
  { label: "Backspace", shortcut: { keyCode: 0x08, modifiers: [] } },
  { label: "Delete", shortcut: { keyCode: 0x2e, modifiers: [] } },
  { label: "LCtrl+RCtrl", shortcut: { keyCode: 0xa3, modifiers: ["control"] } },
  { label: "Clear", shortcut: null },
];

const WaveformBars: React.FC<{ pattern: ControllerVibration; selected: boolean }> = ({
  pattern,
  selected,
}) => {
  const color = selected ? "#609BFF" : "rgba(152, 162, 179, 0.55)";
  const bars: Record<ControllerVibration, number[]> = {
    none: [2, 2, 2, 2],
    softTap: [3, 10, 3, 2],
    doubleTap: [10, 2, 9, 2],
    thump: [6, 15, 15, 6],
    leftPulse: [15, 11, 2, 2],
    rightPulse: [2, 2, 11, 15],
    grabRelease: [12, 3, 3, 8],
  };
  return (
    <div style={{ display: "flex", alignItems: "flex-end", gap: 3, height: 16 }}>
      {bars[pattern].map((h, i) => (
        <span
          key={i}
          style={{
            width: 4,
            height: h,
            borderRadius: 2,
            background: color,
          }}
        />
      ))}
    </div>
  );
};

export const MappingSheet: React.FC<MappingSheetProps> = ({
  control,
  family,
  modifierControl,
  appName,
  initialMapping,
  hasOverride,
  hapticsPrefs,
  hapticsStatus,
  onPreviewVibration,
  onEnableHaptics,
  onClearOverride,
  onSave,
  onClose,
}) => {
  const [draft, setDraft] = useState<ControllerActionMapping>({ ...initialMapping });
  const [isCapturing, setIsCapturing] = useState(false);
  const [showSystemKeys, setShowSystemKeys] = useState(false);

  useEffect(() => {
    if (!isCapturing) return;
    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.code === "Escape" && !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) {
        setIsCapturing(false);
        return;
      }
      const sc = browserKeyEventToShortcut(e);
      if (sc) {
        setDraft((prev) => ({ ...prev, shortcut: sc }));
        setIsCapturing(false);
      }
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [isCapturing]);

  const supportedModes = supportedTriggerModesForAction(draft.actionType);
  const handleActionTypeChange = (next: ActionType) => {
    const allowed = supportedTriggerModesForAction(next);
    const nextMode = allowed.includes(draft.triggerMode)
      ? draft.triggerMode
      : defaultTriggerModeForAction(next);
    setDraft((prev) => ({ ...prev, actionType: next, triggerMode: nextMode }));
  };

  const titleParts: string[] = [];
  if (modifierControl) titleParts.push(`${controlDisplayName(modifierControl, family)} +`);
  titleParts.push(controlDisplayName(control, family));

  const canSave = draft.actionType !== "keyboardShortcut" || Boolean(draft.shortcut);
  const vibMeta = VIBRATION_OPTIONS.find((v) => v.value === draft.vibration);

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal-card"
        style={{ width: 620 }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ padding: "20px 24px 16px", borderBottom: "1px solid var(--border-subtle)" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
            <div
              style={{
                width: 46,
                height: 46,
                borderRadius: 13,
                background: "var(--accent-soft)",
                border: "1px solid rgba(69, 135, 247, 0.3)",
                color: "var(--accent-bright)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Sliders size={21} />
            </div>
            <div style={{ flex: 1 }}>
              <div className="eyebrow">BUTTON MAPPING</div>
              <h2 style={{ margin: "2px 0 0", fontSize: 20, fontWeight: 600 }}>
                {titleParts.join(" ")}
              </h2>
              <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 6, flexWrap: "wrap" }}>
                <span className="status-badge" style={{ padding: "2px 9px", fontSize: 11, background: "rgba(255,255,255,0.05)", color: "var(--text-secondary)" }}>
                  {appName ?? "All Apps"}
                </span>
                {modifierControl && (
                  <span className="status-badge" style={{ padding: "2px 9px", fontSize: 11, background: "var(--accent-soft)", color: "#82B2FF" }}>
                    <Layers size={11} /> While holding {controlDisplayName(modifierControl, family)}
                  </span>
                )}
                <span style={{ fontSize: 11.5, color: "var(--text-tertiary)", marginLeft: 4 }}>
                  Current: {mappingSummary(initialMapping)}
                </span>
              </div>
            </div>
            <button type="button" className="btn btn-sm" onClick={onClose} aria-label="Close">
              <X size={16} />
            </button>
          </div>
        </div>

        <div style={{ padding: 22, overflowY: "auto", display: "flex", flexDirection: "column", gap: 16 }}>
          {/* Action type */}
          <label style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            <span style={{ fontSize: 12, color: "var(--text-secondary)", fontWeight: 500 }}>
              Action type
            </span>
            <select
              value={draft.actionType}
              onChange={(e) => handleActionTypeChange(e.target.value as ActionType)}
            >
              {ACTION_TYPE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </label>

          {/* Glowing Visual Shortcut Capture */}
          {draft.actionType === "keyboardShortcut" && (
            <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 10 }}>
              <span style={{ fontSize: 12, color: "var(--text-secondary)", fontWeight: 500 }}>
                Keyboard shortcut
              </span>
              <button
                type="button"
                onClick={() => setIsCapturing((v) => !v)}
                style={{
                  minHeight: 54,
                  borderRadius: 12,
                  padding: "10px 14px",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  background: isCapturing ? "rgba(69, 135, 247, 0.14)" : "rgba(0, 0, 0, 0.28)",
                  border: `1.5px solid ${isCapturing ? "var(--accent)" : "var(--border-subtle)"}`,
                  boxShadow: isCapturing ? "0 0 20px rgba(69, 135, 247, 0.28)" : "none",
                  transition: "all 0.15s ease",
                }}
              >
                {isCapturing ? (
                  <span style={{ color: "#82B2FF", fontWeight: 600, fontSize: 13 }}>
                    Press any key combination now… (Esc to cancel)
                  </span>
                ) : draft.shortcut ? (
                  <KeycapBadge mapping={draft} large />
                ) : (
                  <span style={{ color: "var(--text-tertiary)", fontSize: 13 }}>
                    Click to capture shortcut
                  </span>
                )}
                <span className="eyebrow" style={{ fontSize: 9.5 }}>
                  {isCapturing ? "LISTENING" : "CLICK TO RECORD"}
                </span>
              </button>

              <button
                type="button"
                onClick={() => setShowSystemKeys((v) => !v)}
                style={{
                  alignSelf: "flex-start",
                  fontSize: 12,
                  color: "#82B2FF",
                  fontWeight: 500,
                }}
              >
                {showSystemKeys ? "Hide system & function keys" : "System & function keys…"}
              </button>

              {showSystemKeys && (
                <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                  <div style={{ display: "grid", gridTemplateColumns: "repeat(6, 1fr)", gap: 6 }}>
                    {Array.from({ length: 12 }, (_, i) => i + 1).map((n) => (
                      <button
                        key={n}
                        type="button"
                        className="btn btn-sm"
                        onClick={() =>
                          setDraft((prev) => ({
                            ...prev,
                            shortcut: { keyCode: 0x6f + n, modifiers: [] },
                          }))
                        }
                      >
                        F{n}
                      </button>
                    ))}
                  </div>
                  <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 6 }}>
                    {SYSTEM_QUICK_KEYS.map((item) => (
                      <button
                        key={item.label}
                        type="button"
                        className="btn btn-sm"
                        onClick={() =>
                          setDraft((prev) => ({ ...prev, shortcut: item.shortcut }))
                        }
                      >
                        {item.label}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Segmented Trigger Mode Selector */}
          <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            <span style={{ fontSize: 12, color: "var(--text-secondary)", fontWeight: 500 }}>
              Trigger mode
            </span>
            <div className="segmented" style={{ alignSelf: "flex-start" }}>
              {supportedModes.map((m) => (
                <button
                  key={m}
                  type="button"
                  className={`segmented-btn ${draft.triggerMode === m ? "active" : ""}`}
                  onClick={() => setDraft((prev) => ({ ...prev, triggerMode: m }))}
                >
                  {TRIGGER_MODE_LABELS[m]}
                </button>
              ))}
            </div>
          </div>

          {/* Repeat timing sliders */}
          {draft.triggerMode === "repeatWhileHeld" && (
            <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 12 }}>
              <StudioSlider
                label="Repeat delay"
                valueLabel={`${draft.repeatDelay.toFixed(2)}s`}
                min={0.05}
                max={1.2}
                step={0.01}
                value={draft.repeatDelay}
                onChange={(v) => setDraft((prev) => ({ ...prev, repeatDelay: v }))}
              />
              <StudioSlider
                label="Repeat interval"
                valueLabel={`${draft.repeatInterval.toFixed(2)}s`}
                min={0.02}
                max={0.4}
                step={0.01}
                value={draft.repeatInterval}
                onChange={(v) => setDraft((prev) => ({ ...prev, repeatInterval: v }))}
              />
            </div>
          )}

          {/* Visual 7-Card Vibration Pattern Selector */}
          <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 10 }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <span style={{ fontSize: 12, color: "var(--text-secondary)", fontWeight: 500 }}>
                Vibration pattern
              </span>
              <button
                type="button"
                className="btn btn-sm"
                disabled={draft.vibration === "none" || !hapticsPrefs.enabled || !hapticsStatus.canPlay}
                onClick={() => onPreviewVibration(draft.vibration)}
              >
                <Play size={12} /> Test vibration
              </button>
            </div>
            <div className="vib-grid">
              {VIBRATION_OPTIONS.map((v) => {
                const selected = draft.vibration === v.value;
                return (
                  <button
                    key={v.value}
                    type="button"
                    className={`vib-card ${selected ? "selected" : ""}`}
                    onClick={() =>
                      setDraft((prev) => ({ ...prev, vibration: v.value }))
                    }
                  >
                    <WaveformBars pattern={v.value} selected={selected} />
                    <span
                      style={{
                        fontSize: 11,
                        fontWeight: selected ? 600 : 500,
                        color: selected ? "#82B2FF" : "var(--text-secondary)",
                      }}
                    >
                      {v.label}
                    </span>
                  </button>
                );
              })}
            </div>
            <div style={{ fontSize: 11.5, color: "var(--text-secondary)" }}>
              {vibMeta?.detail} Previewing never triggers the mapped command.
            </div>
            {!hapticsPrefs.enabled && (
              <button type="button" className="btn btn-sm" onClick={onEnableHaptics}>
                Enable vibration
              </button>
            )}
          </div>
        </div>

        <div
          style={{
            padding: "14px 24px",
            borderTop: "1px solid var(--border-subtle)",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <div>
            {hasOverride && (
              <button type="button" className="btn" onClick={onClearOverride}>
                {appName ? "Use All Apps" : "Use Default"}
              </button>
            )}
          </div>
          <div style={{ display: "flex", gap: 8 }}>
            <button type="button" className="btn" onClick={onClose}>
              Cancel
            </button>
            <button
              type="button"
              className="btn btn-primary"
              disabled={!canSave}
              onClick={() => onSave(draft)}
            >
              {appName ? "Save App Override" : modifierControl ? "Save Override" : "Save"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
