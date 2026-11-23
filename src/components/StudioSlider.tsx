import React from "react";

interface StudioSliderProps {
  label: string;
  valueLabel: string;
  min: number;
  max: number;
  step: number;
  value: number;
  onChange: (value: number) => void;
}

export const StudioSlider: React.FC<StudioSliderProps> = ({
  label,
  valueLabel,
  min,
  max,
  step,
  value,
  onChange,
}) => {
  const pct = Math.max(0, Math.min(100, ((value - min) / (max - min)) * 100));
  return (
    <div className="slider-row">
      <div className="slider-row-header">
        <span>{label}</span>
        <span className="slider-row-value">{valueLabel}</span>
      </div>
      <input
        type="range"
        className="studio-slider"
        min={min}
        max={max}
        step={step}
        value={value}
        style={{ ["--slider-pct" as string]: `${pct}%` }}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </div>
  );
};
