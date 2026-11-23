use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriageKind {
    Bug,
    Feature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriageStatus {
    Queued,
    Approved,
    Fixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriageItem {
    pub id: String,
    pub kind: TriageKind,
    pub title: String,
    pub details: String,
    pub status: TriageStatus,
    pub version: String,
    pub controller: Option<String>,
    pub created_at: String,
}

pub struct TriageQueueService {
    file_path: PathBuf,
    items: Mutex<Vec<TriageItem>>,
}

impl TriageQueueService {
    pub fn default_service() -> Self {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Vibe Controller");
        Self::with_file(dir.join("ade_triage_queue.json"))
    }

    pub fn with_file(file_path: PathBuf) -> Self {
        let items = fs::read_to_string(&file_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Vec<TriageItem>>(&raw).ok())
            .unwrap_or_default();
        Self {
            file_path,
            items: Mutex::new(items),
        }
    }

    fn persist_locked(&self, items: &[TriageItem]) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed creating triage dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(items)
            .map_err(|e| format!("Failed serializing triage queue: {}", e))?;
        fs::write(&self.file_path, json)
            .map_err(|e| format!("Failed writing triage queue: {}", e))
    }

    pub fn list(&self) -> Vec<TriageItem> {
        self.items.lock().unwrap().clone()
    }

    pub fn submit(
        &self,
        kind: TriageKind,
        title: String,
        details: String,
        controller: Option<String>,
    ) -> Result<Vec<TriageItem>, String> {
        let clean_title = title.trim().to_string();
        if clean_title.is_empty() {
            return Err("Title cannot be empty".into());
        }
        let item = TriageItem {
            id: Uuid::new_v4().to_string(),
            kind,
            title: clean_title,
            details: details.trim().to_string(),
            status: TriageStatus::Queued,
            version: env!("CARGO_PKG_VERSION").to_string(),
            controller,
            created_at: chrono_like_timestamp(),
        };
        let mut guard = self.items.lock().unwrap();
        guard.insert(0, item);
        self.persist_locked(&guard)?;
        Ok(guard.clone())
    }

    pub fn set_status(&self, id: &str, status: TriageStatus) -> Result<Vec<TriageItem>, String> {
        let mut guard = self.items.lock().unwrap();
        if let Some(item) = guard.iter_mut().find(|i| i.id == id) {
            item.status = status;
            self.persist_locked(&guard)?;
        }
        Ok(guard.clone())
    }

    pub fn delete(&self, id: &str) -> Result<Vec<TriageItem>, String> {
        let mut guard = self.items.lock().unwrap();
        guard.retain(|i| i.id != id);
        self.persist_locked(&guard)?;
        Ok(guard.clone())
    }

    /// Generates a structured ADE batch auto-fix prompt for all Approved (or Queued if none approved) items.
    pub fn build_ade_batch_prompt(&self) -> String {
        let guard = self.items.lock().unwrap();
        let selected: Vec<&TriageItem> = {
            let approved: Vec<&TriageItem> = guard
                .iter()
                .filter(|i| i.status == TriageStatus::Approved)
                .collect();
            if !approved.is_empty() {
                approved
            } else {
                guard
                    .iter()
                    .filter(|i| i.status == TriageStatus::Queued)
                    .collect()
            }
        };

        if selected.is_empty() {
            return "No queued or approved ThumbDesk triage items to batch-fix.".to_string();
        }

        let mut out = String::from(
            "# ThumbDesk ADE Batch Auto-Fix Request\n\n\
             Please fix/implement the following approved batch of items in `thumbdesk.`, run all Rust & Vitest tests, and present a summary diff for final approval before releasing:\n\n",
        );
        for (idx, item) in selected.iter().enumerate() {
            let tag = match item.kind {
                TriageKind::Bug => "BUG",
                TriageKind::Feature => "FEATURE",
            };
            out.push_str(&format!(
                "{}. **[{}] {}** (v{}{})\n",
                idx + 1,
                tag,
                item.title,
                item.version,
                item.controller
                    .as_ref()
                    .map(|c| format!(" · {}", c))
                    .unwrap_or_default()
            ));
            if !item.details.is_empty() {
                out.push_str(&format!("   - Details: {}\n", item.details));
            }
        }
        out
    }
}

fn chrono_like_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("ts-{}", secs)
}
