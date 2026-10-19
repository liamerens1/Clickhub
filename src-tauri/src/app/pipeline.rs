use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::actions::ActionRunner;
use crate::config::Action;
use crate::core::{Aim, Button, LatencyStats, RingEffect, RingInput, RingSession};
use crate::hook::{ButtonAction, HookEvent, HookEventKind, SHARED};
use crate::overlay::Overlays;
use crate::platform::{qpc_now, qpc_to_micros, tick_count_64};

/// Wake at least this often so the heartbeat stays fresh and idle rings can time out.
const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(100);
/// While a ring is open, aim updates go to the page at most this often (a 1000 Hz mouse would
/// otherwise send 1000 events a second). A change of focused slot is always sent at once, and a
/// trailing update lands the exact final position.
const AIM_INTERVAL: Duration = Duration::from_millis(4);
/// Button events older than the hook's fail-open window (500 ms without a heartbeat) are
/// ignored: the hook passed them through to the apps, so they are not ring input any more.
const STALE_EVENT_US: u64 = 500_000;
/// While a ring is open it is re-asserted on top this often.
const ON_TOP_INTERVAL_US: u64 = 100_000;
/// Log a latency summary every this many button events.
const SUMMARY_EVERY: u64 = 100;
/// Aim dead zone around the ring centre, in CSS pixels (scaled per monitor).
const DEAD_ZONE_CSS_PX: f64 = 44.0;

/// Messages from the rest of the app to the pipeline thread.
pub enum PipelineCommand {
    /// New ring contents after a config (re)load.
    SetRing(Vec<Action>),
    /// The keyboard-shortcut trigger went down or up (opens at the cursor).
    KeyboardTrigger { pressed: bool },
    /// Debug aid: block the pipeline thread once for this many milliseconds.
    Stall(u64),
}

/// The consumer of hook events ("core" in docs/architecture.md): drives the ring session,
/// the overlay and the action runner.
pub struct Pipeline {
    commands: Sender<PipelineCommand>,
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl Pipeline {
    /// # Errors
    ///
    /// Fails only if a thread cannot be started.
    pub fn spawn(events: Receiver<HookEvent>, overlays: Arc<Overlays>) -> std::io::Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let (commands, command_receiver) = channel();
        let worker = Worker {
            events,
            commands: command_receiver,
            stop: Arc::clone(&stop),
            session: RingSession::new(0, DEAD_ZONE_CSS_PX),
            actions: Vec::new(),
            overlays,
            runner: ActionRunner::spawn()?,
            delivery: LatencyStats::default(),
            button_events: 0,
            open_hook_qpc: 0,
            pending_aim: None,
            last_aim_qpc: 0,
            last_aim_nearest: None,
            last_on_top_qpc: 0,
        };
        let join = thread::Builder::new()
            .name("clickhub-pipeline".into())
            .spawn(move || worker.run())?;
        Ok(Self {
            commands,
            stop,
            join: Some(join),
        })
    }

    pub fn send(&self, command: PipelineCommand) {
        if self.commands.send(command).is_err() {
            tracing::error!("pipeline thread is gone; command dropped");
        }
    }

    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take()
            && join.join().is_err()
        {
            tracing::error!("pipeline thread panicked");
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct Worker {
    events: Receiver<HookEvent>,
    commands: Receiver<PipelineCommand>,
    stop: Arc<AtomicBool>,
    session: RingSession,
    actions: Vec<Action>,
    overlays: Arc<Overlays>,
    runner: ActionRunner,
    delivery: LatencyStats,
    button_events: u64,
    open_hook_qpc: i64,
    pending_aim: Option<Aim>,
    last_aim_qpc: i64,
    last_aim_nearest: Option<usize>,
    last_on_top_qpc: i64,
}

impl Worker {
    fn run(mut self) {
        while !self.stop.load(Ordering::Relaxed) {
            SHARED.heartbeat();
            if self.session.is_open()
                && qpc_to_micros(qpc_now() - self.last_on_top_qpc) >= ON_TOP_INTERVAL_US
            {
                // Switching apps (Alt+Tab) can raise another always-on-top window above the ring.
                self.last_on_top_qpc = qpc_now();
                self.overlays.keep_open_ring_on_top();
            }
            while let Ok(command) = self.commands.try_recv() {
                self.command(command);
            }
            let wait = if self.pending_aim.is_some() {
                AIM_INTERVAL
            } else {
                HEARTBEAT_INTERVAL
            };
            match self.events.recv_timeout(wait) {
                Ok(event) => self.handle(event),
                Err(RecvTimeoutError::Timeout) => {
                    if let Some(aim) = self.pending_aim.take() {
                        self.send_aim(aim);
                    }
                    let effects = self.session.handle(RingInput::Tick, tick_count_64());
                    self.apply(&effects);
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        let effects = self.session.reconfigure(0, DEAD_ZONE_CSS_PX);
        self.apply(&effects);
        self.log_summary("final");
    }

    fn command(&mut self, command: PipelineCommand) {
        match command {
            PipelineCommand::SetRing(actions) => {
                let effects = self.session.reconfigure(actions.len(), DEAD_ZONE_CSS_PX);
                self.actions = actions;
                self.apply(&effects);
            }
            PipelineCommand::KeyboardTrigger { pressed } => {
                let input = if pressed {
                    let (x, y) = SHARED.point();
                    self.open_hook_qpc = qpc_now();
                    RingInput::TriggerDown { x, y }
                } else {
                    RingInput::TriggerUp
                };
                let effects = self.session.handle(input, tick_count_64());
                self.apply(&effects);
            }
            PipelineCommand::Stall(ms) => {
                tracing::warn!(stall_ms = ms, "debug: stalling the pipeline");
                thread::sleep(Duration::from_millis(ms));
            }
        }
    }

    fn handle(&mut self, event: HookEvent) {
        let input = match event.kind {
            HookEventKind::Button {
                button,
                action,
                matched,
                swallowed,
            } => {
                let latency_us = qpc_to_micros(qpc_now() - event.qpc);
                if latency_us > STALE_EVENT_US {
                    // Queued while the pipeline was stalled. The hook already let these clicks
                    // through (fail open), so acting on them now would open a ring seconds late.
                    tracing::debug!(latency_us, "dropping a stale button event");
                    return;
                }
                self.delivery.record(latency_us);
                self.button_events += 1;
                if self.button_events.is_multiple_of(SUMMARY_EVERY) {
                    self.log_summary("periodic");
                }
                ring_input(button, action, matched, swallowed, event)
            }
            HookEventKind::Move => Some(RingInput::Move {
                x: event.x,
                y: event.y,
            }),
            HookEventKind::Escape => Some(RingInput::Dismiss),
        };
        if let Some(input) = input {
            if matches!(input, RingInput::TriggerDown { .. }) {
                self.open_hook_qpc = event.qpc;
            }
            let effects = self.session.handle(input, tick_count_64());
            self.apply(&effects);
        }
    }

    fn apply(&mut self, effects: &[RingEffect]) {
        let mut fired = None;
        for effect in effects {
            match *effect {
                RingEffect::Open { x, y } => {
                    SHARED.set_ring_open(true);
                    if let Some(opened) = self.overlays.open(x, y, self.open_hook_qpc) {
                        self.session.set_dead_zone(DEAD_ZONE_CSS_PX * opened.scale);
                        self.session.set_layout(opened.layout.angles);
                    }
                    tracing::debug!(x, y, "ring open");
                }
                RingEffect::Aim(aim) => {
                    let due = qpc_to_micros(qpc_now() - self.last_aim_qpc)
                        >= u64::try_from(AIM_INTERVAL.as_micros()).unwrap_or(u64::MAX);
                    if due || aim.nearest != self.last_aim_nearest {
                        self.send_aim(aim);
                    } else {
                        self.pending_aim = Some(aim);
                    }
                }
                RingEffect::Fire { slot } => {
                    fired = Some(slot);
                    if let Some(action) = self.actions.get(slot) {
                        tracing::info!(slot, action = %action.hint(), "slot fired");
                        self.runner.run(action.clone());
                    } else {
                        tracing::warn!(slot, "fired slot has no action");
                    }
                }
                RingEffect::Close => {
                    self.pending_aim = None;
                    self.last_aim_nearest = None;
                    SHARED.set_ring_open(false);
                    self.overlays.close(fired);
                }
            }
        }
    }

    fn send_aim(&mut self, aim: Aim) {
        self.pending_aim = None;
        self.last_aim_qpc = qpc_now();
        self.last_aim_nearest = aim.nearest;
        self.overlays.aim(aim);
    }

    fn log_summary(&self, reason: &str) {
        let mut callback = LatencyStats::default();
        for ticks in SHARED.durations() {
            callback.record(qpc_to_micros(i64::from(ticks)));
        }
        let delivery = self.delivery.summary();
        let callback = callback.summary();
        tracing::info!(
            reason,
            button_events = self.button_events,
            delivery_p50_us = delivery.map(|s| s.p50_us),
            delivery_p99_us = delivery.map(|s| s.p99_us),
            delivery_max_us = delivery.map(|s| s.max_us),
            callback_samples = callback.map(|s| s.count),
            callback_p50_us = callback.map(|s| s.p50_us),
            callback_p99_us = callback.map(|s| s.p99_us),
            callback_max_us = callback.map(|s| s.max_us),
            dropped_events = SHARED.dropped_events(),
            "hook latency"
        );
    }
}

/// Maps a button event to what it means for the ring.
fn ring_input(
    button: Button,
    action: ButtonAction,
    matched: bool,
    swallowed: bool,
    event: HookEvent,
) -> Option<RingInput> {
    match (action, matched, swallowed) {
        (ButtonAction::Down, true, _) => Some(RingInput::TriggerDown {
            x: event.x,
            y: event.y,
        }),
        (ButtonAction::Up, true, _) => Some(RingInput::TriggerUp),
        // A non-trigger click captured by an open ring.
        (ButtonAction::Down, false, true) if button == Button::Left => {
            Some(RingInput::PrimaryClick)
        }
        (ButtonAction::Down, false, true) => Some(RingInput::Dismiss),
        _ => None,
    }
}
