use std::os::windows::process::CommandExt;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::platform::active_app::RunningAppInfo;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub const ANTI_CHEAT_PROCESSES: &[(&str, &str)] = &[
    ("vgk.exe", "Riot Vanguard"),
    ("vgc.exe", "Riot Vanguard"),
    ("vgtray.exe", "Riot Vanguard"),
    ("easyanticheat.exe", "Easy Anti-Cheat"),
    ("easyanticheat_eos.exe", "Easy Anti-Cheat (EOS)"),
    ("beservice.exe", "BattlEye"),
    ("bedaisy.exe", "BattlEye"),
    ("faceitclient.exe", "FACEIT Anti-Cheat"),
    ("faceitservice.exe", "FACEIT Anti-Cheat"),
    ("eseaclient.exe", "ESEA Anti-Cheat"),
    ("punkbstrA.exe", "PunkBuster"),
    ("punkbstrB.exe", "PunkBuster"),
    ("cod.exe", "Call of Duty (RICOCHET)"),
    ("valorant-win64-shipping.exe", "VALORANT (Vanguard)"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameGuardPreferences {
    pub anti_cheat_auto_pause: bool,
    pub steam_game_auto_pause: bool,
}

impl Default for GameGuardPreferences {
    fn default() -> Self {
        Self {
            anti_cheat_auto_pause: true,
            steam_game_auto_pause: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameGuardStatus {
    pub is_suspended: bool,
    pub reason: Option<String>,
    pub detected_anti_cheat: Option<String>,
    pub steam_running: bool,
    pub steam_game_active: bool,
}

impl Default for GameGuardStatus {
    fn default() -> Self {
        Self {
            is_suspended: false,
            reason: None,
            detected_anti_cheat: None,
            steam_running: false,
            steam_game_active: false,
        }
    }
}

pub fn detect_anti_cheat_name(exe_name: &str) -> Option<&'static str> {
    let lower = exe_name.to_ascii_lowercase();
    ANTI_CHEAT_PROCESSES
        .iter()
        .find(|(proc_name, _)| lower == proc_name.to_ascii_lowercase())
        .map(|(_, label)| *label)
}

pub fn is_steam_game_path(exe_path: &str) -> bool {
    let lower = exe_path.to_ascii_lowercase().replace('/', "\\");
    lower.contains("\\steamapps\\common\\")
}

/// Evaluates foreground app + running apps against Anti-Cheat & Steam compliance rules.
pub fn evaluate_game_guard(
    prefs: GameGuardPreferences,
    foreground: Option<&RunningAppInfo>,
    running_apps: &[RunningAppInfo],
) -> GameGuardStatus {
    let steam_running = running_apps.iter().any(|a| {
        let id = a.bundle_identifier.to_ascii_lowercase();
        id == "steam.exe" || id == "steamwebhelper.exe"
    });

    let detected_ac = foreground
        .and_then(|fg| detect_anti_cheat_name(&fg.bundle_identifier))
        .or_else(|| {
            running_apps
                .iter()
                .find_map(|a| detect_anti_cheat_name(&a.bundle_identifier))
        })
        .map(String::from);

    let steam_game_active = foreground
        .map(|fg| is_steam_game_path(&fg.executable_path))
        .unwrap_or(false);

    if prefs.anti_cheat_auto_pause {
        if let Some(ac) = &detected_ac {
            return GameGuardStatus {
                is_suspended: true,
                reason: Some(format!("Auto-paused for anti-cheat compliance ({})", ac)),
                detected_anti_cheat: detected_ac,
                steam_running,
                steam_game_active,
            };
        }
    }

    if prefs.steam_game_auto_pause && steam_game_active {
        let game_name = foreground
            .map(|fg| fg.display_name.clone())
            .unwrap_or_else(|| "Steam Game".to_string());
        return GameGuardStatus {
            is_suspended: true,
            reason: Some(format!(
                "Auto-paused for Steam game ({}) — Steam Input active",
                game_name
            )),
            detected_anti_cheat: detected_ac,
            steam_running: true,
            steam_game_active: true,
        };
    }

    GameGuardStatus {
        is_suspended: false,
        reason: None,
        detected_anti_cheat: detected_ac,
        steam_running,
        steam_game_active,
    }
}

pub fn open_steam_uri(uri: &str) -> Result<(), String> {
    Command::new("cmd")
        .args(["/C", "start", "", uri])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Failed to open Steam URI: {}", e))?;
    Ok(())
}
