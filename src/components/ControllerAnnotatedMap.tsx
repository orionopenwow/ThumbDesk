import React, { useEffect, useRef, useState } from "react";
import type {
  ControllerControlID,
  ControllerFamily,
  ControllerProfile,
  ControllerSnapshot,
} from "../types/models";
import {
  computeCalloutLayout,
  controlDisplayName,
  hasExplicitOverride,
  isControlActive,
  resolveEffectiveMapping,
} from "../utils/controllerLayout";
import type { ControllerShellColor } from "../utils/shellColors";
import { ControllerArtwork } from "./ControllerArtwork";
import { KeycapBadge } from "./KeycapBadge";

interface ControllerAnnotatedMapProps {
  family: ControllerFamily;
  shellColor: ControllerShellColor;
  showsLabels: boolean;
  snapshot: ControllerSnapshot;
  profile: ControllerProfile;
  activeModifier: ControllerControlID | null;
  activeAppId: string | null;
  hoveredControl: ControllerControlID | null;
  onHoverControl: (control: ControllerControlID | null) => void;
  onSelectControl: (control: ControllerControlID) => void;
}

function controlAccentColor(control: ControllerControlID, family: ControllerFamily): string {
  if (family === "playStation") {
    switch (control) {
      case "buttonNorth":
        return "#5EEAD4"; // Triangle teal
      case "buttonEast":
        return "#F87171"; // Circle coral
      case "buttonSouth":
        return "#818CF8"; // Cross periwinkle
      case "buttonWest":
        return "#C084FC"; // Square lavender
    }
  } else {
    switch (control) {
      case "buttonSouth":
        return "#4ADE80"; // A green
      case "buttonEast":
        return "#F87171"; // B red
      case "buttonWest":
        return "#60A5FA"; // X blue
      case "buttonNorth":
        return "#FACC15"; // Y yellow
    }
  }
  if (
    control === "dpadUp" ||
    control === "dpadDown" ||
    control === "dpadLeft" ||
    control === "dpadRight"
  ) {
    return "#7DD3FC";
  }
  return "#CBD5E1";
}

function compactControlLabel(control: ControllerControlID, family: ControllerFamily): string {
  switch (control) {
    case "dpadUp":
      return "D-Up";
    case "dpadDown":
      return "D-Down";
    case "dpadLeft":
      return "D-Left";
    case "dpadRight":
      return "D-Right";
    case "touchpadButton":
      return "Touch";
    default:
      return controlDisplayName(control, family);
  }
}

export const ControllerAnnotatedMap: React.FC<ControllerAnnotatedMapProps> = ({
  family,
  shellColor,
  showsLabels,
  snapshot,
  profile,
  activeModifier,
  activeAppId,
  hoveredControl,
  onHoverControl,
  onSelectControl,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 920, height: 438 });

  useEffect(() => {
    const el = containerRef.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver((entries) => {
      const rect = entries[0]?.contentRect;
      if (rect && rect.width > 100) {
        setSize({ width: rect.width, height: Math.max(430, rect.height) });
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  if (!showsLabels) {
    const artH = 420;
    const artW = artH / 0.66;
    return (
      <div
        ref={containerRef}
        style={{
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          minHeight: 448,
          padding: "8px 12px",
        }}
      >
        <div style={{ transform: "scale(1.06)" }}>
          <ControllerArtwork
            family={family}
            shellColor={shellColor}
            width={artW}
            height={artH}
            snapshot={snapshot}
            profile={profile}
            activeModifier={activeModifier}
            activeAppId={activeAppId}
            hoveredControl={hoveredControl}
            onHoverControl={onHoverControl}
            onSelectControl={onSelectControl}
          />
        </div>
      </div>
    );
  }

  const layout = computeCalloutLayout(family, size.width, size.height);

  return (
    <div
      ref={containerRef}
      style={{
        position: "relative",
        width: "100%",
        height: 438,
        marginTop: 0,
      }}
    >
      {/* Crisp Hairline Elbow Leader Lines & Glowing Anchor Dots */}
      <svg
        width={size.width}
        height={size.height}
        style={{ position: "absolute", inset: 0, pointerEvents: "none" }}
      >
        {layout.items.map((item) => {
          const active =
            hoveredControl === item.control ||
            isControlActive(snapshot, item.control) ||
            activeModifier === item.control;
          const overridden = hasExplicitOverride(
            profile,
            item.control,
            activeModifier,
            activeAppId
          );
          const stroke = active
            ? "#60A5FA"
            : overridden
            ? "rgba(96, 165, 250, 0.55)"
            : "rgba(148, 163, 184, 0.30)";

          return (
            <g key={item.control}>
              <polyline
                points={`${item.originX},${item.originY} ${item.elbowX},${item.elbowY} ${item.anchorX},${item.anchorY}`}
                fill="none"
                stroke={stroke}
                strokeWidth={active ? 1.75 : 1}
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              {/* Node dot at pill inner edge */}
              <circle
                cx={item.originX}
                cy={item.originY}
                r={2.2}
                fill={active ? "#60A5FA" : "rgba(148, 163, 184, 0.55)"}
              />
              {/* Glowing anchor dot on hardware button */}
              {active && (
                <circle
                  cx={item.anchorX}
                  cy={item.anchorY}
                  r={6.5}
                  fill="rgba(59, 130, 246, 0.3)"
                />
              )}
              <circle
                cx={item.anchorX}
                cy={item.anchorY}
                r={active ? 3.2 : 2.3}
                fill={active ? "#60A5FA" : "rgba(148, 163, 184, 0.72)"}
              />
            </g>
          );
        })}
      </svg>

      {/* Centered Hero Controller Artwork */}
      <div
        style={{
          position: "absolute",
          left: layout.artworkX,
          top: layout.artworkY,
          filter: "drop-shadow(0 20px 32px rgba(0, 0, 0, 0.52))",
        }}
      >
        <ControllerArtwork
          family={family}
          shellColor={shellColor}
          width={layout.artworkW}
          height={layout.artworkH}
          snapshot={snapshot}
          profile={profile}
          activeModifier={activeModifier}
          activeAppId={activeAppId}
          hoveredControl={hoveredControl}
          onHoverControl={onHoverControl}
          onSelectControl={onSelectControl}
        />
      </div>

      {/* Left & Right Horizontal Capsule Callout Pills */}
      {layout.items.map((item) => {
        const mapping = resolveEffectiveMapping(
          profile,
          item.control,
          activeModifier,
          activeAppId
        );
        const overridden = hasExplicitOverride(
          profile,
          item.control,
          activeModifier,
          activeAppId
        );
        const active =
          hoveredControl === item.control ||
          isControlActive(snapshot, item.control) ||
          activeModifier === item.control;

        const pillH = 36;
        const pillClass = `callout-pill${active ? " active" : overridden ? " overridden" : ""}`;

        return (
          <button
            key={item.control}
            type="button"
            className={pillClass}
            onMouseEnter={() => onHoverControl(item.control)}
            onMouseLeave={() => onHoverControl(null)}
            onClick={() => onSelectControl(item.control)}
            style={{
              position: "absolute",
              left: item.labelX,
              top: item.originY - pillH / 2,
              width: item.labelW,
              height: pillH,
              display: "flex",
              flexDirection: "row",
              alignItems: "center",
              justifyContent: "space-between",
              gap: 8,
            }}
          >
            <span
              style={{
                fontSize: 12,
                fontWeight: 600,
                color: active ? "#FFFFFF" : controlAccentColor(item.control, family),
                whiteSpace: "nowrap",
                flexShrink: 0,
              }}
            >
              {compactControlLabel(item.control, family)}
            </span>
            <span
              style={{
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
                display: "flex",
                justifyContent: "flex-end",
              }}
            >
              <KeycapBadge mapping={mapping} align="right" compact />
            </span>
          </button>
        );
      })}
    </div>
  );
};
