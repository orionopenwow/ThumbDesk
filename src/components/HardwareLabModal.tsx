import React, { useEffect, useRef, useState } from "react";
import {
  Activity,
  Compass,
  Cpu,
  Disc,
  RotateCcw,
  Sparkles,
  Terminal,
  X,
} from "lucide-react";
import type {
  AdaptiveTriggerMode,
  ControllerSnapshot,
  GyroActivationMode,
  GyroConfiguration,
  HardwareFxPreferences,
  LightbarMode,
  RadialMenuSlice,
} from "../types/models";
import { StudioSlider } from "./StudioSlider";

interface HardwareLabModalProps {
  snapshot: ControllerSnapshot;
  gyro: GyroConfiguration;
  hardwareFx: HardwareFxPreferences;
  radialMenu: RadialMenuSlice[];
  automationApiUrl: string;
  onUpdateGyro: (next: GyroConfiguration) => void;
  onUpdateHardwareFx: (next: HardwareFxPreferences) => void;
  onOpenRadialPreview: () => void;
  onClose: () => void;
}

type LabTab = "calibration" | "gyro" | "fx" | "api";

const TRIGGER_MODES: { value: AdaptiveTriggerMode; label: string; desc: string }[] = [
  { value: "off", label: "Off", desc: "Linear travel" },
  { value: "softClick", label: "Soft Click", desc: "Tactile 45% detent" },
  { value: "mediumDetent", label: "Medium", desc: "Smooth resistance" },
  { value: "rigidWall", label: "Rigid Wall", desc: "Firm 55% stop" },
  { value: "dualStage", label: "Dual-Stage", desc: "Hover → Click wall" },
];

export const HardwareLabModal: React.FC<HardwareLabModalProps> = ({
  snapshot,
  gyro,
  hardwareFx,
  radialMenu,
  automationApiUrl,
  onUpdateGyro,
  onUpdateHardwareFx,
  onOpenRadialPreview,
  onClose,
}) => {
  const [tab, setTab] = useState<LabTab>("calibration");
  const leftCanvasRef = useRef<HTMLCanvasElement>(null);
  const rightCanvasRef = useRef<HTMLCanvasElement>(null);
  const leftTraceRef = useRef<[number, number][]>([]);
  const rightTraceRef = useRef<[number, number][]>([]);
  const [traceVersion, setTraceVersion] = useState(0);

  // Record stick trace points when live stick moves
  useEffect(() => {
    const lx = snapshot.leftStick.x;
    const ly = snapshot.leftStick.y;
    const rx = snapshot.rightStick.x;
    const ry = snapshot.rightStick.y;
    if (Math.hypot(lx, ly) > 0.08) {
      leftTraceRef.current.push([lx, ly]);
      if (leftTraceRef.current.length > 360) leftTraceRef.current.shift();
    }
    if (Math.hypot(rx, ry) > 0.08) {
      rightTraceRef.current.push([rx, ry]);
      if (rightTraceRef.current.length > 360) rightTraceRef.current.shift();
    }
  }, [snapshot.leftStick.x, snapshot.leftStick.y, snapshot.rightStick.x, snapshot.rightStick.y]);

  useEffect(() => {
    if (tab !== "calibration") return;
    const drawStick = (
      canvas: HTMLCanvasElement | null,
      sx: number,
      sy: number,
      trace: [number, number][]
    ) => {
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      const W = canvas.width;
      const H = canvas.height;
      const cx = W / 2;
      const cy = H / 2;
      const r = W * 0.41;

      ctx.clearRect(0, 0, W, H);
      // Outer reference circle
      ctx.strokeStyle = "rgba(255,255,255,0.12)";
      ctx.lineWidth = 1.2;
      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      ctx.stroke();

      // Deadzone inner ring (12%)
      ctx.strokeStyle = "rgba(76, 141, 255, 0.25)";
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.arc(cx, cy, r * 0.12, 0, Math.PI * 2);
      ctx.stroke();
      ctx.setLineDash([]);

      // Crosshairs
      ctx.strokeStyle = "rgba(255,255,255,0.06)";
      ctx.beginPath();
      ctx.moveTo(cx - r, cy);
      ctx.lineTo(cx + r, cy);
      ctx.moveTo(cx, cy - r);
      ctx.lineTo(cx, cy + r);
      ctx.stroke();

      // Trace points
      if (trace.length > 0) {
        ctx.fillStyle = "rgba(56, 189, 248, 0.55)";
        for (const [tx, ty] of trace) {
          ctx.fillRect(cx + tx * r - 1.5, cy - ty * r - 1.5, 3, 3);
        }
      }

      // Live dot
      const px = cx + sx * r;
      const py = cy - sy * r;
      ctx.strokeStyle = "rgba(125, 211, 252, 0.5)";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.lineTo(px, py);
      ctx.stroke();

      ctx.fillStyle = "#60a5fa";
      ctx.beginPath();
      ctx.arc(px, py, 5.5, 0, Math.PI * 2);
      ctx.fill();
    };

    drawStick(leftCanvasRef.current, snapshot.leftStick.x, snapshot.leftStick.y, leftTraceRef.current);
    drawStick(rightCanvasRef.current, snapshot.rightStick.x, snapshot.rightStick.y, rightTraceRef.current);
  }, [tab, snapshot.leftStick.x, snapshot.leftStick.y, snapshot.rightStick.x, snapshot.rightStick.y, traceVersion]);

  const leftDriftPct = (Math.hypot(snapshot.leftStick.x, snapshot.leftStick.y) * 100).toFixed(1);
  const rightDriftPct = (Math.hypot(snapshot.rightStick.x, snapshot.rightStick.y) * 100).toFixed(1);

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal-card"
        style={{ width: 700, maxHeight: "88vh" }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div style={{ padding: "18px 22px 14px", borderBottom: "1px solid var(--border-subtle)" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <div
              style={{
                width: 42,
                height: 42,
                borderRadius: 11,
                background: "var(--accent-soft)",
                border: "1px solid rgba(69,135,247,0.3)",
                color: "var(--accent-bright)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Cpu size={20} />
            </div>
            <div style={{ flex: 1 }}>
              <div className="eyebrow">HARDWARE &amp; SENSOR LAB</div>
              <h2 style={{ margin: "2px 0 0", fontSize: 18, fontWeight: 600 }}>
                Calibration, Gyro, Adaptive Triggers &amp; API
              </h2>
            </div>
            <button type="button" className="btn btn-sm" onClick={onClose} aria-label="Close">
              <X size={15} />
            </button>
          </div>

          {/* Tabs */}
          <div className="segmented" style={{ marginTop: 14 }}>
            <button
              type="button"
              className={`segmented-btn ${tab === "calibration" ? "active" : ""}`}
              onClick={() => setTab("calibration")}
            >
              <Activity size={12} /> Drift &amp; Polling
            </button>
            <button
              type="button"
              className={`segmented-btn ${tab === "gyro" ? "active" : ""}`}
              onClick={() => setTab("gyro")}
            >
              <Compass size={12} /> 6-Axis Gyro
            </button>
            <button
              type="button"
              className={`segmented-btn ${tab === "fx" ? "active" : ""}`}
              onClick={() => setTab("fx")}
            >
              <Sparkles size={12} /> Triggers &amp; RGB
            </button>
            <button
              type="button"
              className={`segmented-btn ${tab === "api" ? "active" : ""}`}
              onClick={() => setTab("api")}
            >
              <Terminal size={12} /> Radial &amp; API
            </button>
          </div>
        </div>

        {/* Body */}
        <div style={{ padding: 20, overflowY: "auto", display: "flex", flexDirection: "column", gap: 14 }}>
          {tab === "calibration" && (
            <>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 14 }}>
                <div className="product-panel" style={{ padding: 14, textAlign: "center" }}>
                  <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11.5, color: "var(--text-secondary)", marginBottom: 8 }}>
                    <span>Left Stick Circularity</span>
                    <span style={{ fontFamily: "var(--font-mono)", color: "var(--accent-bright)" }}>{leftDriftPct}%</span>
                  </div>
                  <canvas ref={leftCanvasRef} width={200} height={200} style={{ width: 180, height: 180, margin: "0 auto", display: "block" }} />
                </div>
                <div className="product-panel" style={{ padding: 14, textAlign: "center" }}>
                  <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11.5, color: "var(--text-secondary)", marginBottom: 8 }}>
                    <span>Right Stick Circularity</span>
                    <span style={{ fontFamily: "var(--font-mono)", color: "var(--accent-bright)" }}>{rightDriftPct}%</span>
                  </div>
                  <canvas ref={rightCanvasRef} width={200} height={200} style={{ width: 180, height: 180, margin: "0 auto", display: "block" }} />
                </div>
              </div>

              <div className="product-panel" style={{ padding: 12, display: "flex", alignItems: "center", justifyContent: "space-between", fontSize: 12 }}>
                <div style={{ display: "flex", gap: 18, fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>
                  <span>POLLING: <b style={{ color: "var(--green)" }}>{snapshot.pollingRateHz ?? 120} Hz</b></span>
                  <span>EVENTS: <b style={{ color: "var(--text-primary)" }}>{snapshot.eventCount}</b></span>
                  <span>IMU: <b style={{ color: snapshot.hasGyro ? "var(--green)" : "var(--text-tertiary)" }}>{snapshot.hasGyro ? "6-AXIS READY" : "N/A"}</b></span>
                </div>
                <button
                  type="button"
                  className="btn btn-sm"
                  onClick={() => {
                    leftTraceRef.current = [];
                    rightTraceRef.current = [];
                    setTraceVersion((v) => v + 1);
                  }}
                >
                  <RotateCcw size={11} /> Reset Trace
                </button>
              </div>
            </>
          )}

          {tab === "gyro" && (
            <div className="product-panel" style={{ padding: 16, display: "flex", flexDirection: "column", gap: 12 }}>
              <div className="switch-row">
                <span style={{ fontSize: 13, fontWeight: 500 }}>Enable 6-Axis Gyro Mouse (DualSense / DS4)</span>
                <button
                  type="button"
                  className={`toggle-switch ${gyro.enabled ? "on" : ""}`}
                  onClick={() => onUpdateGyro({ ...gyro, enabled: !gyro.enabled })}
                />
              </div>

              <label style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 10 }}>
                <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Activation mode</span>
                <select
                  value={gyro.activationMode}
                  onChange={(e) =>
                    onUpdateGyro({ ...gyro, activationMode: e.target.value as GyroActivationMode })
                  }
                >
                  <option value="touchpadTouch">Touchpad Touch (Capacitive)</option>
                  <option value="always">Always On</option>
                  <option value="holdLeftTrigger">While Holding L2 / LT</option>
                  <option value="holdRightShoulder">While Holding R1 / RB</option>
                </select>
              </label>

              <StudioSlider
                label="Gyro sensitivity"
                valueLabel={`${gyro.sensitivity.toFixed(2)}x`}
                min={0.2}
                max={4.0}
                step={0.05}
                value={gyro.sensitivity}
                onChange={(v) => onUpdateGyro({ ...gyro, sensitivity: v })}
              />
              <StudioSlider
                label="Gyro deadzone"
                valueLabel={`${Math.round(gyro.deadzone * 100)}%`}
                min={0.0}
                max={0.25}
                step={0.01}
                value={gyro.deadzone}
                onChange={(v) => onUpdateGyro({ ...gyro, deadzone: v })}
              />
              <StudioSlider
                label="Gyro smoothing"
                valueLabel={gyro.smoothing.toFixed(2)}
                min={0.0}
                max={0.9}
                step={0.05}
                value={gyro.smoothing}
                onChange={(v) => onUpdateGyro({ ...gyro, smoothing: v })}
              />
            </div>
          )}

          {tab === "fx" && (
            <>
              <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 10 }}>
                <span style={{ fontSize: 12, fontWeight: 600, color: "var(--text-secondary)" }}>
                  DualSense Adaptive Triggers (L2 / R2 Resistance)
                </span>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                  <label style={{ display: "flex", flexDirection: "column", gap: 5 }}>
                    <span style={{ fontSize: 11.5, color: "var(--text-tertiary)" }}>Left Trigger (L2)</span>
                    <select
                      value={hardwareFx.leftTriggerMode}
                      onChange={(e) =>
                        onUpdateHardwareFx({
                          ...hardwareFx,
                          leftTriggerMode: e.target.value as AdaptiveTriggerMode,
                        })
                      }
                    >
                      {TRIGGER_MODES.map((m) => (
                        <option key={m.value} value={m.value}>
                          {m.label} — {m.desc}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label style={{ display: "flex", flexDirection: "column", gap: 5 }}>
                    <span style={{ fontSize: 11.5, color: "var(--text-tertiary)" }}>Right Trigger (R2)</span>
                    <select
                      value={hardwareFx.rightTriggerMode}
                      onChange={(e) =>
                        onUpdateHardwareFx({
                          ...hardwareFx,
                          rightTriggerMode: e.target.value as AdaptiveTriggerMode,
                        })
                      }
                    >
                      {TRIGGER_MODES.map((m) => (
                        <option key={m.value} value={m.value}>
                          {m.label} — {m.desc}
                        </option>
                      ))}
                    </select>
                  </label>
                </div>
              </div>

              <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 10 }}>
                <span style={{ fontSize: 12, fontWeight: 600, color: "var(--text-secondary)" }}>
                  Dynamic Lightbar RGB Telemetry
                </span>
                <div className="segmented" style={{ alignSelf: "flex-start" }}>
                  {(["autoTelemetry", "customColor", "off"] as LightbarMode[]).map((mode) => (
                    <button
                      key={mode}
                      type="button"
                      className={`segmented-btn ${hardwareFx.lightbarMode === mode ? "active" : ""}`}
                      onClick={() => onUpdateHardwareFx({ ...hardwareFx, lightbarMode: mode })}
                    >
                      {mode === "autoTelemetry" ? "Auto Telemetry" : mode === "customColor" ? "Custom RGB" : "Off"}
                    </button>
                  ))}
                </div>
                <div style={{ fontSize: 11.5, color: "var(--text-tertiary)" }}>
                  Auto Telemetry colors the DualSense/DS4 lightbar live: Cyan (Active), Amber (Anti-Cheat Guard), Emerald (Voice Listening), Purple (Modifier Held), Red (Low Battery).
                </div>
              </div>
            </>
          )}

          {tab === "api" && (
            <>
              <div className="product-panel" style={{ padding: 14, display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12 }}>
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600 }}>8-Slice Radial Pie Menu</div>
                  <div style={{ fontSize: 11.5, color: "var(--text-secondary)" }}>
                    {radialMenu.length} configured slices (Terminal, Window Snap, Media, Voice). Map any button to "Open Radial Pie Menu".
                  </div>
                </div>
                <button type="button" className="btn btn-primary btn-sm" onClick={onOpenRadialPreview}>
                  <Disc size={13} /> Open Radial Wheel
                </button>
              </div>

              <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ fontSize: 12, fontWeight: 600, color: "var(--text-secondary)" }}>
                  Localhost Automation API ({automationApiUrl})
                </div>
                <pre
                  style={{
                    margin: 0,
                    padding: 12,
                    borderRadius: 8,
                    background: "rgba(0,0,0,0.38)",
                    border: "1px solid var(--border-subtle)",
                    fontFamily: "var(--font-mono)",
                    fontSize: 11,
                    color: "#93c5fd",
                    overflowX: "auto",
                  }}
                >{`# Query live controller & guard status
curl ${automationApiUrl}/api/status

# Buzz controller when a build or AI task finishes
curl -X POST ${automationApiUrl}/api/haptic -d '{"pattern":"doubleTap"}'

# Set DualSense/DS4 lightbar color from OBS / Stream Deck
curl -X POST ${automationApiUrl}/api/lightbar -d '{"rgb":[56,189,248]}'`}</pre>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
};
