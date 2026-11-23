/// Emitted by `MagicSelectEngine` when the user releases the thumbstick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MagicSelectAction {
    /// Normal free-move release: magnetically snap to the nearest UI target within snap radius.
    SnapNearest,
    /// Small-touch quick flick: jump directly to the next UI target in `(dir_x, dir_y)` screen direction (+X right, +Y down).
    FlickDirectional(f64, f64),
}

/// Tracks primary stick gestures for Magic Select:
/// - Free movement while held
/// - Magnetic snap on release
/// - Small-touch quick flick (< 180 ms) to jump to the next UI element in that direction
#[derive(Debug, Default)]
pub struct MagicSelectEngine {
    active: bool,
    start_time: f64,
    peak_mag: f64,
    peak_x: f64,
    peak_y: f64,
    last_trigger_time: f64,
}

impl MagicSelectEngine {
    pub const FLICK_MAX_DURATION_SECS: f64 = 0.18;
    pub const FLICK_MIN_PEAK: f64 = 0.26;
    pub const COOLDOWN_SECS: f64 = 0.12;

    pub fn reset(&mut self) {
        self.active = false;
        self.peak_mag = 0.0;
        self.peak_x = 0.0;
        self.peak_y = 0.0;
    }

    /// Feeds raw primary stick `(x, y)` (stick coords: +X right, +Y up) and deadzone.
    /// Returns a `MagicSelectAction` on the exact frame the stick is released back to center.
    pub fn update(
        &mut self,
        stick_x: f64,
        stick_y: f64,
        deadzone: f64,
        now_secs: f64,
        enabled: bool,
    ) -> Option<MagicSelectAction> {
        if !enabled {
            self.reset();
            return None;
        }

        let mag = stick_x.hypot(stick_y);
        let engage_thresh = deadzone.max(0.12);
        let release_thresh = engage_thresh * 0.85;

        if !self.active {
            if mag >= engage_thresh {
                self.active = true;
                self.start_time = now_secs;
                self.peak_mag = mag;
                self.peak_x = stick_x;
                self.peak_y = stick_y;
            }
            return None;
        }

        // Stick is still deflected: track peak direction & magnitude
        if mag > self.peak_mag {
            self.peak_mag = mag;
            self.peak_x = stick_x;
            self.peak_y = stick_y;
        }

        // Released back into deadzone -> classify gesture!
        if mag <= release_thresh {
            let duration = (now_secs - self.start_time).max(0.0);
            let peak_mag = self.peak_mag;
            let px = self.peak_x;
            // Convert stick +Y up into screen +Y down
            let py = -self.peak_y;
            self.reset();

            if now_secs - self.last_trigger_time < Self::COOLDOWN_SECS {
                return None;
            }
            self.last_trigger_time = now_secs;

            if duration <= Self::FLICK_MAX_DURATION_SECS && peak_mag >= Self::FLICK_MIN_PEAK {
                return Some(MagicSelectAction::FlickDirectional(px, py));
            }
            if peak_mag >= engage_thresh {
                return Some(MagicSelectAction::SnapNearest);
            }
        }

        None
    }
}
