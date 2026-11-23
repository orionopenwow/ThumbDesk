/// Laptop-style relative cursor engine for DualSense / DualShock 4 capacitive touchpad.
#[derive(Debug, Default)]
pub struct TouchpadMouseEngine {
    was_touching: bool,
    last_x: f64,
    last_y: f64,
}

impl TouchpadMouseEngine {
    pub fn reset(&mut self) {
        self.was_touching = false;
    }

    /// Feeds normalized touchpad coordinates `(x, y)` in `0.0..=1.0` and `touching` state.
    /// Returns smoothed relative pixel delta `(dx, dy)`.
    pub fn tick(
        &mut self,
        touching: bool,
        x: f64,
        y: f64,
        sensitivity: f64,
        enabled: bool,
    ) -> Option<(f64, f64)> {
        if !enabled || !touching {
            self.was_touching = false;
            return None;
        }
        if !self.was_touching {
            self.was_touching = true;
            self.last_x = x;
            self.last_y = y;
            return None;
        }

        let raw_dx = x - self.last_x;
        let raw_dy = y - self.last_y;
        self.last_x = x;
        self.last_y = y;

        // Ignore huge single-frame teleport jumps (e.g. finger swap)
        if raw_dx.abs() > 0.35 || raw_dy.abs() > 0.35 {
            return None;
        }

        let scale = 1400.0 * sensitivity.clamp(0.2, 4.0);
        let dx = raw_dx * scale;
        let dy = raw_dy * scale;
        if dx.abs() < 0.05 && dy.abs() < 0.05 {
            None
        } else {
            Some((dx, dy))
        }
    }
}
