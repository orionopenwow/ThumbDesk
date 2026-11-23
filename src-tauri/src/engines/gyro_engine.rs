use serde::{Deserialize, Serialize};

use crate::models::control::{ControllerControlID, ControllerSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum GyroActivationMode {
    #[default]
    TouchpadTouch,
    Always,
    HoldLeftTrigger,
    HoldRightShoulder,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GyroConfiguration {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub activation_mode: GyroActivationMode,
    #[serde(default = "default_gyro_sensitivity")]
    pub sensitivity: f64,
    #[serde(default = "default_gyro_deadzone")]
    pub deadzone: f64,
    #[serde(default = "default_gyro_smoothing")]
    pub smoothing: f64,
    #[serde(default)]
    pub invert_x: bool,
    #[serde(default)]
    pub invert_y: bool,
}

fn default_gyro_sensitivity() -> f64 {
    1.2
}
fn default_gyro_deadzone() -> f64 {
    0.04
}
fn default_gyro_smoothing() -> f64 {
    0.25
}

impl Default for GyroConfiguration {
    fn default() -> Self {
        Self {
            enabled: false,
            activation_mode: GyroActivationMode::TouchpadTouch,
            sensitivity: default_gyro_sensitivity(),
            deadzone: default_gyro_deadzone(),
            smoothing: default_gyro_smoothing(),
            invert_x: false,
            invert_y: false,
        }
    }
}

/// 6-Axis IMU gyro-to-cursor engine with deadzone, exponential smoothing, and activation gating.
#[derive(Debug, Default)]
pub struct GyroEngine {
    smooth_x: f64,
    smooth_y: f64,
}

impl GyroEngine {
    pub fn reset(&mut self) {
        self.smooth_x = 0.0;
        self.smooth_y = 0.0;
    }

    pub fn is_active(cfg: &GyroConfiguration, snapshot: &ControllerSnapshot) -> bool {
        if !cfg.enabled || !snapshot.is_connected || !snapshot.has_gyro {
            return false;
        }
        match cfg.activation_mode {
            GyroActivationMode::Always => true,
            GyroActivationMode::TouchpadTouch => {
                snapshot.touchpad_touched
                    || snapshot
                        .pressed_controls
                        .contains(&ControllerControlID::TouchpadButton)
            }
            GyroActivationMode::HoldLeftTrigger => {
                snapshot.value_for(ControllerControlID::LeftTrigger) > 0.25
            }
            GyroActivationMode::HoldRightShoulder => snapshot
                .pressed_controls
                .contains(&ControllerControlID::RightShoulder),
        }
    }

    /// Computes cursor delta `(dx, dy)` in pixels from raw normalized gyro yaw/pitch rates.
    pub fn tick(
        &mut self,
        snapshot: &ControllerSnapshot,
        cfg: &GyroConfiguration,
        dt: f64,
    ) -> Option<(f64, f64)> {
        if !Self::is_active(cfg, snapshot) {
            self.reset();
            return None;
        }

        // DualSense/DS4 yaw = gyro[1], pitch = gyro[0]
        let mut raw_x = -(snapshot.gyro[1] as f64);
        let mut raw_y = -(snapshot.gyro[0] as f64);

        let mag = (raw_x * raw_x + raw_y * raw_y).sqrt();
        if mag < cfg.deadzone {
            raw_x = 0.0;
            raw_y = 0.0;
        } else {
            let scale = (mag - cfg.deadzone) / mag;
            raw_x *= scale;
            raw_y *= scale;
        }

        if cfg.invert_x {
            raw_x = -raw_x;
        }
        if cfg.invert_y {
            raw_y = -raw_y;
        }

        let alpha = (1.0 - cfg.smoothing.clamp(0.0, 0.92)).clamp(0.08, 1.0);
        self.smooth_x += (raw_x - self.smooth_x) * alpha;
        self.smooth_y += (raw_y - self.smooth_y) * alpha;

        let px_per_sec = 2400.0 * cfg.sensitivity.clamp(0.1, 5.0);
        let dx = self.smooth_x * px_per_sec * dt;
        let dy = self.smooth_y * px_per_sec * dt;

        if dx.abs() < 1e-4 && dy.abs() < 1e-4 {
            None
        } else {
            Some((dx, dy))
        }
    }
}
