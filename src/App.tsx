import React, { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Battery,
  CheckCircle2,
  Download,
  MoreHorizontal,
  RotateCcw,
  Upload,
} from "lucide-react";
import appIconUrl from "./assets/AppIconStudio.png";
import { backend, isTauriEnv } from "./api/backend";
import { ControllerWorkspace } from "./components/ControllerWorkspace";
import { InspectorSidebar } from "./components/InspectorSidebar";
import { MappingSheet } from "./components/MappingSheet";
import { StickRoleSheet } from "./components/StickRoleSheet";
import type {
  AppStatePayload,
  ControllerControlID,
  ControllerProfile,
  StickSide,
} from "./types/models";
import { hasExplicitOverride, resolveEffectiveMapping } from "./utils/controllerLayout";

export const App: React.FC = () => {
  const [state, setState] = useState<AppStatePayload | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [selectedAppId, setSelectedAppId] = useState<string | null>(null);
  const [selectedModifier, setSelectedModifier] = useState<ControllerControlID | null>(null);
  const [editingControl, setEditingControl] = useState<ControllerControlID | null>(null);
  const [editingStick, setEditingStick] = useState<StickSide | null>(null);
  const [showProfileMenu, setShowProfileMenu] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const runAction = async (fn: () => Promise<AppStatePayload>) => {
    try {
      setErrorMsg(null);
      const next = await fn();
      setState(next);
    } catch (e) {
      setErrorMsg(String(e));
    }
  };

  useEffect(() => {
    if (!isTauriEnv()) return;
    runAction(() => backend.getAppState());

    let unlisten: (() => void) | null = null;
    backend.onTelemetry((snapshot) => {
      setState((prev) => (prev ? { ...prev, snapshot } : prev));
    }).then((fn) => {
      unlisten = fn;
    });

    const pollTimer = window.setInterval(() => {
      backend.getAppState().then(setState).catch(() => {});
    }, 1000);

    return () => {
      if (unlisten) unlisten();
      window.clearInterval(pollTimer);
    };
  }, []);

  if (!state) {
    return (
      <div
        className="app-shell"
        style={{ alignItems: "center", justifyContent: "center", gap: 12 }}
      >
        <img src={appIconUrl} alt="thumbdesk." width={56} height={56} style={{ borderRadius: 14 }} />
        <div className="brand-wordmark" style={{ fontSize: 22 }}>
          <span className="brand-wordmark-thumb">thumb</span>
          <span className="brand-wordmark-desk">desk.</span>
        </div>
      </div>
    );
  }

  const activeProfile: ControllerProfile =
    state.document.profiles.find((p) => p.id === state.document.activeProfileId) ??
    state.document.profiles[0];

  const { snapshot } = state;
  const statusColor = !state.runtimeEnabled
    ? "var(--orange)"
    : snapshot.isConnected
    ? "var(--green)"
    : "var(--text-secondary)";
  const statusLabel = !state.runtimeEnabled
    ? "Paused"
    : snapshot.isConnected
    ? "Active"
    : "Waiting";

  const handleExportProfile = async () => {
    setShowProfileMenu(false);
    try {
      const json = await backend.exportActiveProfileJson();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${activeProfile.id}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setErrorMsg(String(e));
    }
  };

  const handleImportFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    e.target.value = "";
    if (!file) return;
    const text = await file.text();
    await runAction(() => backend.importProfileJson(text));
  };

  const selectedAppName = selectedAppId
    ? activeProfile.applicationMappings.find((a) => a.bundleIdentifier === selectedAppId)
        ?.displayName ?? selectedAppId
    : null;

  return (
    <div className="app-shell">
      <input
        ref={fileInputRef}
        type="file"
        accept=".json,application/json"
        style={{ display: "none" }}
        onChange={handleImportFile}
      />

      {/* SLIM DRAGGABLE TOP BAR — LOGO #03 + #06 'thumbdesk.' LOCKUP LEFT, WINDOW CONTROLS RIGHT */}
      <div className="window-titlebar" data-tauri-drag-region>
        <div className="brand-lockup">
          <img
            src={appIconUrl}
            alt="thumbdesk."
            width={22}
            height={22}
            style={{ borderRadius: 6, filter: "drop-shadow(0 2px 8px rgba(56,189,248,0.28))" }}
          />
          <span className="brand-wordmark">
            <span className="brand-wordmark-thumb">thumb</span>
            <span className="brand-wordmark-desk">desk.</span>
          </span>
        </div>
        <div className="win-lights" aria-label="Window controls">
          <button
            type="button"
            className="win-light win-light-yellow"
            title="Minimize"
            onClick={() => isTauriEnv() && getCurrentWindow().minimize()}
          >
            <span>−</span>
          </button>
          <button
            type="button"
            className="win-light win-light-green"
            title="Maximize / Restore"
            onClick={() => isTauriEnv() && getCurrentWindow().toggleMaximize()}
          >
            <span>+</span>
          </button>
          <button
            type="button"
            className="win-light win-light-red"
            title="Close"
            onClick={() => isTauriEnv() && getCurrentWindow().close()}
          >
            <span>×</span>
          </button>
        </div>
      </div>

      {/* MAIN SPLIT — LEFT SIDEBAR & RIGHT WORKSPACE FLUSH AT TOP */}
      <div className="main-split">
        <InspectorSidebar
          cursor={activeProfile.cursor}
          hapticsPrefs={state.hapticsPreferences}
          hapticsStatus={state.hapticsStatus}
          companionMode={state.companionMode}
          companionEdge={state.companionEdge}
          companionState={state.companionState}
          companionPeers={state.companionPeers}
          onUpdateCursor={(cfg) => runAction(() => backend.updateCursorConfig(cfg))}
          onUpdateHaptics={(prefs) => runAction(() => backend.setHapticsPreferences(prefs))}
          onPreviewVibration={() => backend.previewVibration("doubleTap")}
          onApplySuggestedVibrations={() => runAction(() => backend.applySuggestedVibrations())}
          onSetCompanionMode={(mode) => runAction(() => backend.setCompanionMode(mode))}
          onSetCompanionEdge={(edge) => runAction(() => backend.setCompanionEdge(edge))}
          onConnectCompanion={(peer) => runAction(() => backend.connectCompanion(peer))}
          onDisconnectCompanion={() => runAction(() => backend.disconnectCompanion())}
          onForceHandoff={() => runAction(() => backend.forceCompanionHandoff())}
          onReturnLocal={() => runAction(() => backend.returnCompanionLocal())}
        />

        <ControllerWorkspace
          profile={activeProfile}
          profiles={state.document.profiles}
          activeProfileId={state.document.activeProfileId}
          runtimeEnabled={state.runtimeEnabled}
          statusLabel={statusLabel}
          statusColor={statusColor}
          snapshot={snapshot}
          diagnostics={state.diagnostics}
          selectedAppId={selectedAppId}
          selectedModifier={selectedModifier}
          onToggleRuntimeEnabled={() =>
            runAction(() => backend.setRuntimeEnabled(!state.runtimeEnabled))
          }
          onSelectProfile={(id) => runAction(() => backend.selectProfile(id))}
          onSelectAppId={setSelectedAppId}
          onSelectModifier={setSelectedModifier}
          onSelectControl={setEditingControl}
          onSelectStickRole={setEditingStick}
          onAddModifierLayer={(c) => runAction(() => backend.addModifierLayer(c))}
          onRemoveModifierLayer={(c) => runAction(() => backend.removeModifierLayer(c))}
          onLoadRunningApps={() => backend.listRunningApps()}
          onAddAppScope={(id, name) => runAction(() => backend.addAppScope(id, name))}
          onInstallCodexStarter={() => runAction(() => backend.installCodexStarter())}
          onResetAppScope={(id) => runAction(() => backend.resetAppScope(id))}
          onRemoveAppScope={(id) => runAction(() => backend.removeAppScope(id))}
          onTriggerSweep={() => backend.triggerDiagnosticSweep()}
          onTriggerLeftClick={() => backend.triggerDiagnosticLeftClick()}
        />
      </div>

      {/* FOOTER BAR */}
      <footer className="footer-bar">
        <img src={appIconUrl} alt="ThumbDesk" width={16} height={16} style={{ borderRadius: 4 }} />
        <span style={{ display: "inline-flex", alignItems: "center", gap: 5, fontSize: 11.5, color: "var(--green)" }}>
          <CheckCircle2 size={13} /> {snapshot.name ?? "Connect an Xbox or PlayStation controller"}
          {snapshot.transport ? ` · ${snapshot.transport}` : ""}
        </span>
        {snapshot.batteryLevel !== null && (
          <span style={{ display: "inline-flex", alignItems: "center", gap: 4, fontSize: 11.5, color: "var(--text-secondary)" }}>
            · <Battery size={12} /> {Math.round(snapshot.batteryLevel * 100)}%
          </span>
        )}
        <div style={{ flex: 1 }} />
        {errorMsg && (
          <span style={{ fontSize: 12, color: "var(--red)" }}>{errorMsg}</span>
        )}
        <div style={{ position: "relative" }}>
          <button
            type="button"
            className="btn btn-sm"
            onClick={() => setShowProfileMenu((v) => !v)}
          >
            <MoreHorizontal size={14} /> Manage Profile
          </button>
          {showProfileMenu && (
            <div
              className="modal-card"
              style={{
                position: "absolute",
                bottom: 36,
                right: 0,
                width: 210,
                padding: 6,
                zIndex: 50,
              }}
            >
              <button
                type="button"
                className="btn btn-sm"
                style={{ width: "100%", justifyContent: "flex-start", background: "transparent", borderColor: "transparent" }}
                onClick={() => {
                  setShowProfileMenu(false);
                  fileInputRef.current?.click();
                }}
              >
                <Upload size={13} /> Import Profile…
              </button>
              <button
                type="button"
                className="btn btn-sm"
                style={{ width: "100%", justifyContent: "flex-start", background: "transparent", borderColor: "transparent" }}
                onClick={handleExportProfile}
              >
                <Download size={13} /> Export Profile…
              </button>
              <div className="divider" style={{ margin: "6px 0" }} />
              <button
                type="button"
                className="btn btn-sm btn-danger"
                style={{ width: "100%", justifyContent: "flex-start" }}
                onClick={() => {
                  setShowProfileMenu(false);
                  runAction(() => backend.resetActiveProfileToDefaults());
                }}
              >
                <RotateCcw size={13} /> Reset to Defaults
              </button>
            </div>
          )}
        </div>
      </footer>

      {/* MODALS */}
      {editingControl && (
        <MappingSheet
          control={editingControl}
          family={snapshot.family}
          modifierControl={selectedModifier}
          appName={selectedAppName}
          initialMapping={resolveEffectiveMapping(
            activeProfile,
            editingControl,
            selectedModifier,
            selectedAppId
          )}
          hasOverride={hasExplicitOverride(
            activeProfile,
            editingControl,
            selectedModifier,
            selectedAppId
          )}
          hapticsPrefs={state.hapticsPreferences}
          hapticsStatus={state.hapticsStatus}
          onPreviewVibration={(p) => backend.previewVibration(p)}
          onEnableHaptics={() =>
            runAction(() =>
              backend.setHapticsPreferences({ ...state.hapticsPreferences, enabled: true })
            )
          }
          onClearOverride={() => {
            const ctrl = editingControl;
            setEditingControl(null);
            runAction(() =>
              backend.clearMappingOverride(ctrl, selectedModifier, selectedAppId)
            );
          }}
          onSave={(mapping) => {
            const ctrl = editingControl;
            setEditingControl(null);
            runAction(() =>
              backend.saveMapping(ctrl, selectedModifier, selectedAppId, mapping)
            );
          }}
          onClose={() => setEditingControl(null)}
        />
      )}

      {editingStick && (
        <StickRoleSheet
          side={editingStick}
          cursor={activeProfile.cursor}
          snapshot={snapshot}
          onSave={(role) => {
            const side = editingStick;
            setEditingStick(null);
            runAction(() => backend.assignStickRole(side, role));
          }}
          onClose={() => setEditingStick(null)}
        />
      )}
    </div>
  );
};
