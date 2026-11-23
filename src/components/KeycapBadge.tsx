import React from "react";
import type { ActionType, ControllerActionMapping } from "../types/models";
import { formatShortcut, mappingSummary } from "../utils/shortcuts";

interface KeycapBadgeProps {
  mapping: ControllerActionMapping;
  align?: "left" | "right";
  large?: boolean;
  compact?: boolean;
}

const COMPACT_ACTION_CAPS: Partial<Record<ActionType, string[]>> = {
  leftMouseHold: ["Drag"],
  leftClick: ["L-Click"],
  rightClick: ["R-Click"],
  middleClick: ["M-Click"],
  doubleClick: ["2×Click"],
  scrollUp: ["Scroll", "↑"],
  scrollDown: ["Scroll", "↓"],
  scrollLeft: ["Scroll", "←"],
  scrollRight: ["Scroll", "→"],
  switchSpaceLeft: ["Desk", "←"],
  switchSpaceRight: ["Desk", "→"],
  crossEdgeLeft: ["Edge", "←"],
  crossEdgeRight: ["Edge", "→"],
  crossEdgeUp: ["Edge", "↑"],
  crossEdgeDown: ["Edge", "↓"],
  toggleCursorSpeeds: ["Speed"],
};

function shortenKeyLabel(part: string): string {
  if (part === "Backspace") return "Bksp";
  if (part === "Shift") return "⇧";
  return part;
}

export const KeycapBadge: React.FC<KeycapBadgeProps> = ({
  mapping,
  align = "left",
  large = false,
  compact = false,
}) => {
  if (mapping.actionType === "keyboardShortcut" && mapping.shortcut) {
    const formatted = formatShortcut(mapping.shortcut);
    const parts = formatted.includes(" + ")
      ? formatted.split(" + ")
      : formatted.split("+");
    return (
      <span
        className="kbd-group"
        style={{ justifyContent: align === "right" ? "flex-end" : "flex-start" }}
      >
        {parts.map((part, idx) => (
          <span
            key={`${part}-${idx}`}
            className={`kbd-cap ${large ? "kbd-cap-lg" : ""}`}
          >
            {compact && parts.length >= 3 ? shortenKeyLabel(part) : compact && part === "Backspace" ? "Bksp" : part}
          </span>
        ))}
      </span>
    );
  }

  if (compact && COMPACT_ACTION_CAPS[mapping.actionType]) {
    const caps = COMPACT_ACTION_CAPS[mapping.actionType]!;
    return (
      <span
        className="kbd-group"
        style={{ justifyContent: align === "right" ? "flex-end" : "flex-start" }}
      >
        {caps.map((cap, idx) => (
          <span key={`${cap}-${idx}`} className="kbd-cap">
            {cap}
          </span>
        ))}
      </span>
    );
  }

  const label = mappingSummary(mapping);
  const isNone = mapping.actionType === "none";
  return (
    <span
      style={{
        fontSize: large ? 14 : 11,
        fontWeight: isNone ? 400 : 500,
        color: isNone ? "var(--text-tertiary)" : "var(--text-primary)",
        overflow: "hidden",
        textOverflow: "ellipsis",
        whiteSpace: "nowrap",
      }}
    >
      {label}
    </span>
  );
};
