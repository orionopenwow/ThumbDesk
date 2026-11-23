use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use windows::core::PWSTR;
use windows::Win32::Foundation::{BOOL, CloseHandle, HWND, LPARAM};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningAppInfo {
    pub bundle_identifier: String,
    pub display_name: String,
    pub executable_path: String,
}

fn process_info_for_hwnd(hwnd: HWND) -> Option<RunningAppInfo> {
    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    if pid == 0 {
        return None;
    }
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? };
    let mut buf = [0u16; 1024];
    let mut size = buf.len() as u32;
    let ok = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
    };
    unsafe {
        let _ = CloseHandle(handle);
    }
    if !ok || size == 0 {
        return None;
    }
    let exe_path = String::from_utf16_lossy(&buf[..size as usize]);
    let file_name = Path::new(&exe_path)
        .file_name()
        .and_then(|s| s.to_str())?
        .to_ascii_lowercase();
    let stem = Path::new(&exe_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&file_name);

    let mut chars = stem.chars();
    let display = match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => file_name.clone(),
    };

    Some(RunningAppInfo {
        bundle_identifier: file_name,
        display_name: display,
        executable_path: exe_path,
    })
}

pub fn frontmost_application() -> Option<RunningAppInfo> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return None;
    }
    process_info_for_hwnd(hwnd)
}

pub fn list_running_apps() -> Vec<RunningAppInfo> {
    let mut map: BTreeMap<String, RunningAppInfo> = BTreeMap::new();
    let ptr = &mut map as *mut BTreeMap<String, RunningAppInfo>;

    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }
        let title_len = GetWindowTextLengthW(hwnd);
        if title_len <= 0 {
            return BOOL(1);
        }
        let mut title_buf = vec![0u16; (title_len + 1) as usize];
        let _ = GetWindowTextW(hwnd, &mut title_buf);
        if let Some(info) = process_info_for_hwnd(hwnd) {
            if info.bundle_identifier != "vibe-controller-windows.exe" {
                let map = &mut *(lparam.0 as *mut BTreeMap<String, RunningAppInfo>);
                map.entry(info.bundle_identifier.clone()).or_insert(info);
            }
        }
        BOOL(1)
    }

    unsafe {
        let _ = EnumWindows(Some(callback), LPARAM(ptr as isize));
    }
    map.into_values().collect()
}
