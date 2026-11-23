use crate::models::control::{ControllerSnapshot, StickSide};
use crate::models::profile::{CrossEdgeDirection, CursorConfiguration};

pub const REFERENCE_FRAME_DURATION: f64 = 1.0 / 120.0;
pub const MAXIMUM_CATCH_UP_DURATION: f64 = 0.1;
pub const MAXIMUM_FLICK_BOOST: f64 = 2.0;
pub const FLICK_BOOST_DECAY_DURATION: f64 = 2.0;
pub const FLICK_SWEEP_START_MAGNITUDE: f64 = 0.25;
pub const FLICK_FULL_THROW_MAGNITUDE: f64 = 0.92;
pub const FLICK_ACTIVATION_DURATION: f64 = 0.065;
pub const FLICK_FULL_BOOST_DURATION: f64 = 0.04;
pub const ZOOM_GESTURE_ACTIVATION_THRESHOLD: f64 = 0.55;
pub const ZOOM_GESTURE_RELEASE_THRESHOLD: f64 = 0.42;
pub const CROSS_EDGE_SWEEP_SPEED: f64 = 6000.0;
pub const CROSS_EDGE_SWEEP_DISTANCE: f64 = 1500.0;
pub const CROSS_EDGE_SWEEP_DURATION: f64 = CROSS_EDGE_SWEEP_DISTANCE / CROSS_EDGE_SWEEP_SPEED;

/// Radial deadzone and response curve mapping.
pub fn adjusted_vector(x: f64, y: f64, dead_zone: f64, response_curve: f64) -> (f64, f64) {
    let mag = x.hypot(y);
    if mag <= 0.0 || mag <= dead_zone {
        return (0.0, 0.0);
    }
    let normalized = ((mag - dead_zone) / (1.0 - dead_zone).max(0.0001)).clamp(0.0, 1.0);
    let curved = normalized.powf(response_curve);
    ((x / mag) * curved, (y / mag) * curved)
}

pub fn smoothing_alpha(smoothing: f64) -> f64 {
    let s = smoothing.clamp(0.0, 1.0);
    (1.0 - 0.85 * s).max(0.15)
}

/// Frame-rate independent exponential smoothing relative to 120 Hz reference.
pub fn blend_velocity(
    current: (f64, f64),
    target: (f64, f64),
    smoothing: f64,
    delta_time: f64,
) -> (f64, f64) {
    let alpha_base = smoothing_alpha(smoothing);
    let frames = (delta_time / REFERENCE_FRAME_DURATION).max(0.0);
    let alpha = 1.0 - (1.0 - alpha_base).powf(frames);
    (
        current.0 + (target.0 - current.0) * alpha,
        current.1 + (target.1 - current.1) * alpha,
    )
}

pub fn decayed_flick_boost(current: f64, delta_time: f64) -> f64 {
    let decay_rate = (MAXIMUM_FLICK_BOOST - 1.0) / FLICK_BOOST_DECAY_DURATION;
    (current.min(MAXIMUM_FLICK_BOOST) - decay_rate * delta_time.max(0.0)).max(1.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FlickState {
    WaitingForCenter,
    Armed,
    Tracking { started_at: f64 },
    Disarmed,
}

#[derive(Debug, Clone)]
pub struct FlickBoostTracker {
    state: FlickState,
}

impl Default for FlickBoostTracker {
    fn default() -> Self {
        Self {
            state: FlickState::WaitingForCenter,
        }
    }
}

impl FlickBoostTracker {
    pub fn reset(&mut self) {
        self.state = FlickState::WaitingForCenter;
    }

    pub fn sample(&mut self, x: f64, y: f64, now: f64) -> f64 {
        let mag = x.hypot(y);
        if mag <= FLICK_SWEEP_START_MAGNITUDE {
            self.state = FlickState::Armed;
            return 1.0;
        }
        match self.state {
            FlickState::WaitingForCenter | FlickState::Disarmed => 1.0,
            FlickState::Armed => {
                if mag >= FLICK_FULL_THROW_MAGNITUDE {
                    self.state = FlickState::Disarmed;
                    MAXIMUM_FLICK_BOOST
                } else {
                    self.state = FlickState::Tracking { started_at: now };
                    1.0
                }
            }
            FlickState::Tracking { started_at } => {
                let dt_sweep = (now - started_at).max(0.0);
                if dt_sweep > FLICK_ACTIVATION_DURATION {
                    self.state = FlickState::Disarmed;
                    return 1.0;
                }
                if mag < FLICK_FULL_THROW_MAGNITUDE {
                    return 1.0;
                }
                self.state = FlickState::Disarmed;
                let ratio = ((FLICK_ACTIVATION_DURATION - dt_sweep)
                    / (FLICK_ACTIVATION_DURATION - FLICK_FULL_BOOST_DURATION))
                    .clamp(0.0, 1.0);
                1.0 + ratio * (MAXIMUM_FLICK_BOOST - 1.0)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomDirection {
    ZoomIn,
    ZoomOut,
}

pub fn zoom_gesture_sample(x: f64, y: f64, threshold: f64) -> Option<(ZoomDirection, f64)> {
    if y.abs() < threshold || y.abs() < x.abs() * 0.85 {
        return None;
    }
    let dir = if y > 0.0 {
        ZoomDirection::ZoomIn
    } else {
        ZoomDirection::ZoomOut
    };
    Some((dir, y.abs().clamp(0.0, 1.0)))
}

pub fn zoom_repeat_interval(magnitude: f64) -> f64 {
    let n = ((magnitude - ZOOM_GESTURE_ACTIVATION_THRESHOLD)
        / (1.0 - ZOOM_GESTURE_ACTIVATION_THRESHOLD))
        .clamp(0.0, 1.0);
    0.28 - 0.205 * n
}

#[derive(Debug, Default, Clone)]
pub struct StickZoomRepeater {
    active_direction: Option<ZoomDirection>,
    next_step_time: f64,
}

impl StickZoomRepeater {
    pub fn is_active(&self) -> bool {
        self.active_direction.is_some()
    }

    pub fn reset(&mut self) {
        self.active_direction = None;
        self.next_step_time = 0.0;
    }

    pub fn tick(&mut self, enabled: bool, x: f64, y: f64, now: f64) -> Option<ZoomDirection> {
        if !enabled {
            self.reset();
            return None;
        }
        let threshold = if self.active_direction.is_some() {
            ZOOM_GESTURE_RELEASE_THRESHOLD
        } else {
            ZOOM_GESTURE_ACTIVATION_THRESHOLD
        };
        let Some((dir, mag)) = zoom_gesture_sample(x, y, threshold) else {
            self.reset();
            return None;
        };
        let interval = zoom_repeat_interval(mag);
        if self.active_direction != Some(dir) {
            self.active_direction = Some(dir);
            self.next_step_time = now + interval;
            return Some(dir);
        }
        if now >= self.next_step_time {
            self.next_step_time = now + interval;
            return Some(dir);
        }
        None
    }
}

/// Accumulates fractional subpixel pointer deltas and yields integer steps.
#[derive(Debug, Default, Clone, Copy)]
pub struct SubpixelAccumulator {
    pending_x: f64,
    pending_y: f64,
}

impl SubpixelAccumulator {
    pub fn accumulate(&mut self, dx: f64, dy: f64) -> (i32, i32) {
        self.pending_x += dx;
        self.pending_y += dy;
        let ix = self.pending_x.trunc() as i32;
        let iy = self.pending_y.trunc() as i32;
        self.pending_x -= ix as f64;
        self.pending_y -= iy as f64;
        (ix, iy)
    }

    pub fn reset(&mut self) {
        self.pending_x = 0.0;
        self.pending_y = 0.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrossEdgeSweep {
    pub direction: CrossEdgeDirection,
    pub remaining_duration: f64,
}

impl CrossEdgeSweep {
    pub fn new(direction: CrossEdgeDirection) -> Self {
        Self {
            direction,
            remaining_duration: CROSS_EDGE_SWEEP_DURATION,
        }
    }

    pub fn step(&mut self, dt: f64) -> Option<(f64, f64)> {
        if self.remaining_duration <= 0.0 {
            return None;
        }
        let applied = dt.max(0.0).min(self.remaining_duration);
        self.remaining_duration -= applied;
        let (vx, vy) = self.direction.motion_vector();
        Some((
            vx * CROSS_EDGE_SWEEP_SPEED * applied,
            vy * CROSS_EDGE_SWEEP_SPEED * applied,
        ))
    }

    pub fn is_finished(&self) -> bool {
        self.remaining_duration <= 0.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct CursorMotionState {
    pub smoothed_velocity: (f64, f64),
    pub primary_flick_boost: f64,
    pub flick_tracker: FlickBoostTracker,
    pub zoom_repeater: StickZoomRepeater,
    pub cross_edge_sweep: Option<CrossEdgeSweep>,
    pub accumulator: SubpixelAccumulator,
    pub speeds_swapped: bool,
}

impl CursorMotionState {
    pub fn new() -> Self {
        Self {
            primary_flick_boost: 1.0,
            ..Default::default()
        }
    }

    pub fn reset_transient(&mut self) {
        self.smoothed_velocity = (0.0, 0.0);
        self.primary_flick_boost = 1.0;
        self.flick_tracker.reset();
        self.zoom_repeater.reset();
        self.cross_edge_sweep = None;
        self.accumulator.reset();
    }

    pub fn start_cross_edge_sweep(&mut self, direction: CrossEdgeDirection) {
        self.smoothed_velocity = (0.0, 0.0);
        self.primary_flick_boost = 1.0;
        self.flick_tracker.reset();
        self.zoom_repeater.reset();
        self.cross_edge_sweep = Some(CrossEdgeSweep::new(direction));
    }

    pub fn update_input(&mut self, snapshot: &ControllerSnapshot, config: &CursorConfiguration, now: f64) {
        if !config.flick_boost_enabled || self.zoom_repeater.is_active() {
            self.primary_flick_boost = 1.0;
            self.flick_tracker.reset();
            return;
        }
        if let Some(side) = config.primary_stick.stick_side() {
            let stick = snapshot.stick(side);
            let mag = stick.x.hypot(stick.y);
            if mag <= config.dead_zone {
                self.primary_flick_boost = 1.0;
            }
            let detected = self.flick_tracker.sample(stick.x, stick.y, now);
            self.primary_flick_boost = self.primary_flick_boost.max(detected);
        } else {
            self.primary_flick_boost = 1.0;
            self.flick_tracker.reset();
        }
    }

    fn stick_velocity(
        &self,
        side: StickSide,
        raw_x: f64,
        raw_y: f64,
        speed: f64,
        speed_multiplier: f64,
        config: &CursorConfiguration,
    ) -> (f64, f64) {
        let (ax, ay) = adjusted_vector(raw_x, raw_y, config.dead_zone, config.response_curve);
        let curved_mag = ax.hypot(ay);
        if curved_mag <= 0.0 {
            return (0.0, 0.0);
        }
        let (invert_x, invert_y) = match side {
            StickSide::Left => (config.invert_primary_x, config.invert_primary_y),
            StickSide::Right => (config.invert_precision_x, config.invert_precision_y),
        };
        let mut ux = ax;
        let mut uy = -ay; // Stick up (+Y) -> screen up (-Y)
        if invert_x {
            ux = -ux;
        }
        if invert_y {
            uy = -uy;
        }
        ux *= config.horizontal_speed_multiplier;
        uy *= config.vertical_speed_multiplier;

        let mut vx = ux * speed * speed_multiplier;
        let mut vy = uy * speed * speed_multiplier;
        if config.acceleration_enabled {
            let accel = 1.0 + curved_mag * curved_mag * 0.35;
            vx *= accel;
            vy *= accel;
        }
        (vx, vy)
    }

    /// Computes frame delta `(dx, dy)` for a single tick of duration `raw_dt`.
    pub fn tick(
        &mut self,
        snapshot: &ControllerSnapshot,
        config: &CursorConfiguration,
        raw_dt: f64,
        suppress_left_stick: bool,
    ) -> Option<(f64, f64)> {
        let dt = raw_dt.clamp(0.0, MAXIMUM_CATCH_UP_DURATION);
        if dt <= 0.0 {
            return None;
        }

        // Cross-edge sweep takes exclusive priority until finished
        if let Some(sweep) = &mut self.cross_edge_sweep {
            let delta = sweep.step(dt);
            if sweep.is_finished() {
                self.cross_edge_sweep = None;
            }
            return delta;
        }

        let (primary_speed, precision_speed) = if self.speeds_swapped {
            (config.precision_speed, config.primary_speed)
        } else {
            (config.primary_speed, config.precision_speed)
        };

        let mut vx_comb = 0.0;
        let mut vy_comb = 0.0;
        let mut v_max: f64 = 0.0;

        if let Some(side) = config.primary_stick.stick_side() {
            if !(suppress_left_stick && side == StickSide::Left) {
                let s = snapshot.stick(side);
                let boost = self.primary_flick_boost.max(1.0);
                let (vx, vy) =
                    self.stick_velocity(side, s.x, s.y, primary_speed, boost, config);
                vx_comb += vx;
                vy_comb += vy;
                v_max = v_max.max(primary_speed * boost);
            }
        }

        if let Some(side) = config.precision_stick.stick_side() {
            if !(suppress_left_stick && side == StickSide::Left) {
                let s = snapshot.stick(side);
                let (vx, vy) =
                    self.stick_velocity(side, s.x, s.y, precision_speed, 1.0, config);
                vx_comb += vx;
                vy_comb += vy;
                v_max = v_max.max(precision_speed);
            }
        }

        let comb_mag = vx_comb.hypot(vy_comb);
        if v_max > 0.0 && comb_mag > v_max {
            vx_comb = (vx_comb / comb_mag) * v_max;
            vy_comb = (vy_comb / comb_mag) * v_max;
        }

        if suppress_left_stick {
            self.smoothed_velocity = (vx_comb, vy_comb);
        } else {
            self.smoothed_velocity =
                blend_velocity(self.smoothed_velocity, (vx_comb, vy_comb), config.smoothing, dt);
        }

        if config.flick_boost_enabled {
            self.primary_flick_boost = decayed_flick_boost(self.primary_flick_boost, dt);
        } else {
            self.primary_flick_boost = 1.0;
        }

        if self.smoothed_velocity.0.hypot(self.smoothed_velocity.1) < 0.01 {
            self.smoothed_velocity = (0.0, 0.0);
            return None;
        }

        Some((self.smoothed_velocity.0 * dt, self.smoothed_velocity.1 * dt))
    }
}
