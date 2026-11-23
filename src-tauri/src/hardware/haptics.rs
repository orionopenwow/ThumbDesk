use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::models::vibration::{
    ControllerVibration, HapticsPreferences, HapticsStatus, VibrationLocality, VibrationPhase,
};

pub const STALENESS_LIMIT_SECS: f64 = 0.25;
pub const DUPLICATE_PRESS_DEBOUNCE_SECS: f64 = 0.08;
pub const GRAB_RELEASE_PREVIEW_DELAY_SECS: f64 = 0.35;

pub trait RumbleSink: Send + Sync {
    fn set_motors(&self, left: f32, right: f32) -> bool;
    fn status(&self) -> HapticsStatus;
}

pub fn motor_levels(locality: VibrationLocality, intensity: f32, strength: f32) -> (f32, f32) {
    let scaled = (intensity * strength).clamp(0.0, 1.0);
    match locality {
        VibrationLocality::Left => (scaled, 0.0),
        VibrationLocality::Right => (0.0, scaled),
        VibrationLocality::Handles => (scaled, scaled),
    }
}

struct HapticsInner {
    preferences: HapticsPreferences,
    last_press: Option<(ControllerVibration, Instant)>,
    generation: u64,
}

#[derive(Clone)]
pub struct HapticsEngine {
    inner: Arc<Mutex<HapticsInner>>,
    sink: Arc<dyn RumbleSink>,
}

impl HapticsEngine {
    pub fn new(sink: Arc<dyn RumbleSink>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HapticsInner {
                preferences: HapticsPreferences::default(),
                last_press: None,
                generation: 0,
            })),
            sink,
        }
    }

    pub fn preferences(&self) -> HapticsPreferences {
        self.inner.lock().unwrap().preferences
    }

    pub fn set_preferences(&self, prefs: HapticsPreferences) {
        let mut guard = self.inner.lock().unwrap();
        guard.preferences = prefs;
        if !prefs.enabled {
            guard.generation = guard.generation.wrapping_add(1);
            drop(guard);
            let _ = self.sink.set_motors(0.0, 0.0);
        }
    }

    pub fn status(&self) -> HapticsStatus {
        let prefs = self.preferences();
        let mut st = self.sink.status();
        if !prefs.enabled {
            st.message = "Haptic feedback is turned off.".to_string();
        }
        st
    }

    pub fn stop(&self) {
        {
            let mut guard = self.inner.lock().unwrap();
            guard.generation = guard.generation.wrapping_add(1);
        }
        let _ = self.sink.set_motors(0.0, 0.0);
    }

    pub fn play_connection_pulse(&self) {
        let prefs = self.preferences();
        if prefs.enabled && prefs.connection_feedback {
            self.play(ControllerVibration::DoubleTap, VibrationPhase::Press);
        }
    }

    pub fn preview(&self, pattern: ControllerVibration) {
        if pattern == ControllerVibration::None {
            return;
        }
        self.play_internal(pattern, VibrationPhase::Press, true);
        if pattern == ControllerVibration::GrabRelease {
            let engine = self.clone();
            let gen = self.inner.lock().unwrap().generation;
            thread::spawn(move || {
                thread::sleep(Duration::from_secs_f64(GRAB_RELEASE_PREVIEW_DELAY_SECS));
                if engine.inner.lock().unwrap().generation == gen {
                    engine.play_internal(ControllerVibration::GrabRelease, VibrationPhase::Release, true);
                }
            });
        }
    }

    pub fn play(&self, pattern: ControllerVibration, phase: VibrationPhase) {
        self.play_internal(pattern, phase, false);
    }

    fn play_internal(&self, pattern: ControllerVibration, phase: VibrationPhase, force: bool) {
        if pattern == ControllerVibration::None {
            return;
        }
        let pulses = pattern.pulses(phase);
        if pulses.is_empty() {
            return;
        }
        let now = Instant::now();
        let (strength, gen) = {
            let mut guard = self.inner.lock().unwrap();
            if !guard.preferences.enabled && !force {
                return;
            }
            if !force && phase == VibrationPhase::Press {
                if let Some((last_pat, last_time)) = guard.last_press {
                    if last_pat == pattern
                        && now.duration_since(last_time).as_secs_f64() < DUPLICATE_PRESS_DEBOUNCE_SECS
                    {
                        return;
                    }
                }
                guard.last_press = Some((pattern, now));
            }
            guard.generation = guard.generation.wrapping_add(1);
            (guard.preferences.clamped_strength(), guard.generation)
        };

        let locality = pattern.locality();
        let sink = Arc::clone(&self.sink);
        let inner = Arc::clone(&self.inner);

        thread::spawn(move || {
            let started = Instant::now();
            for pulse in pulses {
                if started.elapsed().as_secs_f64() >= STALENESS_LIMIT_SECS && pulse.time == 0.0 {
                    break;
                }
                let target_start = started + Duration::from_secs_f64(pulse.time);
                if let Some(wait) = target_start.checked_duration_since(Instant::now()) {
                    thread::sleep(wait);
                }
                if inner.lock().unwrap().generation != gen {
                    return;
                }
                let (l, r) = motor_levels(locality, pulse.intensity, strength);
                let _ = sink.set_motors(l, r);
                thread::sleep(Duration::from_secs_f64(pulse.duration));
                if inner.lock().unwrap().generation != gen {
                    return;
                }
                let _ = sink.set_motors(0.0, 0.0);
            }
        });
    }
}
