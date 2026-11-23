import React, { useState } from "react";
import { Disc, X } from "lucide-react";
import type { ControllerSnapshot, RadialMenuSlice } from "../types/models";

interface RadialMenuOverlayProps {
  slices: RadialMenuSlice[];
  snapshot: ControllerSnapshot;
  onSelectSlice: (sliceId: string) => void;
  onClose: () => void;
}

export const RadialMenuOverlay: React.FC<RadialMenuOverlayProps> = ({
  slices,
  snapshot,
  onSelectSlice,
  onClose,
}) => {
  const [hoveredIdx, setHoveredIdx] = useState<number | null>(0);

  const sx = snapshot.leftStick.x;
  const sy = snapshot.leftStick.y;
  let stickIdx: number | null = null;
  if (Math.hypot(sx, sy) > 0.35 && slices.length > 0) {
    const angle = (Math.atan2(sx, sy) * 180) / Math.PI;
    const norm = (angle + 360 + 360 / slices.length / 2) % 360;
    stickIdx = Math.floor(norm / (360 / slices.length)) % slices.length;
  }

  const activeIdx = stickIdx ?? hoveredIdx;
  const count = Math.max(1, slices.length);
  const cx = 200;
  const cy = 200;
  const outerR = 176;
  const innerR = 66;

  const describeArc = (i: number) => {
    const step = (Math.PI * 2) / count;
    const start = i * step - Math.PI / 2 - step / 2 + 0.028;
    const end = (i + 1) * step - Math.PI / 2 - step / 2 - 0.028;
    const x1 = cx + outerR * Math.cos(start);
    const y1 = cy + outerR * Math.sin(start);
    const x2 = cx + outerR * Math.cos(end);
    const y2 = cy + outerR * Math.sin(end);
    const x3 = cx + innerR * Math.cos(end);
    const y3 = cy + innerR * Math.sin(end);
    const x4 = cx + innerR * Math.cos(start);
    const y4 = cy + innerR * Math.sin(start);
    return `M ${x1} ${y1} A ${outerR} ${outerR} 0 0 1 ${x2} ${y2} L ${x3} ${y3} A ${innerR} ${innerR} 0 0 0 ${x4} ${y4} Z`;
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        style={{
          position: "relative",
          width: 400,
          height: 400,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <svg viewBox="0 0 400 400" width={400} height={400}>
          {/* Precision outer compass ring */}
          <circle
            cx={cx}
            cy={cy}
            r={outerR + 9}
            fill="none"
            stroke="rgba(125, 211, 252, 0.14)"
            strokeWidth={1}
            strokeDasharray="3 5"
          />

          {slices.map((s, i) => {
            const isActive = activeIdx === i;
            const midAngle = (i * Math.PI * 2) / count - Math.PI / 2;
            const lx = cx + ((outerR + innerR) / 2 + 4) * Math.cos(midAngle);
            const ly = cy + ((outerR + innerR) / 2 + 4) * Math.sin(midAngle);
            const idxX = cx + (innerR + 18) * Math.cos(midAngle);
            const idxY = cy + (innerR + 18) * Math.sin(midAngle);
            return (
              <g
                key={s.id}
                style={{ cursor: "pointer" }}
                onMouseEnter={() => setHoveredIdx(i)}
                onClick={() => onSelectSlice(s.id)}
              >
                <path
                  d={describeArc(i)}
                  fill={isActive ? "rgba(56, 189, 248, 0.22)" : "rgba(15, 19, 27, 0.94)"}
                  stroke={isActive ? "#7dd3fc" : "rgba(255,255,255,0.11)"}
                  strokeWidth={isActive ? 1.8 : 1}
                />
                <text
                  x={idxX}
                  y={idxY + 3}
                  textAnchor="middle"
                  fill={isActive ? "#7dd3fc" : "rgba(148,163,184,0.42)"}
                  fontSize="8.5"
                  fontFamily="JetBrains Mono, monospace"
                >
                  0{i + 1}
                </text>
                <text
                  x={lx}
                  y={ly + 4}
                  textAnchor="middle"
                  fill={isActive ? "#ffffff" : "#cbd5e1"}
                  fontSize="11.5"
                  fontWeight={isActive ? 600 : 500}
                  fontFamily="Inter, sans-serif"
                >
                  {s.label}
                </text>
              </g>
            );
          })}
          <circle
            cx={cx}
            cy={cy}
            r={innerR - 6}
            fill="#0b0e14"
            stroke="rgba(125, 211, 252, 0.35)"
            strokeWidth={1.3}
          />
        </svg>

        <div
          style={{
            position: "absolute",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 4,
            pointerEvents: "none",
          }}
        >
          <Disc size={18} color="#7dd3fc" />
          <span style={{ fontSize: 11, fontWeight: 600, color: "#f1f5f9" }}>
            {activeIdx !== null ? slices[activeIdx]?.label : "RADIAL"}
          </span>
          <span style={{ fontSize: 9, fontFamily: "var(--font-mono)", color: "#64748b" }}>
            FLICK OR CLICK
          </span>
        </div>

        <button
          type="button"
          className="btn btn-sm"
          onClick={onClose}
          style={{ position: "absolute", top: -6, right: -6 }}
          aria-label="Close radial menu"
        >
          <X size={13} />
        </button>
      </div>
    </div>
  );
};
