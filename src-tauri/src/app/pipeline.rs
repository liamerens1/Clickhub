use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::core::{LatencyStats, PressTracker};
use crate::hook::{ButtonAction, HookEvent, SHARED};
use crate::platform::{qpc_now, qpc_to_micros, tick_count_64};

/// Wake at least this often so the heartbeat stays fresh while idle.
const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(100);
/// Log a latency summary every this many button events.
const SUMMARY_EVERY: u64 = 100;

/// The consumer of hook events ("core" in docs/architecture.md). Phase 1 classifies trigger
/// presses and measures latency; Phase 2 will open the ring from here.
pub struct Pipeline {
    stop: Arc<AtomicBool>,
    stall_ms: Arc<AtomicU64>,
    join: Option<JoinHandle<()>>,
}

impl Pipeline {
    /// # Errors
    ///
    /// Fails only if the OS cannot start a thread.
    pub fn spawn(events: Receiver<HookEvent>) -> std::io::Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let stall_ms = Arc::new(AtomicU64::new(0));
        let worker = Worker {
            events,
            stop: Arc::clone(&stop),
            stall_ms: Arc::clone(&stall_ms),
            tracker: PressTracker::default(),
            delivery: LatencyStats::default(),
            button_events: 0,
        };
        let join = thread::Builder::new()
            .name("clickhub-pipeline".into())
            .spawn(move || worker.run())?;
        Ok(Self {
            stop,
            stall_ms,
            join: Some(join),
        })
    }

    /// Debug aid: blocks the pipeline thread once for `ms`, to prove the hook fails open.
    pub fn stall_for(&self, ms: u64) {
        self.stall_ms.store(ms, Ordering::Relaxed);
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
    stop: Arc<AtomicBool>,
    stall_ms: Arc<AtomicU64>,
    tracker: PressTracker,
    delivery: LatencyStats,
    button_events: u64,
}

impl Worker {
    fn run(mut self) {
        while !self.stop.load(Ordering::Relaxed) {
            SHARED.heartbeat();
            let stall_ms = self.stall_ms.swap(0, Ordering::Relaxed);
            if stall_ms > 0 {
                tracing::warn!(stall_ms, "debug: stalling the pipeline");
                thread::sleep(Duration::from_millis(stall_ms));
            }
            match self.events.recv_timeout(HEARTBEAT_INTERVAL) {
                Ok(event) => self.handle(event),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        self.log_summary("final");
    }

    fn handle(&mut self, event: HookEvent) {
        let latency_us = qpc_to_micros(qpc_now() - event.qpc);
        self.delivery.record(latency_us);
        self.button_events += 1;

        let now_ms = tick_count_64();
        match event.action {
            ButtonAction::Down if event.matched => {
                self.tracker.on_down(now_ms);
                tracing::info!(
                    button = ?event.button,
                    x = event.x,
                    y = event.y,
                    swallowed = event.swallowed,
                    latency_us,
                    "trigger pressed"
                );
            }
            ButtonAction::Up if event.matched => {
                if let Some(press) = self.tracker.on_up(now_ms) {
                    tracing::info!(
                        kind = ?press.kind,
                        held_ms = press.held_ms,
                        swallowed = event.swallowed,
                        latency_us,
                        "trigger released"
                    );
                }
            }
            _ => {}
        }

        if self.button_events.is_multiple_of(SUMMARY_EVERY) {
            self.log_summary("periodic");
        }
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
