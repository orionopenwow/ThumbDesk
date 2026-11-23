import React, { useState } from "react";
import {
  ArrowRight,
  CheckCircle2,
  Download,
  RefreshCw,
  RotateCw,
  ShieldCheck,
  Sparkles,
  X,
} from "lucide-react";
import appIconUrl from "../assets/AppIconStudio.png";
import type { UpdateStatus } from "../types/models";

interface UpdateFooterPillProps {
  status: UpdateStatus;
  onCheck: () => void;
  onOpenInstaller: () => void;
}

export const UpdateFooterPill: React.FC<UpdateFooterPillProps> = ({
  status,
  onCheck,
  onOpenInstaller,
}) => {
  if (status.phase === "readyToRestart") {
    return (
      <button
        type="button"
        className="btn btn-primary btn-sm"
        style={{ gap: 6, borderRadius: 999, padding: "4px 12px" }}
        onClick={onOpenInstaller}
      >
        <RotateCw size={12} />
        Restart to Update · v{status.latestVersion}
      </button>
    );
  }

  if (status.phase === "preloading" || status.phase === "verifying") {
    return (
      <button
        type="button"
        className="btn btn-sm"
        style={{
          gap: 7,
          borderRadius: 999,
          borderColor: "rgba(96, 165, 250, 0.45)",
          color: "#93c5fd",
        }}
        onClick={onOpenInstaller}
      >
        <Download size={12} />
        {status.phase === "verifying"
          ? "Verifying SHA-256…"
          : `Preloading v${status.latestVersion} (${status.percent}%)`}
      </button>
    );
  }

  if (status.phase === "checking" || status.phase === "applying") {
    return (
      <span
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 6,
          fontSize: 11.5,
          color: "#93c5fd",
          padding: "0 6px",
        }}
      >
        <RefreshCw size={12} className="spin" />
        {status.phase === "applying" ? "Restarting…" : "Checking updates…"}
      </span>
    );
  }

  return (
    <button
      type="button"
      className="btn btn-sm"
      style={{
        borderRadius: 999,
        fontSize: 11.5,
        color: status.phase === "error" ? "#fca5a5" : "var(--text-secondary)",
      }}
      onClick={onCheck}
      title={status.message}
    >
      {status.phase === "upToDate" ? (
        <>
          <CheckCircle2 size={12} color="var(--green)" /> v{status.currentVersion} · Up to date
        </>
      ) : (
        <>
          <RefreshCw size={11.5} /> v{status.currentVersion} · Check updates
        </>
      )}
    </button>
  );
};

interface CustomInstallerModalProps {
  status: UpdateStatus;
  onRestartToUpdate: () => Promise<void>;
  onClose: () => void;
}

function formatMB(bytes: number): string {
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export const CustomInstallerModal: React.FC<CustomInstallerModalProps> = ({
  status,
  onRestartToUpdate,
  onClose,
}) => {
  const [optimisticApplying, setOptimisticApplying] = useState(false);

  const isPreloaded =
    status.phase === "readyToRestart" || status.phase === "applying" || optimisticApplying;
  const isVerified = Boolean(status.verifiedSha256) || isPreloaded;
  const isApplying = status.phase === "applying" || optimisticApplying;

  const progressPct = isPreloaded ? 100 : Math.max(8, status.percent);

  const handleInstallNow = async () => {
    setOptimisticApplying(true);
    try {
      await onRestartToUpdate();
    } catch {
      setOptimisticApplying(false);
    }
  };

  return (
    <div className="modal-backdrop" onClick={isApplying ? undefined : onClose}>
      <div
        className="modal-card"
        style={{ width: 520, padding: 24, gap: 18 }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Top Brand + Version Pill */}
        <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
          <img
            src={appIconUrl}
            alt="thumbdesk."
            width={48}
            height={48}
            style={{
              borderRadius: 12,
              filter: "drop-shadow(0 6px 16px rgba(56,189,248,0.32))",
            }}
          />
          <div style={{ flex: 1 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <span className="brand-wordmark" style={{ fontSize: 18 }}>
                <span className="brand-wordmark-thumb">thumb</span>
                <span className="brand-wordmark-desk">desk.</span>
              </span>
              <span
                className="status-badge"
                style={{
                  padding: "2px 9px",
                  fontSize: 11,
                  background: "rgba(59,130,246,0.16)",
                  borderColor: "rgba(96,165,250,0.4)",
                  color: "#93c5fd",
                }}
              >
                <Sparkles size={10} /> Installer
              </span>
            </div>
            <div
              style={{
                marginTop: 4,
                display: "inline-flex",
                alignItems: "center",
                gap: 6,
                fontSize: 12,
                fontFamily: "ui-monospace, Consolas, monospace",
                color: "var(--text-secondary)",
              }}
            >
              <span>v{status.currentVersion}</span>
              <ArrowRight size={12} color="#60a5fa" />
              <span style={{ color: "#7dd3fc", fontWeight: 600 }}>
                v{status.latestVersion ?? status.currentVersion}
              </span>
            </div>
          </div>
          {!isApplying && (
            <button type="button" className="btn btn-sm" onClick={onClose} aria-label="Close">
              <X size={15} />
            </button>
          )}
        </div>

        {/* Animated Progress Bar */}
        <div style={{ display: "flex", flexDirection: "column", gap: 7 }}>
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 12 }}>
            <span style={{ fontWeight: 500, color: "#e2e8f0" }}>
              {isApplying
                ? "Swapping verified binary & relaunching…"
                : status.message}
            </span>
            <span style={{ fontFamily: "ui-monospace, Consolas, monospace", color: "#93c5fd" }}>
              {status.totalBytes > 0
                ? `${formatMB(status.downloadedBytes)} / ${formatMB(status.totalBytes)}`
                : `${progressPct}%`}
            </span>
          </div>
          <div
            style={{
              height: 7,
              borderRadius: 999,
              background: "rgba(255,255,255,0.08)",
              overflow: "hidden",
            }}
          >
            <div
              style={{
                width: `${progressPct}%`,
                height: "100%",
                borderRadius: 999,
                background: "linear-gradient(90deg, #38bdf8 0%, #60a5fa 55%, #818cf8 100%)",
                boxShadow: "0 0 14px rgba(56, 189, 248, 0.55)",
                transition: "width 240ms ease-out",
              }}
            />
          </div>
        </div>

        {/* 4-Step Live Installer Checklist */}
        <div className="product-panel" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 9 }}>
          {[
            { label: "Check GitHub Releases for newer version", done: true },
            {
              label: "Preload standalone ThumbDesk.exe in background",
              done: isPreloaded,
            },
            {
              label: status.verifiedSha256
                ? `SHA-256 verified (${status.verifiedSha256.slice(0, 12)}…)`
                : "Verify SHA-256 cryptographic checksum",
              done: isVerified,
            },
            {
              label: "Atomic binary swap, cache refresh & instant relaunch",
              done: isApplying,
            },
          ].map((step) => (
            <div
              key={step.label}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 9,
                fontSize: 12.5,
                color: step.done ? "#f1f5f9" : "var(--text-secondary)",
              }}
            >
              {step.done ? (
                <CheckCircle2 size={14} color="var(--green)" />
              ) : (
                <ShieldCheck size={14} color="var(--text-tertiary)" />
              )}
              <span>{step.label}</span>
            </div>
          ))}
        </div>

        {/* Optional Release Notes */}
        {status.releaseNotes && (
          <div
            style={{
              maxHeight: 110,
              overflowY: "auto",
              padding: "10px 12px",
              borderRadius: 10,
              background: "rgba(0,0,0,0.24)",
              border: "1px solid var(--border-subtle)",
              fontSize: 11.5,
              color: "var(--text-secondary)",
              whiteSpace: "pre-wrap",
            }}
          >
            {status.releaseNotes.trim()}
          </div>
        )}

        {/* Footer Actions */}
        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8 }}>
          {!isApplying && (
            <button type="button" className="btn" onClick={onClose}>
              Later
            </button>
          )}
          <button
            type="button"
            className="btn btn-primary"
            disabled={!isPreloaded || isApplying}
            onClick={handleInstallNow}
          >
            <RotateCw size={13} className={isApplying ? "spin" : ""} />
            {isApplying ? "Restarting & Installing…" : "Restart & Install Now"}
          </button>
        </div>
      </div>
    </div>
  );
};
