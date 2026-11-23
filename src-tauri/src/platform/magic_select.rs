use serde::{Deserialize, Serialize};

/// A clickable/interactive UI target rectangle on screen.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MagicTarget {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl MagicTarget {
    pub fn center(self) -> (f64, f64) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn contains(self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

/// Finds the nearest interactive target center within `max_radius` pixels of `(cx, cy)`.
pub fn find_nearest_snap_target(
    cx: f64,
    cy: f64,
    targets: &[MagicTarget],
    max_radius: f64,
) -> Option<(f64, f64)> {
    let mut best: Option<(f64, (f64, f64))> = None;
    for t in targets {
        let (tx, ty) = t.center();
        let dist = (tx - cx).hypot(ty - cy);
        // Skip if already dead-center (< 3px) or outside snap radius
        if dist < 3.0 || dist > max_radius {
            continue;
        }
        // Slight preference if cursor is already inside the element's bounding box
        let score = if t.contains(cx, cy) { dist * 0.5 } else { dist };
        if best.map(|(b, _)| score < b).unwrap_or(true) {
            best = Some((score, (tx, ty)));
        }
    }
    best.map(|(_, pos)| pos)
}

/// Finds the best next interactive target in the flick direction `(dir_x, dir_y)` (screen coords: +X right, +Y down).
pub fn find_directional_next_target(
    cx: f64,
    cy: f64,
    dir_x: f64,
    dir_y: f64,
    targets: &[MagicTarget],
) -> Option<(f64, f64)> {
    let dlen = dir_x.hypot(dir_y);
    if dlen < 1e-4 {
        return None;
    }
    let ux = dir_x / dlen;
    let uy = dir_y / dlen;

    let mut best: Option<(f64, (f64, f64))> = None;
    for t in targets {
        let (tx, ty) = t.center();
        let vx = tx - cx;
        let vy = ty - cy;
        let dist = vx.hypot(vy);
        if dist < 12.0 || dist > 900.0 {
            continue;
        }
        let dot = (vx * ux + vy * uy) / dist;
        // Must lie within ~60-degree directional cone
        if dot < 0.48 {
            continue;
        }
        // Score prefers close distance + strong alignment with flick vector
        let score = dist / (dot * dot);
        if best.map(|(b, _)| score < b).unwrap_or(true) {
            best = Some((score, (tx, ty)));
        }
    }
    best.map(|(_, pos)| pos)
}

/// Enumerates visible enabled child UI controls in the current foreground window on Windows.
#[cfg(target_os = "windows")]
pub fn collect_foreground_targets() -> Vec<MagicTarget> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
    use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumChildWindows, GetForegroundWindow, GetWindowRect, IsWindowVisible,
    };

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !IsWindowVisible(hwnd).as_bool() || !IsWindowEnabled(hwnd).as_bool() {
            return BOOL(1);
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let w = (rect.right - rect.left) as f64;
            let h = (rect.bottom - rect.top) as f64;
            // Filter out tiny invisible 0px artifacts or giant full-window containers
            if (14.0..=520.0).contains(&w) && (14.0..=260.0).contains(&h) {
                let targets = &mut *(lparam.0 as *mut Vec<MagicTarget>);
                targets.push(MagicTarget {
                    x: rect.left as f64,
                    y: rect.top as f64,
                    width: w,
                    height: h,
                });
            }
        }
        BOOL(1)
    }

    let mut targets = Vec::with_capacity(64);
    unsafe {
        let fg = GetForegroundWindow();
        if !fg.0.is_null() {
            let ptr = &mut targets as *mut Vec<MagicTarget> as isize;
            let _ = EnumChildWindows(fg, Some(enum_proc), LPARAM(ptr));
        }
    }
    targets
}

#[cfg(not(target_os = "windows"))]
pub fn collect_foreground_targets() -> Vec<MagicTarget> {
    Vec::new()
}
