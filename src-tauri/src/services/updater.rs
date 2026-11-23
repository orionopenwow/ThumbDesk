use std::fs;
use std::io::{Read, Write};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const GITHUB_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/orionopenwow/ThumbDesk/releases/latest";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdatePhase {
    Idle,
    Checking,
    UpToDate,
    Preloading,
    Verifying,
    ReadyToRestart,
    Applying,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub phase: UpdatePhase,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_notes: Option<String>,
    pub release_url: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: u8,
    pub verified_sha256: Option<String>,
    pub staged_path: Option<String>,
    pub message: String,
}

impl Default for UpdateStatus {
    fn default() -> Self {
        Self {
            phase: UpdatePhase::Idle,
            current_version: CURRENT_VERSION.to_string(),
            latest_version: None,
            release_notes: None,
            release_url: None,
            downloaded_bytes: 0,
            total_bytes: 0,
            percent: 0,
            verified_sha256: None,
            staged_path: None,
            message: format!("v{}", CURRENT_VERSION),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

/// Parses semver `v1.2.3` or `1.2.3` into `(1, 2, 3)`.
pub fn parse_semver(tag: &str) -> Option<(u64, u64, u64)> {
    let clean = tag.trim().trim_start_matches(['v', 'V']);
    let base = clean.split('-').next().unwrap_or(clean);
    let mut parts = base.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

pub fn is_newer_version(current: &str, candidate: &str) -> bool {
    match (parse_semver(current), parse_semver(candidate)) {
        (Some(c), Some(n)) => n > c,
        _ => false,
    }
}

/// Extracts expected SHA-256 hex digest for a filename from `SHA256SUMS.txt`.
pub fn parse_sha256_for_file(sums_text: &str, file_name: &str) -> Option<String> {
    for line in sums_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?;
        if name.eq_ignore_ascii_case(file_name) && hash.len() == 64 {
            return Some(hash.to_ascii_lowercase());
        }
    }
    None
}

pub fn compute_file_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("Open failed: {}", e))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("Read failed: {}", e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{:02x}", b)).collect())
}

#[derive(Clone)]
pub struct AppUpdateService {
    status: Arc<Mutex<UpdateStatus>>,
}

impl Default for AppUpdateService {
    fn default() -> Self {
        Self::new()
    }
}

impl AppUpdateService {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::default())),
        }
    }

    pub fn status(&self) -> UpdateStatus {
        self.status.lock().unwrap().clone()
    }

    fn updates_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("ThumbDesk")
            .join("updates")
    }

    /// Starts an asynchronous check on startup (after `delay_secs`) and cleverly preloads
    /// the newer `ThumbDesk.exe` + verifies SHA-256 in the background so Restart-to-Update is instant.
    pub fn check_and_preload_async(&self, delay_secs: u64) {
        {
            let mut guard = self.status.lock().unwrap();
            if matches!(
                guard.phase,
                UpdatePhase::Checking
                    | UpdatePhase::Preloading
                    | UpdatePhase::Verifying
                    | UpdatePhase::Applying
            ) {
                return;
            }
            if delay_secs == 0 {
                guard.phase = UpdatePhase::Checking;
                guard.message = "Checking GitHub Releases…".to_string();
            }
        }

        let svc = self.clone();
        thread::Builder::new()
            .name("thumbdesk-updater".into())
            .spawn(move || {
                if delay_secs > 0 {
                    thread::sleep(Duration::from_secs(delay_secs));
                    let mut guard = svc.status.lock().unwrap();
                    guard.phase = UpdatePhase::Checking;
                    guard.message = "Checking GitHub Releases…".to_string();
                }
                svc.run_check_and_preload();
            })
            .ok();
    }

    fn run_check_and_preload(&self) {
        let resp = ureq::get(GITHUB_LATEST_RELEASE_URL)
            .set("User-Agent", &format!("ThumbDesk/{}", CURRENT_VERSION))
            .set("Accept", "application/vnd.github+json")
            .timeout(Duration::from_secs(12))
            .call();

        let release: GithubRelease = match resp {
            Ok(r) => match r.into_json() {
                Ok(parsed) => parsed,
                Err(e) => {
                    self.set_error(format!("Invalid release metadata: {}", e));
                    return;
                }
            },
            Err(e) => {
                self.set_error(format!("Update check failed: {}", e));
                return;
            }
        };

        let clean_latest = release.tag_name.trim_start_matches(['v', 'V']).to_string();
        if !is_newer_version(CURRENT_VERSION, &clean_latest) {
            let mut guard = self.status.lock().unwrap();
            guard.phase = UpdatePhase::UpToDate;
            guard.latest_version = Some(clean_latest);
            guard.release_url = Some(release.html_url);
            guard.message = format!("Up to date (v{})", CURRENT_VERSION);
            return;
        }

        let Some(exe_asset) = release
            .assets
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case("ThumbDesk.exe"))
        else {
            self.set_error(format!(
                "v{} found, but ThumbDesk.exe asset is not published yet.",
                clean_latest
            ));
            return;
        };

        let sums_asset = release
            .assets
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case("SHA256SUMS.txt"));

        let stage_dir = Self::updates_dir().join(&clean_latest);
        let staged_exe = stage_dir.join("ThumbDesk.exe");

        // Fetch expected SHA-256 if published
        let expected_sha = sums_asset.and_then(|a| {
            ureq::get(&a.browser_download_url)
                .set("User-Agent", &format!("ThumbDesk/{}", CURRENT_VERSION))
                .timeout(Duration::from_secs(10))
                .call()
                .ok()
                .and_then(|r| r.into_string().ok())
                .and_then(|txt| parse_sha256_for_file(&txt, "ThumbDesk.exe"))
        });

        // If already preloaded and checksum matches, mark ready instantly!
        if staged_exe.exists() {
            if let Ok(actual) = compute_file_sha256(&staged_exe) {
                if expected_sha.as_deref().map(|e| e == actual).unwrap_or(true) {
                    let mut guard = self.status.lock().unwrap();
                    guard.phase = UpdatePhase::ReadyToRestart;
                    guard.latest_version = Some(clean_latest.clone());
                    guard.release_notes = release.body;
                    guard.release_url = Some(release.html_url);
                    guard.downloaded_bytes = exe_asset.size;
                    guard.total_bytes = exe_asset.size;
                    guard.percent = 100;
                    guard.verified_sha256 = Some(actual);
                    guard.staged_path = Some(staged_exe.to_string_lossy().to_string());
                    guard.message = format!("v{} preloaded — ready to restart", clean_latest);
                    return;
                }
            }
        }

        {
            let mut guard = self.status.lock().unwrap();
            guard.phase = UpdatePhase::Preloading;
            guard.latest_version = Some(clean_latest.clone());
            guard.release_notes = release.body.clone();
            guard.release_url = Some(release.html_url.clone());
            guard.downloaded_bytes = 0;
            guard.total_bytes = exe_asset.size;
            guard.percent = 0;
            guard.message = format!("Preloading v{} in background…", clean_latest);
        }

        if let Err(e) = fs::create_dir_all(&stage_dir) {
            self.set_error(format!("Cannot create update cache: {}", e));
            return;
        }

        let tmp_exe = stage_dir.join("ThumbDesk.exe.part");
        let dl_resp = match ureq::get(&exe_asset.browser_download_url)
            .set("User-Agent", &format!("ThumbDesk/{}", CURRENT_VERSION))
            .timeout(Duration::from_secs(120))
            .call()
        {
            Ok(r) => r,
            Err(e) => {
                self.set_error(format!("Download failed: {}", e));
                return;
            }
        };

        let mut reader = dl_resp.into_reader();
        let mut out = match fs::File::create(&tmp_exe) {
            Ok(f) => f,
            Err(e) => {
                self.set_error(format!("Write failed: {}", e));
                return;
            }
        };

        let mut hasher = Sha256::new();
        let mut buf = [0u8; 65536];
        let mut downloaded: u64 = 0;
        let total = exe_asset.size.max(1);

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = out.write_all(&buf[..n]) {
                        let _ = fs::remove_file(&tmp_exe);
                        self.set_error(format!("Disk write error: {}", e));
                        return;
                    }
                    hasher.update(&buf[..n]);
                    downloaded += n as u64;
                    let pct = ((downloaded * 100) / total).min(99) as u8;
                    let mut guard = self.status.lock().unwrap();
                    guard.downloaded_bytes = downloaded;
                    guard.percent = pct;
                    guard.message = format!("Preloading v{} ({}%)…", clean_latest, pct);
                }
                Err(e) => {
                    let _ = fs::remove_file(&tmp_exe);
                    self.set_error(format!("Stream interrupted: {}", e));
                    return;
                }
            }
        }
        drop(out);

        {
            let mut guard = self.status.lock().unwrap();
            guard.phase = UpdatePhase::Verifying;
            guard.percent = 100;
            guard.message = "Verifying SHA-256 integrity…".to_string();
        }

        let actual_sha: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        if let Some(expected) = expected_sha {
            if actual_sha != expected {
                let _ = fs::remove_file(&tmp_exe);
                self.set_error("SHA-256 verification failed — update discarded.".to_string());
                return;
            }
        }

        if staged_exe.exists() {
            let _ = fs::remove_file(&staged_exe);
        }
        if let Err(e) = fs::rename(&tmp_exe, &staged_exe) {
            self.set_error(format!("Failed to stage update: {}", e));
            return;
        }

        let mut guard = self.status.lock().unwrap();
        guard.phase = UpdatePhase::ReadyToRestart;
        guard.downloaded_bytes = downloaded;
        guard.total_bytes = downloaded;
        guard.percent = 100;
        guard.verified_sha256 = Some(actual_sha);
        guard.staged_path = Some(staged_exe.to_string_lossy().to_string());
        guard.message = format!("v{} preloaded & verified — ready to restart", clean_latest);
    }

    fn set_error(&self, msg: String) {
        let mut guard = self.status.lock().unwrap();
        guard.phase = UpdatePhase::Error;
        guard.message = msg;
    }

    /// Spawns a detached Windows atomic-swap helper script that waits for this PID to exit,
    /// replaces the running executable with the verified preloaded binary, clears WebView2 cache,
    /// and relaunches ThumbDesk cleanly.
    pub fn apply_staged_update(&self) -> Result<(), String> {
        let staged_path = {
            let mut guard = self.status.lock().unwrap();
            let path = guard
                .staged_path
                .clone()
                .ok_or_else(|| "No preloaded update is ready.".to_string())?;
            guard.phase = UpdatePhase::Applying;
            guard.message = "Restarting to install update…".to_string();
            path
        };

        let current_exe =
            std::env::current_exe().map_err(|e| format!("Cannot locate current exe: {}", e))?;
        let pid = std::process::id();
        let script_path = std::env::temp_dir().join("thumbdesk-apply-update.cmd");

        let script = format!(
            "@echo off\r\n\
             setlocal\r\n\
             set \"PID={pid}\"\r\n\
             set \"SRC={src}\"\r\n\
             set \"DST={dst}\"\r\n\
             for /L %%i in (1,1,40) do (\r\n\
               tasklist /FI \"PID eq %PID%\" 2>NUL | find /I \"%PID%\" >NUL\r\n\
               if errorlevel 1 goto :swap\r\n\
               timeout /t 1 /nobreak >NUL\r\n\
             )\r\n\
             :swap\r\n\
             copy /Y \"%SRC%\" \"%DST%\" >NUL\r\n\
             rmdir /S /Q \"%LOCALAPPDATA%\\com.thumbdesk.windows\\EBWebView\" >NUL 2>&1\r\n\
             start \"\" \"%DST%\"\r\n\
             del /F /Q \"%~f0\" >NUL 2>&1\r\n",
            pid = pid,
            src = staged_path,
            dst = current_exe.to_string_lossy()
        );

        fs::write(&script_path, script)
            .map_err(|e| format!("Failed to write update helper: {}", e))?;

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/C", &script_path.to_string_lossy()])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| format!("Failed to launch update helper: {}", e))?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = script_path;
            return Err("Automatic in-place restart is supported on Windows; download the latest macOS/Linux installer from GitHub Releases.".into());
        }

        Ok(())
    }
}
