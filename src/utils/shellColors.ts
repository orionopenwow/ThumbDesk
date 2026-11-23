import type { ControllerFamily } from "../types/models";

export type ControllerShellColor =
  | "original"
  | "graphite"
  | "white"
  | "blue"
  | "pink"
  | "green";

export interface ShellColorSpec {
  id: ControllerShellColor;
  label: string;
  palette: [string, string, string];
  usesDarkCheckmark: boolean;
}

const PALETTES: Record<Exclude<ControllerShellColor, "original">, [string, string, string]> = {
  graphite: ["#555B65", "#393E47", "#242830"],
  white: ["#FFFFFF", "#E5E8EF", "#A6AEBB"],
  blue: ["#6CAFFF", "#2874D4", "#154580"],
  pink: ["#FF9EC8", "#E95394", "#9F285D"],
  green: ["#B8EA75", "#76B83C", "#386623"],
};

export const SHELL_COLOR_OPTIONS: ControllerShellColor[] = [
  "original",
  "graphite",
  "white",
  "blue",
  "pink",
  "green",
];

export function getShellColorSpec(
  color: ControllerShellColor,
  family: ControllerFamily
): ShellColorSpec {
  const effective: Exclude<ControllerShellColor, "original"> =
    color === "original"
      ? family === "playStation"
        ? "white"
        : "graphite"
      : color;

  const labels: Record<ControllerShellColor, string> = {
    original: "Original",
    graphite: "Graphite",
    white: "White",
    blue: "Blue",
    pink: "Pink",
    green: "Green",
  };

  return {
    id: color,
    label: labels[color],
    palette: PALETTES[effective],
    usesDarkCheckmark: effective === "white" || effective === "green",
  };
}

export function loadSavedShellColor(family: ControllerFamily): ControllerShellColor {
  const key =
    family === "playStation"
      ? "controllerMap.playStationColor"
      : "controllerMap.xboxColor";
  try {
    const val = localStorage.getItem(key) as ControllerShellColor | null;
    if (val && SHELL_COLOR_OPTIONS.includes(val)) {
      return val;
    }
  } catch {
    // ignore storage errors
  }
  return "original";
}

export function saveShellColor(
  family: ControllerFamily,
  color: ControllerShellColor
): void {
  const key =
    family === "playStation"
      ? "controllerMap.playStationColor"
      : "controllerMap.xboxColor";
  try {
    localStorage.setItem(key, color);
  } catch {
    // ignore storage errors
  }
}

/**
 * Mutates the original Xbox or PlayStation SVG shell gradients (#shell / #white, #edge)
 * and rim fills (#shell-rim, #grip-shade) while preserving buttons, sticks, and lighting.
 */
export function recolorControllerSvg(
  rawSvg: string,
  family: ControllerFamily,
  color: ControllerShellColor
): string {
  if (color === "original") {
    return rawSvg;
  }
  const { palette } = getShellColorSpec(color, family);
  const parser = new DOMParser();
  const doc = parser.parseFromString(rawSvg, "image/svg+xml");

  const recolorGradient = (id: string) => {
    const grad = doc.getElementById(id);
    if (!grad) return;
    const stops = grad.querySelectorAll("stop");
    stops.forEach((stop, idx) => {
      stop.setAttribute("stop-color", palette[Math.min(idx, 2)]);
    });
  };

  recolorGradient(family === "playStation" ? "white" : "shell");
  recolorGradient("edge");

  for (const rimId of ["shell-rim", "grip-shade"]) {
    const el = doc.getElementById(rimId);
    if (el) {
      el.setAttribute("fill", palette[2]);
    }
  }

  return new XMLSerializer().serializeToString(doc.documentElement);
}
