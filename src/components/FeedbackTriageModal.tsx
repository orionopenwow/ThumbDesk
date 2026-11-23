import React, { useEffect, useState } from "react";
import {
  Bug,
  Check,
  CheckCircle2,
  Copy,
  ExternalLink,
  Layers,
  Sparkles,
  Trash2,
  X,
} from "lucide-react";
import { backend } from "../api/backend";
import type { TriageItem, TriageKind } from "../types/models";

interface FeedbackTriageModalProps {
  controllerName: string | null;
  version: string;
  onClose: () => void;
}

export const FeedbackTriageModal: React.FC<FeedbackTriageModalProps> = ({
  controllerName,
  version,
  onClose,
}) => {
  const [tab, setTab] = useState<"report" | "batch">("report");
  const [kind, setKind] = useState<TriageKind>("bug");
  const [title, setTitle] = useState("");
  const [details, setDetails] = useState("");
  const [items, setItems] = useState<TriageItem[]>([]);
  const [copied, setCopied] = useState(false);
  const [savedNotice, setSavedNotice] = useState(false);

  useEffect(() => {
    backend.listTriageItems().then(setItems).catch(() => {});
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    const next = await backend.submitTriageItem(kind, title.trim(), details.trim());
    setItems(next);
    setTitle("");
    setDetails("");
    setSavedNotice(true);
    window.setTimeout(() => setSavedNotice(false), 1800);
    setTab("batch");
  };

  const handleOpenGitHubIssue = () => {
    const label = kind === "bug" ? "bug,ade-batch" : "enhancement,ade-batch";
    const body = `${details.trim() || "Describe steps or idea here."}\n\n---\n**Auto-attached Telemetry**\n- Version: v${version}\n- Controller: ${controllerName ?? "None detected"}`;
    const url = `https://github.com/orionopenwow/ThumbDesk/issues/new?labels=${encodeURIComponent(
      label
    )}&title=${encodeURIComponent(title.trim() || `[${kind.toUpperCase()}] `)}&body=${encodeURIComponent(
      body
    )}`;
    window.open(url, "_blank", "noopener");
  };

  const handleCopyBatchPrompt = async () => {
    const prompt = await backend.exportAdeBatchPrompt();
    try {
      await navigator.clipboard.writeText(prompt);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2200);
    } catch {
      // Fallback ignored in headless environments
    }
  };

  const approvedCount = items.filter((i) => i.status === "approved").length;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal-card"
        style={{ width: 600, maxHeight: "86vh" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ padding: "18px 22px 14px", borderBottom: "1px solid var(--border-subtle)" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <div
              style={{
                width: 40,
                height: 40,
                borderRadius: 11,
                background: "rgba(56, 189, 248, 0.12)",
                border: "1px solid rgba(125, 211, 252, 0.3)",
                color: "#7dd3fc",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Bug size={19} />
            </div>
            <div style={{ flex: 1 }}>
              <div className="eyebrow">ADE TRIAGE &amp; BATCH AUTO-FIX</div>
              <h2 style={{ margin: "2px 0 0", fontSize: 17, fontWeight: 600 }}>
                Bug Report &amp; Feature Request
              </h2>
            </div>
            <button type="button" className="btn btn-sm" onClick={onClose} aria-label="Close">
              <X size={15} />
            </button>
          </div>

          <div className="segmented" style={{ marginTop: 12 }}>
            <button
              type="button"
              className={`segmented-btn ${tab === "report" ? "active" : ""}`}
              onClick={() => setTab("report")}
            >
              <Bug size={12} /> New Report
            </button>
            <button
              type="button"
              className={`segmented-btn ${tab === "batch" ? "active" : ""}`}
              onClick={() => setTab("batch")}
            >
              <Layers size={12} /> ADE Batch Queue ({items.length})
            </button>
          </div>
        </div>

        <div style={{ padding: 20, overflowY: "auto", display: "flex", flexDirection: "column", gap: 14 }}>
          {tab === "report" ? (
            <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              <div className="segmented" style={{ alignSelf: "flex-start" }}>
                <button
                  type="button"
                  className={`segmented-btn ${kind === "bug" ? "active" : ""}`}
                  onClick={() => setKind("bug")}
                >
                  <Bug size={12} /> Bug Report
                </button>
                <button
                  type="button"
                  className={`segmented-btn ${kind === "feature" ? "active" : ""}`}
                  onClick={() => setKind("feature")}
                >
                  <Sparkles size={12} /> Feature Request
                </button>
              </div>

              <input
                type="text"
                required
                placeholder={
                  kind === "bug"
                    ? "Short summary (e.g. Magic Select snaps too far on dual monitors)"
                    : "Feature idea (e.g. Add per-game deadzone override)"
                }
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                style={{
                  fontSize: 13,
                  padding: "9px 11px",
                  borderRadius: 9,
                  background: "rgba(0,0,0,0.3)",
                  border: "1px solid var(--border-subtle)",
                  color: "var(--text-primary)",
                  outline: "none",
                }}
              />

              <textarea
                rows={4}
                placeholder="Steps to reproduce or expected behavior (hardware & version auto-attached)…"
                value={details}
                onChange={(e) => setDetails(e.target.value)}
                style={{
                  fontSize: 12.5,
                  padding: "9px 11px",
                  borderRadius: 9,
                  background: "rgba(0,0,0,0.3)",
                  border: "1px solid var(--border-subtle)",
                  color: "var(--text-primary)",
                  outline: "none",
                  resize: "vertical",
                  fontFamily: "inherit",
                }}
              />

              <div style={{ fontSize: 11, fontFamily: "var(--font-mono)", color: "var(--text-tertiary)" }}>
                AUTO-ATTACHED: v{version} · {controllerName ?? "No controller"} · Target: ADE Batch Queue
              </div>

              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 8, marginTop: 4 }}>
                <button type="button" className="btn btn-sm" onClick={handleOpenGitHubIssue}>
                  <ExternalLink size={12} /> Also File GitHub Issue
                </button>
                <button type="submit" className="btn btn-primary" disabled={!title.trim()}>
                  Queue for ADE Batch
                </button>
              </div>
            </form>
          ) : (
            <>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 10 }}>
                <div style={{ fontSize: 12, color: "var(--text-secondary)" }}>
                  {savedNotice ? (
                    <span style={{ color: "var(--green)" }}>Queued for ADE batch review!</span>
                  ) : (
                    <span>
                      Approve items below, then copy the structured batch prompt into your ADE to auto-fix together for approval.
                    </span>
                  )}
                </div>
                <button
                  type="button"
                  className="btn btn-primary btn-sm"
                  disabled={items.length === 0}
                  onClick={handleCopyBatchPrompt}
                  style={{ flexShrink: 0 }}
                >
                  {copied ? <Check size={12} /> : <Copy size={12} />}
                  {copied
                    ? "Copied ADE Prompt"
                    : `Copy ADE Batch (${approvedCount > 0 ? `${approvedCount} approved` : items.length})`}
                </button>
              </div>

              {items.length === 0 ? (
                <div className="product-panel" style={{ padding: 20, textAlign: "center", fontSize: 12.5, color: "var(--text-tertiary)" }}>
                  No queued bug reports or feature requests yet.
                </div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                  {items.map((item) => {
                    const isApproved = item.status === "approved";
                    return (
                      <div
                        key={item.id}
                        className="product-panel"
                        style={{
                          padding: "10px 12px",
                          display: "flex",
                          alignItems: "flex-start",
                          gap: 10,
                          borderColor: isApproved ? "rgba(52, 211, 153, 0.35)" : undefined,
                        }}
                      >
                        <button
                          type="button"
                          className="icon-ghost-btn"
                          title={isApproved ? "Approved for ADE batch (click to unapprove)" : "Approve for ADE batch"}
                          onClick={async () => {
                            const next = await backend.setTriageStatus(
                              item.id,
                              isApproved ? "queued" : "approved"
                            );
                            setItems(next);
                          }}
                          style={{ color: isApproved ? "var(--green)" : "var(--text-tertiary)", marginTop: 2 }}
                        >
                          <CheckCircle2 size={15} />
                        </button>
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <div style={{ display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" }}>
                            <span
                              style={{
                                fontSize: 9.5,
                                fontFamily: "var(--font-mono)",
                                padding: "1px 6px",
                                borderRadius: 4,
                                background:
                                  item.kind === "bug"
                                    ? "rgba(239, 68, 68, 0.16)"
                                    : "rgba(56, 189, 248, 0.16)",
                                color: item.kind === "bug" ? "#fca5a5" : "#7dd3fc",
                              }}
                            >
                              {item.kind.toUpperCase()}
                            </span>
                            <span style={{ fontSize: 13, fontWeight: 600 }}>{item.title}</span>
                          </div>
                          {item.details && (
                            <div style={{ fontSize: 11.5, color: "var(--text-secondary)", marginTop: 3 }}>
                              {item.details}
                            </div>
                          )}
                        </div>
                        <button
                          type="button"
                          className="icon-ghost-btn"
                          title="Delete item"
                          onClick={async () => {
                            const next = await backend.deleteTriageItem(item.id);
                            setItems(next);
                          }}
                        >
                          <Trash2 size={13} />
                        </button>
                      </div>
                    );
                  })}
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};
