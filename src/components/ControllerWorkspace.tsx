import React, { useMemo, useState } from "react";
import {
  AppWindow,
  Check,
  ChevronDown,
  ChevronRight,
  Gamepad2,
  Layers,
  List,
  Plus,
  Search,
  Trash2,
  Zap,
} from "lucide-react";
import type {
  ControllerControlID,
  ControllerFamily,
  ControllerProfile,
  ControllerSnapshot,
  RunningAppInfo,
  RuntimeDiagnostics,
  StickSide,
} from "../types/models";
import {
  controlDisplayName,
  hasExplicitOverride,
  hotspotsForFamily,
  resolveEffectiveMapping,
} from "../utils/controllerLayout";
import { mappingSummary, TRIGGER_MODE_LABELS } from "../utils/shortcuts";
import {
  getShellColorSpec,
  loadSavedShellColor,
  saveShellColor,
  SHELL_COLOR_OPTIONS,
  type ControllerShellColor,
} from "../utils/shellColors";
import { ControllerAnnotatedMap } from "./ControllerAnnotatedMap";
import { DiagnosticsPanel } from "./DiagnosticsPanel";
import { KeycapBadge } from "./KeycapBadge";

interface ControllerWorkspaceProps {
  profile: ControllerProfile;
  profiles: ControllerProfile[];
  activeProfileId: string;
  runtimeEnabled: boolean;
  statusLabel: string;
  statusColor: string;
  snapshot: ControllerSnapshot;
  diagnostics: RuntimeDiagnostics;
  selectedAppId: string | null;
  selectedModifier: ControllerControlID | null;
  onToggleRuntimeEnabled: () => void;
  onSelectProfile: (profileId: string) => void;
  onSelectAppId: (appId: string | null) => void;
  onSelectModifier: (mod: ControllerControlID | null) => void;
  onSelectControl: (control: ControllerControlID) => void;
  onSelectStickRole: (side: StickSide) => void;
  onAddModifierLayer: (control: ControllerControlID) => void;
  onRemoveModifierLayer: (control: ControllerControlID) => void;
  onLoadRunningApps: () => Promise<RunningAppInfo[]>;
  onAddAppScope: (bundleId: string, displayName: string) => void;
  onInstallCodexStarter: () => void;
  onResetAppScope: (bundleId: string) => void;
  onRemoveAppScope: (bundleId: string) => void;
  onTriggerSweep: () => void;
  onTriggerLeftClick: () => void;
}

export const ControllerWorkspace: React.FC<ControllerWorkspaceProps> = ({
  profile,
  profiles,
  activeProfileId,
  runtimeEnabled,
  statusLabel,
  statusColor,
  snapshot,
  diagnostics,
  selectedAppId,
  selectedModifier,
  onToggleRuntimeEnabled,
  onSelectProfile,
  onSelectAppId,
  onSelectModifier,
  onSelectControl,
  onSelectStickRole,
  onAddModifierLayer,
  onRemoveModifierLayer,
  onLoadRunningApps,
  onAddAppScope,
  onInstallCodexStarter,
  onResetAppScope,
  onRemoveAppScope,
  onTriggerSweep,
  onTriggerLeftClick,
}) => {
  const [showsList, setShowsList] = useState(false);
  const [search, setSearch] = useState("");
  const [layoutPreview, setLayoutPreview] = useState<"auto" | ControllerFamily>("auto");
  const [showsLabels, setShowsLabels] = useState<boolean>(() => {
    try {
      const saved = localStorage.getItem("controllerMap.showLabels");
      return saved === null ? true : saved === "true";
    } catch {
      return true;
    }
  });
  const [hoveredControl, setHoveredControl] = useState<ControllerControlID | null>(null);
  const [showColorPicker, setShowColorPicker] = useState(false);
  const [showAddAppMenu, setShowAddAppMenu] = useState(false);
  const [runningApps, setRunningApps] = useState<RunningAppInfo[]>([]);
  const [showAddModMenu, setShowAddModMenu] = useState(false);

  const effectiveFamily: ControllerFamily =
    layoutPreview === "auto"
      ? snapshot.family === "playStation"
        ? "playStation"
        : "xbox"
      : layoutPreview;

  const [shellColor, setShellColor] = useState<ControllerShellColor>(() =>
    loadSavedShellColor(effectiveFamily)
  );

  // Sync saved shell color when family switches
  React.useEffect(() => {
    setShellColor(loadSavedShellColor(effectiveFamily));
  }, [effectiveFamily]);

  const handleSetShellColor = (c: ControllerShellColor) => {
    setShellColor(c);
    saveShellColor(effectiveFamily, c);
    setShowColorPicker(false);
  };

  const handleToggleLabels = (val: boolean) => {
    setShowsLabels(val);
    try {
      localStorage.setItem("controllerMap.showLabels", String(val));
    } catch {
      // ignore
    }
  };

  // Live held modifier preview: if user holds a configured modifier on controller, preview its layer
  const liveHeldModifier = useMemo(() => {
    for (const layer of profile.modifierLayers) {
      if (snapshot.pressedControls.includes(layer.modifierControl)) {
        return layer.modifierControl;
      }
    }
    return null;
  }, [profile.modifierLayers, snapshot.pressedControls]);

  const visibleModifier = liveHeldModifier ?? selectedModifier;
  const regions = hotspotsForFamily(effectiveFamily);

  const filteredBindings = useMemo(() => {
    const q = search.trim().toLowerCase();
    return regions.filter((r) => {
      if (!q) return true;
      const name = controlDisplayName(r.control, effectiveFamily).toLowerCase();
      const summary = mappingSummary(
        resolveEffectiveMapping(profile, r.control, visibleModifier, selectedAppId)
      ).toLowerCase();
      return name.includes(q) || summary.includes(q);
    });
  }, [regions, search, effectiveFamily, profile, visibleModifier, selectedAppId]);

  const existingModIds = new Set(profile.modifierLayers.map((l) => l.modifierControl));
  const availableModControls = regions
    .map((r) => r.control)
    .filter((c) => !existingModIds.has(c));

  const activeColorSpec = getShellColorSpec(shellColor, effectiveFamily);

  const stickRoleLabel = (side: StickSide) => {
    if (profile.cursor.primaryStick === side) return "Primary cursor";
    if (profile.cursor.precisionStick === side) return "Precision cursor";
    return "Off";
  };

  const readoutControl =
    hoveredControl ??
    snapshot.pressedControls.find((c) => c !== visibleModifier) ??
    null;

  const overrideControlsForVisibleLayer = useMemo(() => {
    if (!visibleModifier) return [];
    return regions
      .map((r) => r.control)
      .filter((c) => hasExplicitOverride(profile, c, visibleModifier, selectedAppId));
  }, [regions, profile, visibleModifier, selectedAppId]);

  return (
    <div className="workspace-scroll">
      <div className="workspace-inner">
        {/* Flush Top Control Row: Status, Enabled, Profile + Controller/Bindings */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: 10,
            flexWrap: "wrap",
          }}
        >
          <div style={{ display: "inline-flex", alignItems: "center", gap: 10 }}>
            <h1 style={{ margin: 0, fontSize: 16.5, fontWeight: 600, letterSpacing: -0.3, display: "inline-flex", alignItems: "center", gap: 7 }}>
              <Gamepad2 size={15} color="var(--accent-bright)" /> Controller map
            </h1>
            <div
              className="status-badge"
              style={{
                padding: "3px 10px",
                fontSize: 11.5,
                background: `color-mix(in srgb, ${statusColor} 14%, transparent)`,
                borderColor: `color-mix(in srgb, ${statusColor} 32%, transparent)`,
                color: statusColor,
              }}
            >
              <span className="status-dot" style={{ background: statusColor }} />
              {statusLabel}
            </div>
            <div style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
              <button
                type="button"
                className={`toggle-switch ${runtimeEnabled ? "on" : ""}`}
                onClick={onToggleRuntimeEnabled}
                aria-label="Toggle input enabled"
              />
              <span style={{ fontSize: 12, fontWeight: 500, color: "var(--text-secondary)" }}>
                Enabled
              </span>
            </div>
          </div>

          <div style={{ display: "inline-flex", alignItems: "center", gap: 8 }}>
            <select
              style={{ minWidth: 155, padding: "5px 10px" }}
              value={activeProfileId}
              onChange={(e) => onSelectProfile(e.target.value)}
            >
              {profiles.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>

            <div className="segmented">
              <button
                type="button"
                className={`segmented-btn ${!showsList ? "active" : ""}`}
                onClick={() => setShowsList(false)}
              >
                <Gamepad2 size={12.5} /> Controller
              </button>
              <button
                type="button"
                className={`segmented-btn ${showsList ? "active" : ""}`}
                onClick={() => setShowsList(true)}
              >
                <List size={12.5} /> Bindings
              </button>
            </div>
          </div>
        </div>

        {/* Unified Single-Row App Scope & Modifier Layer Toolbar */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            flexWrap: "wrap",
            gap: 8,
            paddingTop: 5,
            borderTop: "1px solid var(--border-subtle)",
          }}
        >
          {/* Left: App Scope */}
          <div style={{ display: "flex", alignItems: "center", gap: 6, position: "relative" }}>
            <span style={{ fontSize: 12, fontWeight: 500, color: "var(--text-secondary)", display: "inline-flex", alignItems: "center", gap: 4 }}>
              <AppWindow size={13} color="var(--text-tertiary)" /> App:
            </span>
            <select
              style={{ minWidth: 130 }}
              value={selectedAppId ?? ""}
              onChange={(e) => onSelectAppId(e.target.value || null)}
            >
              <option value="">All Apps</option>
              {profile.applicationMappings.map((app) => (
                <option key={app.bundleIdentifier} value={app.bundleIdentifier}>
                  {app.displayName}
                </option>
              ))}
            </select>

            <div style={{ position: "relative" }}>
              <button
                type="button"
                className="btn btn-sm"
                onClick={async () => {
                  const next = !showAddAppMenu;
                  setShowAddAppMenu(next);
                  if (next) {
                    const apps = await onLoadRunningApps();
                    setRunningApps(apps);
                  }
                }}
              >
                <Plus size={13} /> Add App
              </button>
              {showAddAppMenu && (
                <div
                  className="modal-card"
                  style={{
                    position: "absolute",
                    top: 34,
                    left: 0,
                    width: 240,
                    padding: 6,
                    zIndex: 40,
                    maxHeight: 280,
                    overflowY: "auto",
                  }}
                >
                  <button
                    type="button"
                    className="btn btn-sm"
                    style={{ width: "100%", justifyContent: "flex-start" }}
                    onClick={() => {
                      onInstallCodexStarter();
                      onSelectAppId("com.openai.codex");
                      setShowAddAppMenu(false);
                    }}
                  >
                    Codex / ChatGPT Starter
                  </button>
                  <div className="divider" style={{ margin: "6px 0" }} />
                  <div style={{ padding: "2px 8px", fontSize: 10.5, color: "var(--text-tertiary)" }}>
                    Running Windows Apps
                  </div>
                  {runningApps.length === 0 && (
                    <div style={{ padding: "6px 8px", fontSize: 11.5, color: "var(--text-secondary)" }}>
                      No running apps detected
                    </div>
                  )}
                  {runningApps.map((app) => (
                    <button
                      key={app.bundleIdentifier}
                      type="button"
                      className="btn btn-sm"
                      style={{
                        width: "100%",
                        justifyContent: "flex-start",
                        background: "transparent",
                        borderColor: "transparent",
                      }}
                      onClick={() => {
                        onAddAppScope(app.bundleIdentifier, app.displayName);
                        onSelectAppId(app.bundleIdentifier);
                        setShowAddAppMenu(false);
                      }}
                    >
                      {app.displayName} ({app.bundleIdentifier})
                    </button>
                  ))}
                </div>
              )}
            </div>

            {selectedAppId && (
              <>
                <button
                  type="button"
                  className="btn btn-sm"
                  onClick={() => onResetAppScope(selectedAppId)}
                >
                  Reset
                </button>
                <button
                  type="button"
                  className="btn btn-sm btn-danger"
                  onClick={() => {
                    onRemoveAppScope(selectedAppId);
                    onSelectAppId(null);
                  }}
                >
                  <Trash2 size={13} />
                </button>
              </>
            )}
          </div>

          {/* Right: Modifier Layer */}
          <div style={{ display: "flex", alignItems: "center", gap: 6, position: "relative" }}>
            <span style={{ fontSize: 12, fontWeight: 500, color: "var(--text-secondary)", display: "inline-flex", alignItems: "center", gap: 4 }}>
              <Layers size={13} color="var(--text-tertiary)" /> Layer:
            </span>
            <div style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
              <button
                type="button"
                className={`btn btn-sm ${visibleModifier === null ? "layer-pill-active" : ""}`}
                disabled={liveHeldModifier !== null}
                onClick={() => onSelectModifier(null)}
              >
                Default
              </button>
              {profile.modifierLayers.map((layer) => (
                <button
                  key={layer.modifierControl}
                  type="button"
                  className={`btn btn-sm ${
                    visibleModifier === layer.modifierControl ? "layer-pill-active" : ""
                  }`}
                  disabled={liveHeldModifier !== null}
                  onClick={() => onSelectModifier(layer.modifierControl)}
                >
                  {controlDisplayName(layer.modifierControl, effectiveFamily)} held
                </button>
              ))}
            </div>

            <div style={{ position: "relative" }}>
              <button
                type="button"
                className="btn btn-sm"
                onClick={() => setShowAddModMenu((v) => !v)}
              >
                <Plus size={13} /> Add Modifier
              </button>
              {showAddModMenu && (
                <div
                  className="modal-card"
                  style={{
                    position: "absolute",
                    top: 34,
                    right: 0,
                    width: 190,
                    padding: 6,
                    zIndex: 40,
                    maxHeight: 260,
                    overflowY: "auto",
                  }}
                >
                  {availableModControls.map((ctrl) => (
                    <button
                      key={ctrl}
                      type="button"
                      className="btn btn-sm"
                      style={{
                        width: "100%",
                        justifyContent: "flex-start",
                        background: "transparent",
                        borderColor: "transparent",
                      }}
                      onClick={() => {
                        onAddModifierLayer(ctrl);
                        onSelectModifier(ctrl);
                        setShowAddModMenu(false);
                      }}
                    >
                      {controlDisplayName(ctrl, effectiveFamily)}
                    </button>
                  ))}
                  {selectedModifier && (
                    <>
                      <div className="divider" style={{ margin: "6px 0" }} />
                      <button
                        type="button"
                        className="btn btn-sm btn-danger"
                        style={{ width: "100%" }}
                        onClick={() => {
                          onRemoveModifierLayer(selectedModifier);
                          onSelectModifier(null);
                          setShowAddModMenu(false);
                        }}
                      >
                        Remove Layer
                      </button>
                    </>
                  )}
                </div>
              )}
            </div>

            {liveHeldModifier && (
              <span
                style={{
                  fontSize: 12,
                  color: "var(--accent-bright)",
                  fontWeight: 600,
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 4,
                }}
              >
                <Zap size={13} /> {controlDisplayName(liveHeldModifier, effectiveFamily)}
              </span>
            )}
          </div>
        </div>

        {/* MAIN VIEW: Controller Canvas OR Bindings List */}
        {!showsList ? (
          <div className="artwork-canvas" style={{ padding: "8px 10px 12px" }}>
            {/* Interactive SVG Map */}
            <ControllerAnnotatedMap
              family={effectiveFamily}
              shellColor={shellColor}
              showsLabels={showsLabels}
              snapshot={snapshot}
              profile={profile}
              activeModifier={visibleModifier}
              activeAppId={selectedAppId}
              hoveredControl={hoveredControl}
              onHoverControl={setHoveredControl}
              onSelectControl={onSelectControl}
            />

            {/* Hidden-labels readout or modifier chips */}
            {!showsLabels && (
              <div style={{ minHeight: 54, marginTop: 8, display: "flex", alignItems: "center", gap: 10 }}>
                {!readoutControl && visibleModifier && overrideControlsForVisibleLayer.length > 0 ? (
                  <div style={{ display: "flex", gap: 8, overflowX: "auto", paddingBottom: 4 }}>
                    {overrideControlsForVisibleLayer.map((ctrl) => {
                      const m = resolveEffectiveMapping(profile, ctrl, visibleModifier, selectedAppId);
                      return (
                        <button
                          key={ctrl}
                          type="button"
                          onClick={() => onSelectControl(ctrl)}
                          style={{
                            padding: "6px 12px",
                            borderRadius: 10,
                            background: "rgba(69,135,247,0.08)",
                            border: "1px solid rgba(69,135,247,0.22)",
                            textAlign: "left",
                            whiteSpace: "nowrap",
                          }}
                        >
                          <div style={{ fontSize: 10, fontWeight: 600, color: "var(--accent)" }}>
                            {controlDisplayName(ctrl, effectiveFamily)}
                          </div>
                          <div style={{ fontSize: 12 }}>{mappingSummary(m)}</div>
                        </button>
                      );
                    })}
                  </div>
                ) : (
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 12,
                      width: "100%",
                      padding: "8px 12px",
                      borderRadius: 10,
                      background: "rgba(255,255,255,0.03)",
                    }}
                  >
                    <div style={{ flex: 1 }}>
                      <div style={{ fontWeight: 600, fontSize: 13 }}>
                        {readoutControl
                          ? controlDisplayName(readoutControl, effectiveFamily)
                          : "Explore your controller"}
                      </div>
                      <div style={{ fontSize: 12, color: "var(--text-secondary)" }}>
                        {readoutControl
                          ? mappingSummary(
                              resolveEffectiveMapping(
                                profile,
                                readoutControl,
                                visibleModifier,
                                selectedAppId
                              )
                            )
                          : "Point to a button to see its assigned action."}
                      </div>
                    </div>
                    <span className="eyebrow">
                      {readoutControl
                        ? TRIGGER_MODE_LABELS[
                            resolveEffectiveMapping(
                              profile,
                              readoutControl,
                              visibleModifier,
                              selectedAppId
                            ).triggerMode
                          ]
                        : "CLICK TO EDIT"}
                    </span>
                  </div>
                )}
              </div>
            )}

            {/* Centered Bottom Stick Telemetry Capsules */}
            <div style={{ display: "flex", justifyContent: "center", gap: 18, marginTop: 8 }}>
              {(["left", "right"] as const).map((side) => {
                const st = side === "left" ? snapshot.leftStick : snapshot.rightStick;
                return (
                  <button
                    key={side}
                    type="button"
                    className="stick-capsule"
                    onClick={() => onSelectStickRole(side)}
                  >
                    <div
                      style={{
                        width: 34,
                        height: 34,
                        borderRadius: "50%",
                        background: "radial-gradient(circle, #1e293b 0%, #0f172a 100%)",
                        border: "1px solid rgba(96, 165, 250, 0.35)",
                        boxShadow: "inset 0 0 8px rgba(59, 130, 246, 0.25)",
                        position: "relative",
                        flexShrink: 0,
                      }}
                    >
                      <div
                        style={{
                          width: 9,
                          height: 9,
                          borderRadius: "50%",
                          background: "#60A5FA",
                          boxShadow: "0 0 10px #3B82F6",
                          position: "absolute",
                          left: 11.5 + st.x * 8,
                          top: 11.5 - st.y * 8,
                        }}
                      />
                    </div>
                    <div style={{ textAlign: "left" }}>
                      <div style={{ fontSize: 11, fontWeight: 700, letterSpacing: 0.7, color: "#E2E8F0" }}>
                        {side === "left" ? "LEFT STICK ·" : "RIGHT STICK ·"}
                      </div>
                      <div style={{ fontSize: 12.5, color: "var(--text-secondary)" }}>
                        {stickRoleLabel(side)}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>

            {/* Subtle Corner Appearance Controls (bottom-left & bottom-right of stage) */}
            <div
              style={{
                position: "absolute",
                left: 14,
                bottom: 12,
                display: "flex",
                alignItems: "center",
                gap: 6,
                opacity: 0.72,
              }}
            >
              <select
                value={layoutPreview}
                onChange={(e) => setLayoutPreview(e.target.value as "auto" | ControllerFamily)}
                style={{
                  fontSize: 10,
                  fontWeight: 600,
                  letterSpacing: 0.8,
                  padding: "3px 7px",
                  borderRadius: 7,
                }}
              >
                <option value="auto">
                  {effectiveFamily === "playStation" ? "PS (AUTO)" : "XBOX (AUTO)"}
                </option>
                <option value="xbox">XBOX</option>
                <option value="playStation">PLAYSTATION</option>
              </select>

              <div style={{ position: "relative" }}>
                <button
                  type="button"
                  className="btn btn-sm"
                  style={{ minHeight: 24, padding: "2px 8px", fontSize: 10.5 }}
                  onClick={() => setShowColorPicker((v) => !v)}
                >
                  <span
                    style={{
                      width: 10,
                      height: 10,
                      borderRadius: "50%",
                      background: activeColorSpec.palette[1],
                      border: "1px solid rgba(255,255,255,0.25)",
                    }}
                  />
                  Color
                  <ChevronDown size={10} />
                </button>
                {showColorPicker && (
                  <div
                    className="modal-card"
                    style={{
                      position: "absolute",
                      bottom: 30,
                      left: 0,
                      width: 248,
                      padding: 14,
                      zIndex: 50,
                    }}
                  >
                    <div style={{ fontWeight: 600, fontSize: 13 }}>Controller color</div>
                    <div style={{ fontSize: 11.5, color: "var(--text-secondary)", marginBottom: 10 }}>
                      {effectiveFamily === "playStation" ? "PlayStation" : "Xbox"} shell · Appearance only
                    </div>
                    <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 8 }}>
                      {SHELL_COLOR_OPTIONS.map((opt) => {
                        const spec = getShellColorSpec(opt, effectiveFamily);
                        const selected = shellColor === opt;
                        return (
                          <button
                            key={opt}
                            type="button"
                            onClick={() => handleSetShellColor(opt)}
                            style={{
                              display: "flex",
                              flexDirection: "column",
                              alignItems: "center",
                              gap: 5,
                              padding: "8px 4px",
                              borderRadius: 10,
                              background: selected ? "var(--accent-soft)" : "transparent",
                            }}
                          >
                            <div
                              style={{
                                width: 30,
                                height: 30,
                                borderRadius: "50%",
                                background: spec.palette[1],
                                border: "1px solid rgba(255,255,255,0.22)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                              }}
                            >
                              {selected && (
                                <Check
                                  size={14}
                                  color={spec.usesDarkCheckmark ? "#111" : "#fff"}
                                  strokeWidth={3}
                                />
                              )}
                            </div>
                            <span style={{ fontSize: 11 }}>{spec.label}</span>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                )}
              </div>
            </div>

            <div
              style={{
                position: "absolute",
                right: 16,
                bottom: 14,
                display: "flex",
                alignItems: "center",
                gap: 10,
                opacity: 0.72,
              }}
            >
              <label style={{ display: "inline-flex", alignItems: "center", gap: 5, fontSize: 11, cursor: "pointer", color: "var(--text-secondary)" }}>
                <input
                  type="checkbox"
                  checked={showsLabels}
                  onChange={(e) => handleToggleLabels(e.target.checked)}
                />
                Labels
              </label>
            </div>
          </div>
        ) : (
          /* BINDINGS LIST VIEW */
          <div className="artwork-canvas" style={{ overflow: "hidden" }}>
            <div style={{ padding: 14, display: "flex", alignItems: "center", gap: 10 }}>
              <Search size={15} color="var(--text-secondary)" />
              <input
                type="search"
                placeholder="Find a button or action"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                style={{ flex: 1, background: "transparent", border: "none", padding: 0 }}
              />
            </div>
            <div className="divider" />
            <div>
              {filteredBindings.map((r) => {
                const m = resolveEffectiveMapping(
                  profile,
                  r.control,
                  visibleModifier,
                  selectedAppId
                );
                return (
                  <button
                    key={r.control}
                    type="button"
                    onClick={() => onSelectControl(r.control)}
                    style={{
                      width: "100%",
                      display: "flex",
                      alignItems: "center",
                      gap: 14,
                      padding: "12px 18px",
                      borderBottom: "1px solid var(--border-subtle)",
                      textAlign: "left",
                    }}
                  >
                    <span style={{ width: 130, fontWeight: 600, fontSize: 13 }}>
                      {controlDisplayName(r.control, effectiveFamily)}
                    </span>
                    <span style={{ flex: 1 }}>
                      <KeycapBadge mapping={m} />
                    </span>
                    <ChevronRight size={14} color="var(--text-tertiary)" />
                  </button>
                );
              })}
              {filteredBindings.length === 0 && (
                <div style={{ padding: 40, textAlign: "center", color: "var(--text-secondary)" }}>
                  No controls matching "{search}"
                </div>
              )}
            </div>
          </div>
        )}

        {/* Collapsible Diagnostics */}
        <DiagnosticsPanel
          snapshot={snapshot}
          diagnostics={diagnostics}
          onTriggerSweep={onTriggerSweep}
          onTriggerLeftClick={onTriggerLeftClick}
        />
      </div>
    </div>
  );
};
