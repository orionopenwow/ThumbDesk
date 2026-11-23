import React, { useEffect, useRef, useState } from "react";
import xboxStudioUrl from "../assets/controllers/xbox_studio.png";
import psStudioUrl from "../assets/controllers/playstation_studio.png";
import type {
  ControllerControlID,
  ControllerFamily,
  ControllerProfile,
  ControllerSnapshot,
  StickSide,
} from "../types/models";
import {
  ARTWORK_DESIGN_WIDTH,
  controlDisplayName,
  controlGlyph,
  getControlAmount,
  getStickVector,
  hasExplicitOverride,
  hotspotsForFamily,
  isControlActive,
  type HotspotRegion,
} from "../utils/controllerLayout";
import type { ControllerShellColor } from "../utils/shellColors";

interface ControllerArtworkProps {
  family: ControllerFamily;
  shellColor: ControllerShellColor;
  width: number;
  height: number;
  snapshot: ControllerSnapshot;
  profile: ControllerProfile;
  activeModifier: ControllerControlID | null;
  activeAppId: string | null;
  hoveredControl: ControllerControlID | null;
  onHoverControl: (control: ControllerControlID | null) => void;
  onSelectControl: (control: ControllerControlID) => void;
}

function shellColorFilter(color: ControllerShellColor, family: ControllerFamily): string {
  switch (color) {
    case "original":
      return "none";
    case "graphite":
      return family === "playStation" ? "brightness(0.45) contrast(1.18)" : "none";
    case "white":
      return family === "xbox" ? "invert(0.86) hue-rotate(180deg) brightness(1.08)" : "none";
    case "blue":
      return "sepia(0.35) saturate(2.6) hue-rotate(185deg) brightness(0.95)";
    case "pink":
      return "sepia(0.35) saturate(2.4) hue-rotate(290deg) brightness(0.98)";
    case "green":
      return "sepia(0.35) saturate(2.3) hue-rotate(65deg) brightness(0.95)";
  }
}

const LiveThumbstick: React.FC<{
  region: HotspotRegion;
  scale: number;
  side: StickSide;
  snapshot: ControllerSnapshot;
  hovered: boolean;
  overridden: boolean;
  onHover: (v: boolean) => void;
  onClick: () => void;
}> = ({ region, scale, side, snapshot, hovered, overridden, onHover, onClick }) => {
  const vec = getStickVector(snapshot, side);
  const pressed =
    side === "left"
      ? snapshot.leftStick.pressed || snapshot.pressedControls.includes("leftThumbstickButton")
      : snapshot.rightStick.pressed || snapshot.pressedControls.includes("rightThumbstickButton");
  const active = Math.hypot(vec.dx, vec.dy) > 0.1 || pressed;
  const size = region.w * scale;
  const deflection = 24 * scale;

  return (
    <button
      type="button"
      onMouseEnter={() => onHover(true)}
      onMouseLeave={() => onHover(false)}
      onClick={onClick}
      aria-label={side === "left" ? "Left Stick Click" : "Right Stick Click"}
      style={{
        position: "absolute",
        left: (region.x - region.w / 2) * scale,
        top: (region.y - region.h / 2) * scale,
        width: size,
        height: size,
        borderRadius: "50%",
        background: pressed ? "rgba(59, 130, 246, 0.28)" : "transparent",
        border: `${2 * scale}px solid ${
          active || hovered
            ? "rgba(96, 165, 250, 0.9)"
            : overridden
            ? "rgba(59, 130, 246, 0.45)"
            : "transparent"
        }`,
        boxShadow: active ? "0 0 18px rgba(59, 130, 246, 0.45)" : "none",
        padding: 0,
        overflow: "visible",
        transition: "border-color 120ms ease, background 120ms ease",
      }}
    >
      {active && (
        <svg
          width={size}
          height={size}
          style={{ position: "absolute", inset: 0, pointerEvents: "none" }}
        >
          <line
            x1={size / 2}
            y1={size / 2}
            x2={size / 2 + vec.dx * deflection}
            y2={size / 2 + vec.dy * deflection}
            stroke="rgba(96, 165, 250, 0.85)"
            strokeWidth={3.5 * scale}
            strokeLinecap="round"
          />
          <circle
            cx={size / 2 + vec.dx * deflection}
            cy={size / 2 + vec.dy * deflection}
            r={7 * scale}
            fill="#60A5FA"
          />
        </svg>
      )}
    </button>
  );
};

const LiveTrigger: React.FC<{
  region: HotspotRegion;
  scale: number;
  label: string;
  amount: number;
  hovered: boolean;
  overridden: boolean;
  onHover: (v: boolean) => void;
  onClick: () => void;
}> = ({ region, scale, label, amount, hovered, overridden, onHover, onClick }) => {
  const w = region.w * scale;
  const h = region.h * scale;
  const active = amount > 0.02;
  const visible = active || hovered;

  return (
    <button
      type="button"
      onMouseEnter={() => onHover(true)}
      onMouseLeave={() => onHover(false)}
      onClick={onClick}
      aria-label={label}
      style={{
        position: "absolute",
        left: (region.x - region.w / 2) * scale,
        top: (region.y - region.h / 2) * scale,
        width: w,
        height: h,
        borderRadius: 14 * scale,
        background: visible ? "rgba(33, 41, 54, 0.72)" : "transparent",
        border: `${(active ? 2.5 : 1.5) * scale}px solid ${
          visible
            ? "rgba(96, 165, 250, 0.95)"
            : overridden
            ? "rgba(59, 130, 246, 0.45)"
            : "transparent"
        }`,
        overflow: "hidden",
        transform: `translateY(${6 * scale * amount}px)`,
        transition: "transform 55ms linear, background 120ms ease, border-color 120ms ease",
        padding: 0,
      }}
    >
      {active && (
        <div
          style={{
            position: "absolute",
            left: 0,
            right: 0,
            bottom: 0,
            height: `${amount * 100}%`,
            background: "rgba(59, 130, 246, 0.85)",
            transition: "height 55ms linear",
          }}
        />
      )}
      {visible && (
        <span
          style={{
            position: "relative",
            zIndex: 1,
            color: "#ffffff",
            fontWeight: 700,
            fontSize: Math.max(10, 14 * scale),
          }}
        >
          {label}
        </span>
      )}
    </button>
  );
};

const HardwareHitTarget: React.FC<{
  region: HotspotRegion;
  scale: number;
  family: ControllerFamily;
  pressed: boolean;
  hovered: boolean;
  overridden: boolean;
  isModifier: boolean;
  onHover: (v: boolean) => void;
  onClick: () => void;
}> = ({
  region,
  scale,
  family,
  pressed,
  hovered,
  overridden,
  isModifier,
  onHover,
  onClick,
}) => {
  const [releaseFlash, setReleaseFlash] = useState(false);
  const prevPressed = useRef(pressed);

  useEffect(() => {
    if (prevPressed.current && !pressed) {
      setReleaseFlash(true);
      const t = window.setTimeout(() => setReleaseFlash(false), 110);
      prevPressed.current = pressed;
      return () => window.clearTimeout(t);
    }
    prevPressed.current = pressed;
  }, [pressed]);

  const lit = pressed || releaseFlash;
  const highlighted = lit || hovered;
  const fill = lit
    ? "rgba(59, 130, 246, 0.82)"
    : hovered
    ? "rgba(59, 130, 246, 0.22)"
    : isModifier
    ? "rgba(59, 130, 246, 0.22)"
    : "transparent";

  return (
    <button
      type="button"
      onMouseEnter={() => onHover(true)}
      onMouseLeave={() => onHover(false)}
      onClick={onClick}
      aria-label={controlDisplayName(region.control, family)}
      style={{
        position: "absolute",
        left: (region.x - region.w / 2) * scale,
        top: (region.y - region.h / 2) * scale,
        width: region.w * scale,
        height: region.h * scale,
        borderRadius: region.r * scale,
        background: fill,
        border: `${(lit ? 2.5 : 1.5) * scale}px solid ${
          highlighted
            ? "rgba(96, 165, 250, 0.95)"
            : overridden
            ? "rgba(59, 130, 246, 0.45)"
            : "transparent"
        }`,
        boxShadow: highlighted ? "0 0 14px rgba(59, 130, 246, 0.45)" : "none",
        transition: lit ? "none" : "background 100ms ease-out, border-color 100ms ease-out",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 0,
      }}
    >
      {lit && (
        <span
          style={{
            color: "#ffffff",
            fontWeight: 700,
            fontSize: Math.max(10, 17 * scale),
            pointerEvents: "none",
          }}
        >
          {controlGlyph(region.control, family)}
        </span>
      )}
    </button>
  );
};

export const ControllerArtwork: React.FC<ControllerArtworkProps> = ({
  family,
  shellColor,
  width,
  height,
  snapshot,
  profile,
  activeModifier,
  activeAppId,
  hoveredControl,
  onHoverControl,
  onSelectControl,
}) => {
  const scale = width / ARTWORK_DESIGN_WIDTH;
  const regions = hotspotsForFamily(family);
  const src = family === "playStation" ? psStudioUrl : xboxStudioUrl;
  const filter = shellColorFilter(shellColor, family);

  return (
    <div style={{ position: "relative", width, height }}>
      <img
        src={src}
        alt={family === "playStation" ? "PlayStation Controller" : "Xbox Controller"}
        draggable={false}
        style={{
          width,
          height,
          objectFit: "contain",
          pointerEvents: "none",
          filter,
          transition: "filter 180ms ease",
        }}
      />
      {regions.map((region) => {
        const hovered = hoveredControl === region.control;
        const overridden = hasExplicitOverride(
          profile,
          region.control,
          activeModifier,
          activeAppId
        );

        if (region.control === "leftThumbstickButton" || region.control === "rightThumbstickButton") {
          return (
            <LiveThumbstick
              key={region.control}
              region={region}
              scale={scale}
              side={region.control === "leftThumbstickButton" ? "left" : "right"}
              snapshot={snapshot}
              hovered={hovered}
              overridden={overridden}
              onHover={(v) => onHoverControl(v ? region.control : null)}
              onClick={() => onSelectControl(region.control)}
            />
          );
        }

        if (region.control === "leftTrigger" || region.control === "rightTrigger") {
          return (
            <LiveTrigger
              key={region.control}
              region={region}
              scale={scale}
              label={controlDisplayName(region.control, family)}
              amount={getControlAmount(snapshot, region.control)}
              hovered={hovered}
              overridden={overridden}
              onHover={(v) => onHoverControl(v ? region.control : null)}
              onClick={() => onSelectControl(region.control)}
            />
          );
        }

        return (
          <HardwareHitTarget
            key={region.control}
            region={region}
            scale={scale}
            family={family}
            pressed={isControlActive(snapshot, region.control)}
            hovered={hovered}
            overridden={overridden}
            isModifier={activeModifier === region.control}
            onHover={(v) => onHoverControl(v ? region.control : null)}
            onClick={() => onSelectControl(region.control)}
          />
        );
      })}
    </div>
  );
};
