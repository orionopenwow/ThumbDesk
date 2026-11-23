use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub const FORK_COMMAND: &str = "forkThread";
pub const FORK_ACCELERATOR_WINDOWS: &str = "Ctrl+Alt+Shift+F";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexKeybindingProvisionResult {
    CodexConfigurationUnavailable,
    AlreadyConfigured,
    Installed,
}

pub fn ensure_fork_shortcut(home_dir: &Path) -> Result<CodexKeybindingProvisionResult, String> {
    let codex_dir = home_dir.join(".codex");
    if !codex_dir.is_dir() {
        return Ok(CodexKeybindingProvisionResult::CodexConfigurationUnavailable);
    }
    let keybindings_path = codex_dir.join("keybindings.json");
    let mut entries: Vec<Value> = if keybindings_path.exists() {
        let raw = fs::read_to_string(&keybindings_path)
            .map_err(|e| format!("Failed to read Codex keybindings: {}", e))?;
        let parsed: Value = serde_json::from_str(&raw)
            .map_err(|e| format!("Corrupt Codex keybindings.json: {}", e))?;
        let arr = parsed
            .as_array()
            .ok_or_else(|| "Codex keybindings.json root must be a JSON array".to_string())?;
        if !arr.iter().all(|v| v.is_object()) {
            return Err("Codex keybindings.json must contain JSON objects".to_string());
        }
        arr.clone()
    } else {
        Vec::new()
    };

    let already = entries.iter().any(|entry| {
        entry.get("command").and_then(|v| v.as_str()) == Some(FORK_COMMAND)
            && entry
                .get("key")
                .and_then(|v| v.as_str())
                .map(|k| k.eq_ignore_ascii_case(FORK_ACCELERATOR_WINDOWS))
                .unwrap_or(false)
    });
    if already {
        return Ok(CodexKeybindingProvisionResult::AlreadyConfigured);
    }

    entries.push(json!({
        "command": FORK_COMMAND,
        "key": FORK_ACCELERATOR_WINDOWS,
    }));

    let serialized = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("Failed to serialize Codex keybindings: {}", e))?;
    let tmp = codex_dir.join("keybindings.json.tmp");
    fs::write(&tmp, serialized)
        .map_err(|e| format!("Failed to write temp Codex keybindings: {}", e))?;
    if keybindings_path.exists() {
        let _ = fs::remove_file(&keybindings_path);
    }
    fs::rename(&tmp, &keybindings_path)
        .map_err(|e| format!("Failed to commit Codex keybindings: {}", e))?;
    Ok(CodexKeybindingProvisionResult::Installed)
}
