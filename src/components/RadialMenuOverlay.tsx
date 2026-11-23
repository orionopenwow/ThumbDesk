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
  const [hoveredIdx, setHoveredIdx] = useState<number | null>(null);

  // Determine stick-selected slice if thumbstick is deflected
  const sx = snapshot.leftStick.x;
  const sy = snapshot.leftStick.y;
  let stickIdx: number | null = null;
  if (Math.hypot(sx, sy) > 0.35 && slices.length > 0) {
    // 0 deg at top (12 o'clock), clockwise
    const angle = (Math.atan2(sx, sy) * 180) / Math.PI;
    const norm = (angle + 360 + 360 / slices.length / 2) % 360;
    stickIdx = Math.floor(norm / (360 / slices.length)) % slices.length;
  }

  const activeIdx = hoveredIdx ?? stickIdx;
  const count = Math.max(1, slices.length);
  const cx = 190;
  const cy = 190;
  const outerR = 168;
  const innerR = 64;

  const describeArc = (i: number) => {
    const step = (Math.PI * 2) / count;
    const start = i * step - Math.PI / 2 - step / 2 + 0.03;
    const end = (i + 1) * step - Math.PI / 2 - step / 2 - 0.03;
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
          width: 380,
          height: 380,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <svg viewBox="0 0 380 380" width={380} height={380}>
          {slices.map((s, i) => {
            const isActive = activeIdx === i;
            const midAngle = (i * Math.PI * 2) / count - Math.PI / 2;
            const lx = cx + ((outerR + innerR) / 2) * Math.cos(midAngle);
            const ly = cy + ((outerR + innerR) / 2) * Math.sin(midAngle);
            return (
              <g
                key={s.id}
                style={{ cursor: "pointer" }}
                onMouseEnter={() => setHoveredIdx(i)}
                onMouseLeave={() => setHoveredIdx(null)}
                onClick={() => onSelectSlice(s.id)}
              >
                <path
                  d={describeArc(i)}
                  fill={isActive ? "rgba(76, 141, 255, 0.34)" : "rgba(17, 21, 28, 0.94)"}
                  stroke={isActive ? "#7dd3fc" : "rgba(255,255,255,0.12)"}
                  strokeWidth={isActive ? 1.8 : 1}
                />
                <text
                  x={lx}
                  y={ly + 4}
                  textAnchor="middle"
                  fill={isActive ? "#ffffff" : "#cbd5e1"}
                  fontSize="11"
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
            fill="#0d1015"
            stroke="rgba(255,255,255,0.14)"
            strokeWidth={1.2}
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
          <span style={{ fontSize: 10.5, fontFamily: "var(--font-mono)", color: "#94a3b8" }}>
            {activeIdx !== null ? slices[activeIdx]?.label : "RADIAL"}
          </span>
        </div>

        <button
          type="button"
          className="btn btn-sm"
          onClick={onClose}
          style={{ position: "absolute", top: -10, right: -10 }}
          aria-label="Close radial menu"
        >
          <X size={13} />
        </button>
      </div>
    </div>
  );
};
