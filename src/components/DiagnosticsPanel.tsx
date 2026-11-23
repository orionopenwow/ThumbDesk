import React, { useState } from "react";
import { Activity, ChevronDown, ChevronRight, Radio } from "lucide-react";
import type { ControllerSnapshot, RuntimeDiagnostics } from "../types/models";
import { controlDisplayName, getControlAmount } from "../utils/controllerLayout";

interface DiagnosticsPanelProps {
  snapshot: ControllerSnapshot;
  diagnostics: RuntimeDiagnostics;
  onTriggerSweep: () => void;
  onTriggerLeftClick: () => void;
}

const AxisMeter: React.FC<{ label: string; value: number }> = ({ label, value }) => {
  const pct = ((Math.max(-1, Math.min(1, value)) + 1) / 2) * 100;
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 11.5 }}>
      <span style={{ width: 14, color: "var(--text-secondary)" }}>{label}</span>
      <div
        style={{
          width: 96,
          height: 10,
          borderRadius: 999,
          background: "rgba(255,255,255,0.08)",
          position: "relative",
        }}
      >
        <div
          style={{
            position: "absolute",
            left: "50%",
            top: 1,
            bottom: 1,
            width: 1,
            background: "rgba(255,255,255,0.22)",
          }}
        />
        <div
          style={{
            position: "absolute",
            top: 0,
            left: `calc(${pct}% - 5px)`,
            width: 10,
            height: 10,
            borderRadius: "50%",
            background: "var(--accent)",
          }}
        />
      </div>
      <span style={{ fontFamily: "ui-monospace, Consolas, monospace", width: 42, textAlign: "right" }}>
        {value.toFixed(2)}
      </span>
    </div>
  );
};

export const DiagnosticsPanel: React.FC<DiagnosticsPanelProps> = ({
  snapshot,
  diagnostics,
  onTriggerSweep,
  onTriggerLeftClick,
}) => {
  const [expanded, setExpanded] = useState(false);

  const lt = getControlAmount(snapshot, "leftTrigger");
  const rt = getControlAmount(snapshot, "rightTrigger");

  return (
    <div style={{ borderTop: "1px solid var(--border-subtle)", paddingTop: 14 }}>
      <button
        type="button"
        onClick={() => setExpanded((v) => !v)}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          width: "100%",
          padding: "6px 0",
          textAlign: "left",
        }}
      >
        <Activity size={16} color="var(--accent)" />
        <span style={{ fontSize: 13.5, fontWeight: 500, flex: 1 }}>Input diagnostics</span>
        <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>
          {snapshot.isConnected
            ? `${snapshot.transport ?? "Connected"} · ${snapshot.eventCount} events`
            : "Listening for controller"}
        </span>
        {expanded ? <ChevronDown size={15} /> : <ChevronRight size={15} />}
      </button>

      {expanded && (
        <div style={{ marginTop: 14, display: "flex", flexDirection: "column", gap: 14 }}>
          <div style={{ display: "grid", gridTemplateColumns: "1.4fr 1fr 1fr", gap: 12 }}>
            <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 10 }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <span style={{ display: "inline-flex", alignItems: "center", gap: 6, fontWeight: 600 }}>
                  <Radio size={14} color={snapshot.isConnected ? "var(--green)" : "var(--text-secondary)"} />
                  Controller
                </span>
                <span style={{ fontSize: 11.5, color: snapshot.isConnected ? "var(--green)" : "var(--text-secondary)" }}>
                  {snapshot.eventCount} events
                </span>
              </div>
              <div style={{ display: "flex", gap: 14, flexWrap: "wrap" }}>
                <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                  <span style={{ fontSize: 11, color: "var(--text-secondary)" }}>Left stick</span>
                  <AxisMeter label="X" value={snapshot.leftStick.x} />
                  <AxisMeter label="Y" value={snapshot.leftStick.y} />
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                  <span style={{ fontSize: 11, color: "var(--text-secondary)" }}>Right stick</span>
                  <AxisMeter label="X" value={snapshot.rightStick.x} />
                  <AxisMeter label="Y" value={snapshot.rightStick.y} />
                </div>
              </div>
              <div style={{ display: "flex", gap: 14, fontSize: 11.5 }}>
                <span>LT: {(lt * 100).toFixed(0)}%</span>
                <span>RT: {(rt * 100).toFixed(0)}%</span>
              </div>
              <div
                style={{
                  fontFamily: "ui-monospace, Consolas, monospace",
                  fontSize: 11.5,
                  color: "var(--text-secondary)",
                }}
              >
                Pressed:{" "}
                {snapshot.pressedControls.length > 0
                  ? snapshot.pressedControls
                      .map((c) => controlDisplayName(c, snapshot.family))
                      .join(", ")
                  : "None"}
              </div>
            </div>

            <div className="product-panel" style={{ padding: 14 }}>
              <div style={{ fontWeight: 600, marginBottom: 6 }}>Cursor</div>
              <div
                style={{
                  fontSize: 13,
                  fontWeight: 600,
                  color: diagnostics.cursorState === "moving" ? "var(--accent)" : "var(--text-secondary)",
                  textTransform: "capitalize",
                }}
              >
                {diagnostics.cursorState}
              </div>
              <div style={{ fontSize: 12, color: "var(--text-secondary)", marginTop: 4 }}>
                {diagnostics.cursorMessage}
              </div>
              {diagnostics.activeAppName && (
                <div style={{ fontSize: 11.5, color: "var(--text-tertiary)", marginTop: 10 }}>
                  Foreground app: {diagnostics.activeAppName} ({diagnostics.activeAppId})
                </div>
              )}
            </div>

            <div className="product-panel" style={{ padding: 14 }}>
              <div style={{ fontWeight: 600, marginBottom: 6 }}>Last action</div>
              <div style={{ fontSize: 13, fontWeight: 500 }}>{diagnostics.lastActionStatus}</div>
              <div style={{ fontSize: 12, color: "var(--text-secondary)", marginTop: 4 }}>
                Live feedback from shortcuts, clicks, and cross-screen actions.
              </div>
            </div>
          </div>

          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <span style={{ fontSize: 12, color: "var(--text-secondary)", fontWeight: 500 }}>
              Hardware checks
            </span>
            <button type="button" className="btn btn-primary btn-sm" onClick={onTriggerSweep}>
              Cross-Edge Sweep
            </button>
            <button type="button" className="btn btn-sm" onClick={onTriggerLeftClick}>
              Left Click
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
