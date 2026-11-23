use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::models::hardware_fx::{HardwareFxPreferences, LightbarMode};
use crate::models::vibration::ControllerVibration;
use crate::runtime::AppRuntime;

pub const AUTOMATION_API_PORT: u16 = 49152;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationApiStatus {
    pub enabled: bool,
    pub endpoint: String,
    pub requests_served: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HapticRequest {
    pattern: ControllerVibration,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LightbarRequest {
    rgb: [u8; 3],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileRequest {
    profile_id: String,
}

/// Spawns a lightweight localhost-only (`127.0.0.1:49152`) HTTP JSON bridge for
/// OBS, Stream Deck, VS Code tasks, and build/AI completion notifications.
pub fn start_automation_api(runtime: Arc<AppRuntime>) {
    std::thread::spawn(move || {
        let Ok(rt) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };

        rt.block_on(async move {
            let Ok(listener) =
                TcpListener::bind(format!("127.0.0.1:{}", AUTOMATION_API_PORT)).await
            else {
                return;
            };

            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    continue;
                };
                let rt_ref = Arc::clone(&runtime);
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let Ok(n) = socket.read(&mut buf).await else {
                        return;
                    };
                    if n == 0 {
                        return;
                    }
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let (status_code, body) = handle_http_request(&req, &rt_ref);
                    let resp = format!(
                        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        status_code,
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                });
            }
        });
    });
}

pub fn handle_http_request(raw: &str, runtime: &AppRuntime) -> (&'static str, String) {
    let mut lines = raw.lines();
    let first = lines.next().unwrap_or("");
    let parts: Vec<&str> = first.split_whitespace().collect();
    if parts.len() < 2 {
        return ("400 Bad Request", r#"{"error":"malformed request"}"#.into());
    }
    let method = parts[0];
    let path = parts[1];
    let body = raw.split("\r\n\r\n").nth(1).unwrap_or("").trim();

    match (method, path) {
        ("GET", "/api/status") => {
            let st = runtime.state_payload();
            let summary = serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "connected": st.snapshot.is_connected,
                "controller": st.snapshot.name,
                "runtimeEnabled": st.runtime_enabled,
                "guardSuspended": st.game_guard_status.is_suspended,
                "voiceListening": st.voice_state.listening,
                "activeProfileId": st.document.active_profile_id,
            });
            ("200 OK", summary.to_string())
        }
        ("POST", "/api/haptic") => {
            if let Ok(req) = serde_json::from_str::<HapticRequest>(body) {
                runtime.preview_vibration(req.pattern);
                ("200 OK", r#"{"ok":true}"#.into())
            } else {
                runtime.preview_vibration(ControllerVibration::DoubleTap);
                ("200 OK", r#"{"ok":true,"pattern":"doubleTap"}"#.into())
            }
        }
        ("POST", "/api/lightbar") => {
            if let Ok(req) = serde_json::from_str::<LightbarRequest>(body) {
                let mut prefs: HardwareFxPreferences = runtime.state_payload().hardware_fx;
                prefs.lightbar_mode = LightbarMode::CustomColor;
                prefs.custom_rgb = req.rgb;
                let _ = runtime.set_hardware_fx(prefs);
                ("200 OK", r#"{"ok":true}"#.into())
            } else {
                ("400 Bad Request", r#"{"error":"expected {rgb:[r,g,b]}"}"#.into())
            }
        }
        ("POST", "/api/profile") => {
            if let Ok(req) = serde_json::from_str::<ProfileRequest>(body) {
                match runtime.select_profile(req.profile_id) {
                    Ok(_) => ("200 OK", r#"{"ok":true}"#.into()),
                    Err(e) => ("400 Bad Request", serde_json::json!({"error": e}).to_string()),
                }
            } else {
                ("400 Bad Request", r#"{"error":"expected {profileId}"}"#.into())
            }
        }
        _ => ("404 Not Found", r#"{"error":"unknown endpoint"}"#.into()),
    }
}
