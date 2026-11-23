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
  initialTab?: LabTab;
  onUpdateGyro: (next: GyroConfiguration) => void;
  onUpdateHardwareFx: (next: HardwareFxPreferences) => void;
  onOpenRadialPreview: () => void;
  onClose: () => void;
}

export type LabTab = "calibration" | "gyro" | "fx" | "api";

const TRIGGER_MODES: { value: AdaptiveTriggerMode; label: string; desc: string; curve: string }[] = [
  { value: "off", label: "Off", desc: "Linear travel", curve: "M 4 34 L 76 6" },
  { value: "softClick", label: "Soft Click", desc: "Tactile 45% detent", curve: "M 4 34 L 34 26 L 40 12 L 48 22 L 76 14" },
  { value: "mediumDetent", label: "Medium", desc: "Smooth resistance", curve: "M 4 34 L 24 30 Q 48 16 76 10" },
  { value: "rigidWall", label: "Rigid Wall", desc: "Firm 55% stop", curve: "M 4 34 L 44 32 L 46 6 L 76 6" },
  { value: "dualStage", label: "Dual-Stage", desc: "Hover → Click wall", curve: "M 4 34 L 30 24 L 44 24 L 48 10 L 76 8" },
];

const RGB_PRESETS: { name: string; rgb: [number, number, number] }[] = [
  { name: "ThumbDesk Cyan", rgb: [56, 189, 248] },
  { name: "Studio Blue", rgb: [76, 141, 255] },
  { name: "Voice Emerald", rgb: [52, 211, 153] },
  { name: "Modifier Violet", rgb: [168, 85, 247] },
  { name: "Guard Amber", rgb: [245, 158, 11] },
  { name: "Signal Red", rgb: [239, 68, 68] },
];

export const HardwareLabModal: React.FC<HardwareLabModalProps> = ({
  snapshot,
  gyro,
  hardwareFx,
  radialMenu,
  automationApiUrl,
  initialTab = "calibration",
  onUpdateGyro,
  onUpdateHardwareFx,
  onOpenRadialPreview,
  onClose,
}) => {
  const [tab, setTab] = useState<LabTab>(initialTab);
  const leftCanvasRef = useRef<HTMLCanvasElement>(null);
  const rightCanvasRef = useRef<HTMLCanvasElement>(null);
  const leftTraceRef = useRef<[number, number][]>([]);
  const rightTraceRef = useRef<[number, number][]>([]);
  const [traceVersion, setTraceVersion] = useState(0);

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
      const r = W * 0.4;

      ctx.clearRect(0, 0, W, H);

      // Subtle radar concentric rings (50% & 100%)
      ctx.strokeStyle = "rgba(255,255,255,0.06)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(cx, cy, r * 0.5, 0, Math.PI * 2);
      ctx.stroke();

      // Outer reference circle
      ctx.strokeStyle = "rgba(125, 211, 252, 0.28)";
      ctx.lineWidth = 1.4;
      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      ctx.stroke();

      // Degree tick marks every 45 deg
      ctx.strokeStyle = "rgba(255,255,255,0.16)";
      for (let deg = 0; deg < 360; deg += 45) {
        const rad = (deg * Math.PI) / 180;
        ctx.beginPath();
        ctx.moveTo(cx + Math.cos(rad) * (r - 5), cy + Math.sin(rad) * (r - 5));
        ctx.lineTo(cx + Math.cos(rad) * (r + 4), cy + Math.sin(rad) * (r + 4));
        ctx.stroke();
      }

      // Deadzone inner ring (12%)
      ctx.fillStyle = "rgba(76, 141, 255, 0.08)";
      ctx.beginPath();
      ctx.arc(cx, cy, r * 0.14, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = "rgba(76, 141, 255, 0.4)";
      ctx.setLineDash([2, 3]);
      ctx.beginPath();
      ctx.arc(cx, cy, r * 0.14, 0, Math.PI * 2);
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
        ctx.fillStyle = "rgba(56, 189, 248, 0.65)";
        for (const [tx, ty] of trace) {
          ctx.fillRect(cx + tx * r - 1.5, cy - ty * r - 1.5, 3, 3);
        }
      }

      // Live vector & glowing reticle
      const px = cx + sx * r;
      const py = cy - sy * r;
      ctx.strokeStyle = "rgba(125, 211, 252, 0.65)";
      ctx.lineWidth = 1.6;
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.lineTo(px, py);
      ctx.stroke();

      ctx.fillStyle = "rgba(56, 189, 248, 0.25)";
      ctx.beginPath();
      ctx.arc(px, py, 10, 0, Math.PI * 2);
      ctx.fill();

      ctx.fillStyle = "#7dd3fc";
      ctx.beginPath();
      ctx.arc(px, py, 4.5, 0, Math.PI * 2);
      ctx.fill();
    };

    drawStick(leftCanvasRef.current, snapshot.leftStick.x, snapshot.leftStick.y, leftTraceRef.current);
    drawStick(rightCanvasRef.current, snapshot.rightStick.x, snapshot.rightStick.y, rightTraceRef.current);
  }, [tab, snapshot.leftStick.x, snapshot.leftStick.y, snapshot.rightStick.x, snapshot.rightStick.y, traceVersion]);

  const leftDriftPct = (Math.hypot(snapshot.leftStick.x, snapshot.leftStick.y) * 100).toFixed(1);
  const rightDriftPct = (Math.hypot(snapshot.rightStick.x, snapshot.rightStick.y) * 100).toFixed(1);
  const activeRgb = hardwareFx.customRgb;
  const activeRgbCss = `rgb(${activeRgb[0]}, ${activeRgb[1]}, ${activeRgb[2]})`;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal-card"
        style={{ width: 720, maxHeight: "90vh" }}
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
                background: "rgba(56, 189, 248, 0.12)",
                border: "1px solid rgba(125, 211, 252, 0.3)",
                color: "#7dd3fc",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Cpu size={20} />
            </div>
            <div style={{ flex: 1 }}>
              <div className="eyebrow">HARDWARE &amp; SENSOR LAB · v2.0</div>
              <h2 style={{ margin: "2px 0 0", fontSize: 18, fontWeight: 600 }}>
                Calibration, 6-Axis Gyro, Adaptive Triggers &amp; API
              </h2>
            </div>
            <button type="button" className="btn btn-sm" onClick={onClose} aria-label="Close">
              <X size={15} />
            </button>
          </div>

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
                  <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11.5, color: "var(--text-secondary)", marginBottom: 6 }}>
                    <span style={{ fontFamily: "var(--font-mono)" }}>LEFT STICK · POLAR</span>
                    <span style={{ fontFamily: "var(--font-mono)", color: "#7dd3fc" }}>DRIFT {leftDriftPct}%</span>
                  </div>
                  <canvas ref={leftCanvasRef} width={210} height={210} style={{ width: 186, height: 186, margin: "0 auto", display: "block" }} />
                  <div style={{ fontSize: 10.5, fontFamily: "var(--font-mono)", color: "var(--text-tertiary)", marginTop: 4 }}>
                    X: {snapshot.leftStick.x.toFixed(3)} · Y: {snapshot.leftStick.y.toFixed(3)}
                  </div>
                </div>
                <div className="product-panel" style={{ padding: 14, textAlign: "center" }}>
                  <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11.5, color: "var(--text-secondary)", marginBottom: 6 }}>
                    <span style={{ fontFamily: "var(--font-mono)" }}>RIGHT STICK · POLAR</span>
                    <span style={{ fontFamily: "var(--font-mono)", color: "#7dd3fc" }}>DRIFT {rightDriftPct}%</span>
                  </div>
                  <canvas ref={rightCanvasRef} width={210} height={210} style={{ width: 186, height: 186, margin: "0 auto", display: "block" }} />
                  <div style={{ fontSize: 10.5, fontFamily: "var(--font-mono)", color: "var(--text-tertiary)", marginTop: 4 }}>
                    X: {snapshot.rightStick.x.toFixed(3)} · Y: {snapshot.rightStick.y.toFixed(3)}
                  </div>
                </div>
              </div>

              <div className="product-panel" style={{ padding: 12, display: "flex", alignItems: "center", justifyContent: "space-between", fontSize: 12 }}>
                <div style={{ display: "flex", gap: 18, fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>
                  <span>POLLING: <b style={{ color: "var(--green)" }}>{snapshot.pollingRateHz ?? 120} Hz</b></span>
                  <span>LATENCY: <b style={{ color: "#7dd3fc" }}>{(1000 / (snapshot.pollingRateHz ?? 120)).toFixed(1)} ms</b></span>
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
            <>
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
                  <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>Activation gate</span>
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

              <div className="product-panel" style={{ padding: 12, display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 10, fontFamily: "var(--font-mono)", fontSize: 11 }}>
                {(["PITCH (X)", "YAW (Y)", "ROLL (Z)"] as const).map((axis, i) => {
                  const val = snapshot.gyro?.[i] ?? 0;
                  const pct = Math.min(100, Math.abs(val) * 100);
                  return (
                    <div key={axis} style={{ display: "flex", flexDirection: "column", gap: 5 }}>
                      <div style={{ display: "flex", justifyContent: "space-between", color: "var(--text-secondary)" }}>
                        <span>{axis}</span>
                        <span style={{ color: "#7dd3fc" }}>{val.toFixed(2)}</span>
                      </div>
                      <div style={{ height: 4, borderRadius: 99, background: "rgba(255,255,255,0.07)", overflow: "hidden" }}>
                        <div style={{ width: `${Math.max(6, pct)}%`, height: "100%", background: "#38bdf8" }} />
                      </div>
                    </div>
                  );
                })}
              </div>
            </>
          )}

          {tab === "fx" && (
            <>
              <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 10 }}>
                <span style={{ fontSize: 12, fontWeight: 600, color: "var(--text-secondary)" }}>
                  DualSense Adaptive Triggers (USB HID 0x02 Force Curves)
                </span>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                  {(["leftTriggerMode", "rightTriggerMode"] as const).map((key) => {
                    const label = key === "leftTriggerMode" ? "Left Trigger (L2)" : "Right Trigger (R2)";
                    const cur = TRIGGER_MODES.find((m) => m.value === hardwareFx[key]) ?? TRIGGER_MODES[0];
                    return (
                      <div key={key} style={{ display: "flex", flexDirection: "column", gap: 7, padding: 10, borderRadius: 9, background: "rgba(0,0,0,0.24)", border: "1px solid var(--border-subtle)" }}>
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                          <span style={{ fontSize: 11.5, color: "var(--text-secondary)", fontWeight: 500 }}>{label}</span>
                          <svg width="80" height="38" viewBox="0 0 80 38">
                            <path d={cur.curve} fill="none" stroke="#7dd3fc" strokeWidth="1.8" strokeLinecap="round" />
                          </svg>
                        </div>
                        <select
                          value={hardwareFx[key]}
                          onChange={(e) =>
                            onUpdateHardwareFx({
                              ...hardwareFx,
                              [key]: e.target.value as AdaptiveTriggerMode,
                            })
                          }
                        >
                          {TRIGGER_MODES.map((m) => (
                            <option key={m.value} value={m.value}>
                              {m.label} — {m.desc}
                            </option>
                          ))}
                        </select>
                      </div>
                    );
                  })}
                </div>
              </div>

              <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 10 }}>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                  <span style={{ fontSize: 12, fontWeight: 600, color: "var(--text-secondary)" }}>
                    Dynamic Lightbar RGB Telemetry
                  </span>
                  <div style={{ width: 64, height: 6, borderRadius: 99, background: hardwareFx.lightbarMode === "off" ? "#1e293b" : activeRgbCss, boxShadow: hardwareFx.lightbarMode === "off" ? "none" : `0 0 12px ${activeRgbCss}` }} />
                </div>
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
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                  {RGB_PRESETS.map((p) => (
                    <button
                      key={p.name}
                      type="button"
                      className="btn btn-sm"
                      onClick={() =>
                        onUpdateHardwareFx({
                          ...hardwareFx,
                          lightbarMode: "customColor",
                          customRgb: p.rgb,
                        })
                      }
                      style={{ gap: 6, fontSize: 11 }}
                    >
                      <span style={{ width: 9, height: 9, borderRadius: "50%", background: `rgb(${p.rgb.join(",")})` }} />
                      {p.name}
                    </button>
                  ))}
                </div>
              </div>
            </>
          )}

          {tab === "api" && (
            <>
              <div className="product-panel" style={{ padding: 14, display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12 }}>
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600 }}>8-Slice Obsidian Radial Pie Menu</div>
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
