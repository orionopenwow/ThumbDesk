use std::collections::BTreeMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::engines::action_engine::TRIGGER_ACTUATION_THRESHOLD;
use crate::models::control::{ControllerControlID, ControllerSnapshot};

#[derive(Debug, Clone)]
struct ScrollSession {
    modifier: Option<ControllerControlID>,
    vertical: i32,
    horizontal: i32,
    interval: Duration,
    next_fire: Instant,
}

struct SharedRepeaterState {
    sessions: BTreeMap<ControllerControlID, ScrollSession>,
    latest_input: ControllerSnapshot,
    shutdown: bool,
}

/// Dedicated background scroll repeater with runaway prevention on release or disconnect.
pub struct HeldScrollRepeater {
    state: Arc<(Mutex<SharedRepeaterState>, Condvar)>,
    _worker: Option<thread::JoinHandle<()>>,
}

impl HeldScrollRepeater {
    pub fn new<F>(on_scroll: F) -> Self
    where
        F: Fn(i32, i32) + Send + Sync + 'static,
    {
        let state = Arc::new((
            Mutex::new(SharedRepeaterState {
                sessions: BTreeMap::new(),
                latest_input: ControllerSnapshot::default(),
                shutdown: false,
            }),
            Condvar::new(),
        ));

        let worker_state = Arc::clone(&state);
        let handle = thread::Builder::new()
            .name("vibe-held-scroll".into())
            .spawn(move || {
                let (lock, cvar) = &*worker_state;
                loop {
                    let mut guard = lock.lock().unwrap();
                    if guard.shutdown {
                        break;
                    }
                    if guard.sessions.is_empty() {
                        guard = cvar.wait(guard).unwrap();
                        continue;
                    }

                    let now = Instant::now();
                    let mut to_emit = Vec::new();
                    let mut earliest: Option<Instant> = None;

                    for session in guard.sessions.values_mut() {
                        if session.next_fire <= now {
                            to_emit.push((session.vertical, session.horizontal));
                            session.next_fire = now + session.interval;
                        }
                        earliest = Some(match earliest {
                            Some(e) => e.min(session.next_fire),
                            None => session.next_fire,
                        });
                    }

                    if !to_emit.is_empty() {
                        drop(guard);
                        for (v, h) in to_emit {
                            on_scroll(v, h);
                        }
                        continue;
                    }

                    if let Some(wake) = earliest {
                        let wait_dur = wake.saturating_duration_since(Instant::now());
                        if !wait_dur.is_zero() {
                            let _ = cvar.wait_timeout(guard, wait_dur).unwrap();
                        }
                    }
                }
            })
            .ok();

        Self {
            state,
            _worker: handle,
        }
    }

    fn is_held(
        control: ControllerControlID,
        modifier: Option<ControllerControlID>,
        snapshot: &ControllerSnapshot,
    ) -> bool {
        if !snapshot.is_connected {
            return false;
        }
        let actuated = |c: ControllerControlID| match c {
            ControllerControlID::LeftTrigger | ControllerControlID::RightTrigger => {
                snapshot.value_for(c) >= TRIGGER_ACTUATION_THRESHOLD
            }
            _ => snapshot.pressed_controls.contains(&c),
        };
        if !actuated(control) {
            return false;
        }
        if let Some(m) = modifier {
            if !actuated(m) {
                return false;
            }
        }
        true
    }

    pub fn update_input(&self, snapshot: &ControllerSnapshot) {
        let (lock, cvar) = &*self.state;
        let mut guard = lock.lock().unwrap();
        guard.latest_input = snapshot.clone();
        let before = guard.sessions.len();
        guard
            .sessions
            .retain(|&control, s| Self::is_held(control, s.modifier, snapshot));
        if guard.sessions.len() != before {
            cvar.notify_all();
        }
    }

    pub fn start(
        &self,
        control: ControllerControlID,
        modifier: Option<ControllerControlID>,
        vertical: i32,
        horizontal: i32,
        delay_secs: f64,
        interval_secs: f64,
    ) {
        let (lock, cvar) = &*self.state;
        let mut guard = lock.lock().unwrap();
        guard.sessions.remove(&control);
        if !Self::is_held(control, modifier, &guard.latest_input) {
            return;
        }
        let delay = Duration::from_secs_f64(delay_secs.max(0.01));
        let interval = Duration::from_secs_f64(interval_secs.max(0.01));
        guard.sessions.insert(
            control,
            ScrollSession {
                modifier,
                vertical,
                horizontal,
                interval,
                next_fire: Instant::now() + delay,
            },
        );
        cvar.notify_all();
    }

    pub fn stop(&self, control: ControllerControlID) {
        let (lock, cvar) = &*self.state;
        let mut guard = lock.lock().unwrap();
        if guard.sessions.remove(&control).is_some() {
            cvar.notify_all();
        }
    }

    pub fn stop_all(&self) {
        let (lock, cvar) = &*self.state;
        let mut guard = lock.lock().unwrap();
        if !guard.sessions.is_empty() {
            guard.sessions.clear();
            cvar.notify_all();
        }
    }
}

impl Drop for HeldScrollRepeater {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.state;
        if let Ok(mut guard) = lock.lock() {
            guard.shutdown = true;
            cvar.notify_all();
        }
    }
}
