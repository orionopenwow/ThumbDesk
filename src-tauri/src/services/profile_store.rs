use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::control::ControllerControlID;
use crate::models::profile::{
    ActionType, ApplicationMappingOverrides, ControllerActionMapping, ControllerProfile,
    ProfileDocument, TriggerMode,
};
use crate::models::shortcut::ShortcutDescriptor;

pub const PROFILE_FILE_NAME: &str = "profiles.json";

#[derive(Debug, Clone)]
pub struct ProfileStore {
    base_dir: PathBuf,
}

impl ProfileStore {
    pub fn default_store() -> Self {
        let dir = dirs::config_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("Vibe Controller");
        Self { base_dir: dir }
    }

    pub fn with_directory(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn file_path(&self) -> PathBuf {
        self.base_dir.join(PROFILE_FILE_NAME)
    }

    pub fn load_or_create(&self) -> Result<ProfileDocument, String> {
        let path = self.file_path();
        if !path.exists() {
            let doc = ProfileDocument::default_document();
            self.save(&doc)?;
            return Ok(doc);
        }
        let raw = fs::read_to_string(&path).map_err(|e| format!("Failed to read profiles: {}", e))?;
        let doc: ProfileDocument =
            serde_json::from_str(&raw).map_err(|e| format!("Corrupt profiles.json: {}", e))?;
        Ok(Self::normalize(doc))
    }

    pub fn save(&self, document: &ProfileDocument) -> Result<(), String> {
        let normalized = Self::normalize(document.clone());
        fs::create_dir_all(&self.base_dir)
            .map_err(|e| format!("Failed to create profile dir: {}", e))?;
        let json = serde_json::to_string_pretty(&normalized)
            .map_err(|e| format!("Failed to serialize profiles: {}", e))?;
        let path = self.file_path();
        let tmp = self.base_dir.join(format!("{}.tmp", PROFILE_FILE_NAME));
        fs::write(&tmp, json).map_err(|e| format!("Failed to write temp profiles: {}", e))?;
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
        fs::rename(&tmp, &path).map_err(|e| format!("Failed to commit profiles: {}", e))
    }

    /// Exact v3 / v4 / v5 migration & normalization rules from Vibe Controller.
    pub fn normalize(mut doc: ProfileDocument) -> ProfileDocument {
        let original_version = doc.version;

        if original_version < 3 {
            if let Some(gabes) = doc.profiles.iter_mut().find(|p| p.id == "gabes-defaults") {
                if !gabes
                    .application_mappings
                    .iter()
                    .any(|a| a.bundle_identifier == ApplicationMappingOverrides::CODEX_BUNDLE_IDENTIFIER)
                {
                    gabes
                        .application_mappings
                        .push(ControllerProfile::codex_starter_application_mappings());
                }
            }
        }

        if original_version < 4 {
            for profile in &mut doc.profiles {
                for app in &mut profile.application_mappings {
                    if app.bundle_identifier == ApplicationMappingOverrides::CODEX_BUNDLE_IDENTIFIER
                    {
                        if let Some(rt) = app.mappings.get(&ControllerControlID::RightTrigger) {
                            if rt.action_type == ActionType::KeyboardShortcut
                                && rt.trigger_mode == TriggerMode::HoldWhilePressed
                                && rt
                                    .shortcut
                                    .as_ref()
                                    .map(|s| s.key_code == 61 && s.modifiers.is_empty())
                                    .unwrap_or(false)
                            {
                                app.mappings.remove(&ControllerControlID::RightTrigger);
                            }
                        }
                    }
                }
            }
        }

        if original_version < 5 {
            if let Some(gabes) = doc.profiles.iter_mut().find(|p| p.id == "gabes-defaults") {
                if let Some(codex) = gabes
                    .application_mappings
                    .iter_mut()
                    .find(|a| a.bundle_identifier == ApplicationMappingOverrides::CODEX_BUNDLE_IDENTIFIER)
                {
                    *codex = ControllerProfile::codex_starter_application_mappings();
                }
            }
        }

        doc.version = doc.version.max(ProfileDocument::CURRENT_VERSION);
        if doc.profiles.is_empty() {
            doc.profiles = ProfileDocument::default_document().profiles;
        }
        if !doc.profiles.iter().any(|p| p.id == doc.active_profile_id) {
            doc.active_profile_id = doc.profiles[0].id.clone();
        }
        doc
    }

    pub fn export_profile(profile: &ControllerProfile, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(profile)
            .map_err(|e| format!("Failed to serialize profile: {}", e))?;
        fs::write(path, json).map_err(|e| format!("Failed to write export file: {}", e))
    }

    /// Imports either a single ControllerProfile or a ProfileDocument JSON file.
    /// Automatically detects macOS GAPE-style profiles and converts shortcuts to Windows VK.
    pub fn import_profile_str(raw: &str, into: &ProfileDocument) -> Result<ProfileDocument, String> {
        let is_macos_json = raw.contains("\"command\"") || raw.contains("\"option\"");
        if let Ok(mut single) = serde_json::from_str::<ControllerProfile>(raw) {
            if is_macos_json {
                convert_macos_profile_shortcuts(&mut single);
            }
            return Ok(Self::merge_profile(single, into));
        }
        let mut doc: ProfileDocument = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid profile JSON: {}", e))?;
        if is_macos_json {
            for p in &mut doc.profiles {
                convert_macos_profile_shortcuts(p);
            }
        }
        let mut merged = Self::normalize(into.clone());
        for p in doc.profiles {
            merged = Self::merge_profile(p, &merged);
        }
        Ok(Self::normalize(merged))
    }

    pub fn merge_profile(
        mut profile: ControllerProfile,
        into: &ProfileDocument,
    ) -> ProfileDocument {
        let mut merged = Self::normalize(into.clone());
        let existing_ids: BTreeSet<String> = merged.profiles.iter().map(|p| p.id.clone()).collect();
        let existing_names: BTreeSet<String> =
            merged.profiles.iter().map(|p| p.name.clone()).collect();

        if existing_ids.contains(&profile.id) {
            let base = profile.id.clone();
            let mut n = 2;
            while existing_ids.contains(&format!("{}-{}", base, n)) {
                n += 1;
            }
            profile.id = format!("{}-{}", base, n);
        }
        if existing_names.contains(&profile.name) {
            let base = profile.name.clone();
            let mut n = 2;
            while existing_names.contains(&format!("{} {}", base, n)) {
                n += 1;
            }
            profile.name = format!("{} {}", base, n);
        }

        merged.active_profile_id = profile.id.clone();
        merged.profiles.push(profile);
        merged
    }
}

fn convert_mapping(mapping: &mut ControllerActionMapping) {
    if mapping.action_type == ActionType::KeyboardShortcut {
        if let Some(s) = &mapping.shortcut {
            mapping.shortcut = Some(ShortcutDescriptor::from_macos_shortcut(
                s.key_code,
                &s.modifiers,
            ));
        }
    }
}

fn convert_macos_profile_shortcuts(profile: &mut ControllerProfile) {
    for m in profile.mappings.values_mut() {
        convert_mapping(m);
    }
    for layer in &mut profile.modifier_layers {
        for m in layer.mappings.values_mut() {
            convert_mapping(m);
        }
    }
    for app in &mut profile.application_mappings {
        for m in app.mappings.values_mut() {
            convert_mapping(m);
        }
        for layer in &mut app.modifier_layers {
            for m in layer.mappings.values_mut() {
                convert_mapping(m);
            }
        }
    }
}
