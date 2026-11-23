import React, { useState } from "react";
import type {
  ControllerSnapshot,
  CursorConfiguration,
  StickRoleChoice,
  StickSide,
} from "../types/models";

interface StickRoleSheetProps {
  side: StickSide;
  cursor: CursorConfiguration;
  snapshot: ControllerSnapshot;
  onSave: (role: StickRoleChoice) => void;
  onClose: () => void;
}

export const StickRoleSheet: React.FC<StickRoleSheetProps> = ({
  side,
  cursor,
  snapshot,
  onSave,
  onClose,
}) => {
  const initialRole: StickRoleChoice =
    cursor.primaryStick === side
      ? "primary"
      : cursor.precisionStick === side
      ? "precision"
      : "off";

  const [role, setRole] = useState<StickRoleChoice>(initialRole);
  const stick = side === "left" ? snapshot.leftStick : snapshot.rightStick;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal-card"
        style={{ width: 420, padding: 24, gap: 18 }}
        onClick={(e) => e.stopPropagation()}
      >
        <div>
          <h2 style={{ margin: 0, fontSize: 19, fontWeight: 600 }}>
            {side === "left" ? "Left Stick" : "Right Stick"}
          </h2>
          <p style={{ margin: "4px 0 0", fontSize: 12.5, color: "var(--text-secondary)" }}>
            Choose how this analog stick controls the Windows pointer.
          </p>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 18 }}>
          <div
            style={{
              width: 110,
              height: 110,
              borderRadius: "50%",
              background: "rgba(0,0,0,0.22)",
              border: "1px solid var(--border-strong)",
              position: "relative",
              flexShrink: 0,
            }}
          >
            <div
              style={{
                width: 28,
                height: 28,
                borderRadius: "50%",
                background: "rgba(69, 135, 247, 0.35)",
                border: "2px solid var(--accent)",
                position: "absolute",
                left: 41 + stick.x * 26,
                top: 41 - stick.y * 26,
                transition: "left 50ms linear, top 50ms linear",
              }}
            />
          </div>
          <div>
            <div style={{ fontSize: 12, color: "var(--text-secondary)", marginBottom: 4 }}>
              Live vector
            </div>
            <div
              style={{
                fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace",
                fontSize: 13,
              }}
            >
              x {stick.x.toFixed(2)} &nbsp; y {stick.y.toFixed(2)}
            </div>
          </div>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {(
            [
              ["primary", "Primary cursor", "Fast full-screen pointer navigation"],
              ["precision", "Precision cursor", "Fine targeting for small controls"],
              ["off", "Off", "Disable cursor movement on this stick"],
            ] as const
          ).map(([val, title, sub]) => (
            <label
              key={val}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 10,
                padding: "10px 12px",
                borderRadius: 10,
                border: `1px solid ${role === val ? "var(--accent)" : "var(--border-subtle)"}`,
                background: role === val ? "var(--accent-soft)" : "var(--bg-panel)",
                cursor: "pointer",
              }}
            >
              <input
                type="radio"
                name="stick-role"
                checked={role === val}
                onChange={() => setRole(val)}
              />
              <div>
                <div style={{ fontWeight: 600, fontSize: 13 }}>{title}</div>
                <div style={{ fontSize: 11.5, color: "var(--text-secondary)" }}>{sub}</div>
              </div>
            </label>
          ))}
        </div>

        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8 }}>
          <button type="button" className="btn" onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="btn btn-primary" onClick={() => onSave(role)}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
};
