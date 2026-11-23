import React, { useState } from "react";
import {
  Gamepad2,
  Mic,
  MonitorSmartphone,
  MousePointer2,
  Play,
  Plus,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Trash2,
  Vibrate,
} from "lucide-react";
import type {
  CompanionConnectionState,
  CompanionEdge,
  CompanionMode,
  CompanionPeer,
  CursorConfiguration,
  GameGuardPreferences,
  GameGuardStatus,
  HapticsPreferences,
  HapticsStatus,
  StickAssignment,
  VoiceCommandRule,
  VoiceModeState,
} from "../types/models";
import { StudioSlider } from "./StudioSlider";

interface InspectorSidebarProps {
  cursor: CursorConfiguration;
  hapticsPrefs: HapticsPreferences;
  hapticsStatus: HapticsStatus;
  companionMode: CompanionMode;
  companionEdge: CompanionEdge;
  companionState: CompanionConnectionState;
  companionPeers: CompanionPeer[];
  voiceState: VoiceModeState;
  gameGuardPrefs: GameGuardPreferences;
  gameGuardStatus: GameGuardStatus;
  onUpdateCursor: (next: CursorConfiguration) => void;
  onUpdateHaptics: (next: HapticsPreferences) => void;
  onPreviewVibration: () => void;
  onApplySuggestedVibrations: () => void;
  onSetCompanionMode: (mode: CompanionMode) => void;
  onSetCompanionEdge: (edge: CompanionEdge) => void;
  onConnectCompanion: (peerId: string | null) => void;
  onDisconnectCompanion: () => void;
  onForceHandoff: () => void;
  onReturnLocal: () => void;
  onSetVoiceEnabled: (enabled: boolean) => void;
  onSetVoiceRules: (rules: VoiceCommandRule[]) => void;
  onTestCommand: (script: string) => void;
  onUpdateGameGuard: (prefs: GameGuardPreferences) => void;
  onOpenSteamController: () => void;
  onOpenSteamBigPicture: () => void;
}

export const InspectorSidebar: React.FC<InspectorSidebarProps> = ({
  cursor,
  hapticsPrefs,
  hapticsStatus,
  companionMode,
  companionEdge,
  companionState,
  companionPeers,
  voiceState,
  gameGuardPrefs,
  gameGuardStatus,
  onUpdateCursor,
  onUpdateHaptics,
  onPreviewVibration,
  onApplySuggestedVibrations,
  onSetCompanionMode,
  onSetCompanionEdge,
  onConnectCompanion,
  onDisconnectCompanion,
  onForceHandoff,
  onReturnLocal,
  onSetVoiceEnabled,
  onSetVoiceRules,
  onTestCommand,
  onUpdateGameGuard,
  onOpenSteamController,
  onOpenSteamBigPicture,
}) => {
  const [showFineTuning, setShowFineTuning] = useState(false);
  const [showCompanionOpts, setShowCompanionOpts] = useState(false);
  const [showVibOpts, setShowVibOpts] = useState(false);
  const [showVoiceRules, setShowVoiceRules] = useState(false);
  const [newPhrase, setNewPhrase] = useState("");
  const [newCommand, setNewCommand] = useState("");
  const [selectedPeer, setSelectedPeer] = useState<string>("");

  const patchCursor = (partial: Partial<CursorConfiguration>) =>
    onUpdateCursor({ ...cursor, ...partial });

  const companionEnabled = companionMode !== "off";
  const companionReady = companionState.state === "connected";

  const hostSummary =
    companionMode === "off"
      ? "Host: Local Windows PC"
      : companionReady
      ? `Connected: ${companionState.detail}`
      : `Mode: ${companionMode === "controller" ? "Controller PC" : "Receiver PC"}`;

  return (
    <aside className="inspector-sidebar">
      {/* CURSOR CARD */}
      <section className="sidebar-card">
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600, display: "inline-flex", alignItems: "center", gap: 7 }}>
            <MousePointer2 size={14} color="var(--accent-bright)" /> Cursor
          </h3>
          <button
            type="button"
            className="icon-ghost-btn"
            title="Stick roles & fine tuning"
            onClick={() => setShowFineTuning((v) => !v)}
          >
            <SlidersHorizontal size={13} />
          </button>
        </div>

        <StudioSlider
          label="Primary speed"
          valueLabel={`${(cursor.primarySpeed / 1600).toFixed(1)}x`}
          min={400}
          max={4200}
          step={10}
          value={cursor.primarySpeed}
          onChange={(v) => patchCursor({ primarySpeed: v })}
        />

        <StudioSlider
          label="Precision speed"
          valueLabel={`${(cursor.precisionSpeed / 1400).toFixed(1)}x`}
          min={100}
          max={1800}
          step={10}
          value={cursor.precisionSpeed}
          onChange={(v) => patchCursor({ precisionSpeed: v })}
        />

        <StudioSlider
          label="Dead zone"
          valueLabel={`${Math.round(cursor.deadZone * 100)}%`}
          min={0.02}
          max={0.35}
          step={0.01}
          value={cursor.deadZone}
          onChange={(v) => patchCursor({ deadZone: v })}
        />

        {showFineTuning && (
          <div style={{ display: "flex", flexDirection: "column", gap: 10, paddingTop: 6, borderTop: "1px solid var(--border-subtle)" }}>
            <label style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 8 }}>
              <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Primary stick</span>
              <select
                value={cursor.primaryStick}
                onChange={(e) => patchCursor({ primaryStick: e.target.value as StickAssignment })}
              >
                <option value="left">Left Stick</option>
                <option value="right">Right Stick</option>
                <option value="off">Off</option>
              </select>
            </label>

            <label style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 8 }}>
              <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Precision stick</span>
              <select
                value={cursor.precisionStick}
                onChange={(e) => patchCursor({ precisionStick: e.target.value as StickAssignment })}
              >
                <option value="right">Right Stick</option>
                <option value="left">Left Stick</option>
                <option value="off">Off</option>
              </select>
            </label>

            <div className="switch-row">
              <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Flick boost</span>
              <button
                type="button"
                className={`toggle-switch ${cursor.flickBoostEnabled ? "on" : ""}`}
                onClick={() => patchCursor({ flickBoostEnabled: !cursor.flickBoostEnabled })}
              />
            </div>

            <StudioSlider
              label="Smoothing"
              valueLabel={cursor.smoothing.toFixed(2)}
              min={0}
              max={1}
              step={0.02}
              value={cursor.smoothing}
              onChange={(v) => patchCursor({ smoothing: v })}
            />

            <StudioSlider
              label="Response curve"
              valueLabel={cursor.responseCurve.toFixed(2)}
              min={1.0}
              max={2.6}
              step={0.05}
              value={cursor.responseCurve}
              onChange={(v) => patchCursor({ responseCurve: v })}
            />

            {(
              [
                ["accelerationEnabled", "Acceleration"],
                ["invertPrimaryX", "Invert primary X"],
                ["invertPrimaryY", "Invert primary Y"],
                ["invertPrecisionX", "Invert precision X"],
                ["invertPrecisionY", "Invert precision Y"],
              ] as const
            ).map(([key, label]) => (
              <div key={key} className="switch-row" style={{ minHeight: 24 }}>
                <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>{label}</span>
                <button
                  type="button"
                  className={`toggle-switch ${cursor[key] ? "on" : ""}`}
                  onClick={() => patchCursor({ [key]: !cursor[key] })}
                />
              </div>
            ))}

            <StudioSlider
              label="Horizontal multiplier"
              valueLabel={`${cursor.horizontalSpeedMultiplier.toFixed(2)}x`}
              min={0.2}
              max={2.0}
              step={0.05}
              value={cursor.horizontalSpeedMultiplier}
              onChange={(v) => patchCursor({ horizontalSpeedMultiplier: v })}
            />

            <StudioSlider
              label="Vertical multiplier"
              valueLabel={`${cursor.verticalSpeedMultiplier.toFixed(2)}x`}
              min={0.2}
              max={2.0}
              step={0.05}
              value={cursor.verticalSpeedMultiplier}
              onChange={(v) => patchCursor({ verticalSpeedMultiplier: v })}
            />
          </div>
        )}
      </section>

      {/* MULTI-PC CONTROL CARD */}
      <section className="sidebar-card">
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600, display: "inline-flex", alignItems: "center", gap: 7 }}>
            <MonitorSmartphone size={14} color="var(--accent-bright)" /> Multi-PC Control
          </h3>
          <button
            type="button"
            className="icon-ghost-btn"
            title="Connection options"
            onClick={() => setShowCompanionOpts((v) => !v)}
          >
            <SlidersHorizontal size={13} />
          </button>
        </div>

        <div className="switch-row">
          <span style={{ fontSize: 13, color: "#cbd5e1" }}>Enable Multi-PC</span>
          <button
            type="button"
            className={`toggle-switch ${companionEnabled ? "on" : ""}`}
            onClick={() => {
              onSetCompanionMode(companionEnabled ? "off" : "controller");
              if (!companionEnabled) setShowCompanionOpts(true);
            }}
            aria-label="Enable Multi-PC"
          />
        </div>

        <div className="switch-row">
          <span style={{ fontSize: 13, color: "#cbd5e1" }}>Edge screen switch</span>
          <button
            type="button"
            className={`toggle-switch ${companionMode === "controller" ? "on" : ""}`}
            onClick={() => {
              onSetCompanionMode(companionMode === "controller" ? "off" : "controller");
              setShowCompanionOpts(true);
            }}
            aria-label="Edge screen switch"
          />
        </div>

        <div style={{ fontSize: 12, color: "var(--text-secondary)" }}>{hostSummary}</div>

        {showCompanionOpts && (
          <div style={{ display: "flex", flexDirection: "column", gap: 8, paddingTop: 6, borderTop: "1px solid var(--border-subtle)" }}>
            <label style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 8 }}>
              <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Mode</span>
              <select
                value={companionMode}
                onChange={(e) => onSetCompanionMode(e.target.value as CompanionMode)}
              >
                <option value="off">Local Windows Input</option>
                <option value="controller">Controller PC</option>
                <option value="receiver">Receiver PC</option>
              </select>
            </label>

            {companionMode === "controller" && (
              <>
                <label style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 8 }}>
                  <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Handoff edge</span>
                  <select
                    value={companionEdge}
                    onChange={(e) => onSetCompanionEdge(e.target.value as CompanionEdge)}
                  >
                    <option value="left">Left</option>
                    <option value="right">Right</option>
                    <option value="top">Top</option>
                    <option value="bottom">Bottom</option>
                  </select>
                </label>

                <label style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 8 }}>
                  <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Receiver</span>
                  <select
                    value={selectedPeer}
                    onChange={(e) => setSelectedPeer(e.target.value)}
                  >
                    <option value="">Auto / first available</option>
                    {companionPeers.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                  </select>
                </label>

                <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
                  <button
                    type="button"
                    className="btn btn-sm"
                    onClick={() => onConnectCompanion(selectedPeer || null)}
                  >
                    Connect
                  </button>
                  <button type="button" className="btn btn-sm" onClick={onDisconnectCompanion}>
                    Disconnect
                  </button>
                  <button
                    type="button"
                    className="btn btn-sm"
                    disabled={!companionReady}
                    onClick={onForceHandoff}
                  >
                    Test Handoff
                  </button>
                  <button
                    type="button"
                    className="btn btn-sm"
                    disabled={!companionReady}
                    onClick={onReturnLocal}
                  >
                    Return Local
                  </button>
                </div>
              </>
            )}

            {companionMode === "receiver" && (
              <div style={{ fontSize: 11.5, color: "var(--text-secondary)" }}>
                Advertising over LAN UDP/TCP for another Vibe Controller PC.
              </div>
            )}
          </div>
        )}
      </section>

      {/* VIBRATION CARD */}
      <section className="sidebar-card">
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600, display: "inline-flex", alignItems: "center", gap: 7 }}>
            <Vibrate size={14} color="var(--accent-bright)" /> Vibration
          </h3>
          <button
            type="button"
            className="icon-ghost-btn"
            title="Vibration options"
            onClick={() => setShowVibOpts((v) => !v)}
          >
            <SlidersHorizontal size={13} />
          </button>
        </div>

        <StudioSlider
          label="Strength"
          valueLabel={`${Math.round(hapticsPrefs.strength * 100)}%`}
          min={0.1}
          max={1.0}
          step={0.05}
          value={hapticsPrefs.strength}
          onChange={(v) => onUpdateHaptics({ ...hapticsPrefs, strength: v })}
        />

        <button
          type="button"
          className="btn"
          style={{ width: "100%", marginTop: 1 }}
          disabled={!hapticsPrefs.enabled || !hapticsStatus.canPlay}
          onClick={onPreviewVibration}
        >
          <Play size={12} /> Test vibration
        </button>

        {showVibOpts && (
          <div style={{ display: "flex", flexDirection: "column", gap: 9, paddingTop: 6, borderTop: "1px solid var(--border-subtle)" }}>
            <div style={{ fontSize: 11, color: "var(--text-secondary)" }}>
              {hapticsStatus.message}
            </div>
            <div className="switch-row">
              <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Haptic feedback</span>
              <button
                type="button"
                className={`toggle-switch ${hapticsPrefs.enabled ? "on" : ""}`}
                onClick={() =>
                  onUpdateHaptics({ ...hapticsPrefs, enabled: !hapticsPrefs.enabled })
                }
              />
            </div>
            <div className="switch-row">
              <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Pulse on connection</span>
              <button
                type="button"
                className={`toggle-switch ${hapticsPrefs.connectionFeedback ? "on" : ""}`}
                onClick={() =>
                  onUpdateHaptics({
                    ...hapticsPrefs,
                    connectionFeedback: !hapticsPrefs.connectionFeedback,
                  })
                }
              />
            </div>
            <button type="button" className="btn btn-sm" onClick={onApplySuggestedVibrations}>
              <Sparkles size={12} /> Use suggested patterns
            </button>
          </div>
        )}
      </section>

      {/* VOICE MODE CARD */}
      <section className="sidebar-card">
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600, display: "inline-flex", alignItems: "center", gap: 7 }}>
            <Mic size={14} color={voiceState.listening ? "var(--green)" : "var(--accent-bright)"} /> Voice Mode
          </h3>
          <button
            type="button"
            className="icon-ghost-btn"
            title="Edit voice command scripts"
            onClick={() => setShowVoiceRules((v) => !v)}
          >
            <SlidersHorizontal size={13} />
          </button>
        </div>

        <div className="switch-row">
          <span style={{ fontSize: 13, color: "#cbd5e1" }}>Listen for scripts</span>
          <button
            type="button"
            className={`toggle-switch ${voiceState.enabled ? "on" : ""}`}
            onClick={() => onSetVoiceEnabled(!voiceState.enabled)}
            aria-label="Enable Voice Mode"
          />
        </div>

        <div style={{ fontSize: 11.5, color: voiceState.listening ? "var(--green)" : "var(--text-secondary)" }}>
          {voiceState.lastHeard
            ? `Heard "${voiceState.lastHeard}"`
            : voiceState.statusMessage}
        </div>

        {showVoiceRules && (
          <div style={{ display: "flex", flexDirection: "column", gap: 7, paddingTop: 6, borderTop: "1px solid var(--border-subtle)" }}>
            {voiceState.rules.map((rule) => (
              <div
                key={rule.id}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                  padding: "5px 7px",
                  borderRadius: 8,
                  background: "rgba(255,255,255,0.03)",
                  border: "1px solid var(--border-subtle)",
                }}
              >
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    "{rule.phrase}"
                  </div>
                  <div style={{ fontSize: 10.5, fontFamily: "var(--font-mono)", color: "var(--text-tertiary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {rule.command}
                  </div>
                </div>
                <button
                  type="button"
                  className="icon-ghost-btn"
                  title="Test command"
                  onClick={() => onTestCommand(rule.command)}
                >
                  <Play size={11} />
                </button>
                <button
                  type="button"
                  className="icon-ghost-btn"
                  title="Remove rule"
                  onClick={() =>
                    onSetVoiceRules(voiceState.rules.filter((r) => r.id !== rule.id))
                  }
                >
                  <Trash2 size={11} />
                </button>
              </div>
            ))}

            <input
              type="text"
              placeholder='Phrase (e.g. "run build")'
              value={newPhrase}
              onChange={(e) => setNewPhrase(e.target.value)}
              style={{
                fontSize: 11.5,
                padding: "6px 8px",
                borderRadius: 7,
                background: "rgba(0,0,0,0.28)",
                border: "1px solid var(--border-subtle)",
                color: "var(--text-primary)",
                outline: "none",
              }}
            />
            <input
              type="text"
              placeholder="Command / script (e.g. wt.exe)"
              value={newCommand}
              onChange={(e) => setNewCommand(e.target.value)}
              style={{
                fontSize: 11.5,
                fontFamily: "var(--font-mono)",
                padding: "6px 8px",
                borderRadius: 7,
                background: "rgba(0,0,0,0.28)",
                border: "1px solid var(--border-subtle)",
                color: "var(--text-primary)",
                outline: "none",
              }}
            />
            <button
              type="button"
              className="btn btn-sm"
              disabled={!newPhrase.trim() || !newCommand.trim()}
              onClick={() => {
                onSetVoiceRules([
                  ...voiceState.rules,
                  {
                    id: `rule-${Date.now()}`,
                    phrase: newPhrase.trim(),
                    command: newCommand.trim(),
                    enabled: true,
                  },
                ]);
                setNewPhrase("");
                setNewCommand("");
              }}
            >
              <Plus size={12} /> Add voice script
            </button>
          </div>
        )}
      </section>

      {/* ANTI-CHEAT GUARD & STEAM CARD */}
      <section className="sidebar-card">
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600, display: "inline-flex", alignItems: "center", gap: 7 }}>
            <ShieldCheck size={14} color={gameGuardStatus.isSuspended ? "var(--orange)" : "var(--green)"} /> Game Guard & Steam
          </h3>
        </div>

        <div className="switch-row">
          <span style={{ fontSize: 12.5, color: "#cbd5e1" }}>Anti-cheat auto-pause</span>
          <button
            type="button"
            className={`toggle-switch ${gameGuardPrefs.antiCheatAutoPause ? "on" : ""}`}
            onClick={() =>
              onUpdateGameGuard({
                ...gameGuardPrefs,
                antiCheatAutoPause: !gameGuardPrefs.antiCheatAutoPause,
              })
            }
            aria-label="Anti-cheat auto-pause"
          />
        </div>

        <div className="switch-row">
          <span style={{ fontSize: 12.5, color: "#cbd5e1" }}>Steam game auto-pause</span>
          <button
            type="button"
            className={`toggle-switch ${gameGuardPrefs.steamGameAutoPause ? "on" : ""}`}
            onClick={() =>
              onUpdateGameGuard({
                ...gameGuardPrefs,
                steamGameAutoPause: !gameGuardPrefs.steamGameAutoPause,
              })
            }
            aria-label="Steam game auto-pause"
          />
        </div>

        <div
          style={{
            fontSize: 11.5,
            color: gameGuardStatus.isSuspended ? "var(--orange)" : "var(--text-secondary)",
          }}
        >
          {gameGuardStatus.isSuspended
            ? gameGuardStatus.reason
            : gameGuardStatus.steamRunning
            ? "Steam detected · Compliant"
            : "Compliant · No anti-cheat active"}
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6, marginTop: 2 }}>
          <button type="button" className="btn btn-sm" onClick={onOpenSteamController}>
            <Gamepad2 size={11} /> Steam Input
          </button>
          <button type="button" className="btn btn-sm" onClick={onOpenSteamBigPicture}>
            Big Picture
          </button>
        </div>
      </section>
    </aside>
  );
};
