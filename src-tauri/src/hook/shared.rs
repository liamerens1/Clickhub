use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::SyncSender;

use super::HookEvent;
use crate::core::{Button, ModifierSet, TriggerFilter};
use crate::platform::tick_count_64;

/// Callback durations kept for statistics (a ring buffer, overwritten in order).
pub const DURATION_SAMPLES: usize = 2_048;

/// If the pipeline thread has not checked in for this long, the callback stops swallowing:
/// a hung or crashed core must never eat the user's clicks (fail open).
const CORE_TIMEOUT_MS: u64 = 500;

/// State shared between the hook callback and the rest of the app.
///
/// The callback is an `extern "system"` function with no user-data pointer, so this has to be a
/// static. Every field is an atomic: the callback never locks, allocates or blocks.
pub struct Shared {
    filter_button: AtomicU8,
    filter_modifiers: AtomicU8,
    filter_swallow: AtomicBool,
    paused: AtomicBool,
    /// The foreground app is full-screen: the trigger passes through untouched.
    suppressed: AtomicBool,
    /// A ring is open: moves are posted, and clicks and Esc are captured for it.
    ring_open: AtomicBool,
    core_heartbeat_ms: AtomicU64,
    /// Bit per `Button::code()` whose `Down` was swallowed; its `Up` must be swallowed too.
    pub(super) swallowed_buttons: AtomicU8,
    /// Esc's key-down was swallowed for an open ring; swallow its key-up too.
    pub(super) escape_swallowed: AtomicBool,
    /// Last cursor position seen by the mouse hook, packed as `(x << 32) | (y as u32)`.
    pub(super) last_point: AtomicU64,
    pub(super) last_mouse_tick: AtomicU32,
    pub(super) last_keyboard_tick: AtomicU32,
    pub(super) dropped_events: AtomicU64,
    durations: [AtomicU32; DURATION_SAMPLES],
    duration_index: AtomicUsize,
}

pub static SHARED: Shared = Shared::new();

/// Where the callback posts events. Set once when the hook thread starts. A bounded
/// `sync_channel` preallocates its buffer, so `try_send` never allocates.
pub(super) static SENDER: OnceLock<SyncSender<HookEvent>> = OnceLock::new();

impl Shared {
    const fn new() -> Self {
        Self {
            filter_button: AtomicU8::new(0),
            filter_modifiers: AtomicU8::new(0),
            filter_swallow: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            suppressed: AtomicBool::new(false),
            ring_open: AtomicBool::new(false),
            core_heartbeat_ms: AtomicU64::new(0),
            swallowed_buttons: AtomicU8::new(0),
            escape_swallowed: AtomicBool::new(false),
            last_point: AtomicU64::new(0),
            last_mouse_tick: AtomicU32::new(0),
            last_keyboard_tick: AtomicU32::new(0),
            dropped_events: AtomicU64::new(0),
            durations: [const { AtomicU32::new(0) }; DURATION_SAMPLES],
            duration_index: AtomicUsize::new(0),
        }
    }

    pub fn set_filter(&self, filter: TriggerFilter) {
        self.filter_button
            .store(filter.button.map_or(0, Button::code), Ordering::Relaxed);
        self.filter_modifiers
            .store(filter.modifiers.bits(), Ordering::Relaxed);
        self.filter_swallow.store(filter.swallow, Ordering::Relaxed);
    }

    pub(super) fn filter(&self) -> TriggerFilter {
        TriggerFilter {
            button: Button::from_code(self.filter_button.load(Ordering::Relaxed)),
            modifiers: ModifierSet::from_bits(self.filter_modifiers.load(Ordering::Relaxed)),
            swallow: self.filter_swallow.load(Ordering::Relaxed),
        }
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }

    /// Set while the foreground app is full-screen.
    pub fn set_suppressed(&self, suppressed: bool) {
        self.suppressed.store(suppressed, Ordering::Relaxed);
    }

    pub fn is_suppressed(&self) -> bool {
        self.suppressed.load(Ordering::Relaxed)
    }

    /// Set by the pipeline while a ring is open.
    pub fn set_ring_open(&self, open: bool) {
        self.ring_open.store(open, Ordering::Relaxed);
    }

    /// A ring is open and the pipeline is alive. When the pipeline stalls this turns false
    /// within [`CORE_TIMEOUT_MS`], so clicks are never captured by a ring nobody serves.
    pub(super) fn ring_capturing(&self) -> bool {
        self.ring_open.load(Ordering::Relaxed) && self.core_alive()
    }

    pub(super) fn store_point(&self, x: i32, y: i32) {
        let packed = (u64::from(x.cast_unsigned()) << 32) | u64::from(y.cast_unsigned());
        self.last_point.store(packed, Ordering::Relaxed);
    }

    /// Last cursor position the mouse hook saw, in physical screen pixels.
    pub fn point(&self) -> (i32, i32) {
        let packed = self.last_point.load(Ordering::Relaxed);
        #[allow(clippy::cast_possible_truncation)] // unpacking two 32-bit halves.
        (
            ((packed >> 32) as u32).cast_signed(),
            (packed as u32).cast_signed(),
        )
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    /// Called by the pipeline thread on every loop iteration.
    pub fn heartbeat(&self) {
        self.core_heartbeat_ms
            .store(tick_count_64(), Ordering::Relaxed);
    }

    pub(super) fn core_alive(&self) -> bool {
        let last = self.core_heartbeat_ms.load(Ordering::Relaxed);
        tick_count_64().saturating_sub(last) < CORE_TIMEOUT_MS
    }

    pub(super) fn record_duration(&self, qpc_ticks: i64) {
        let index = self.duration_index.fetch_add(1, Ordering::Relaxed) % DURATION_SAMPLES;
        let ticks = u32::try_from(qpc_ticks.max(0)).unwrap_or(u32::MAX);
        self.durations[index].store(ticks, Ordering::Relaxed);
    }

    /// Recorded callback durations in QPC ticks (up to [`DURATION_SAMPLES`], unordered).
    pub fn durations(&self) -> Vec<u32> {
        let recorded = self
            .duration_index
            .load(Ordering::Relaxed)
            .min(DURATION_SAMPLES);
        self.durations[..recorded]
            .iter()
            .map(|sample| sample.load(Ordering::Relaxed))
            .collect()
    }

    /// Events the callback could not post because the channel was full.
    pub fn dropped_events(&self) -> u64 {
        self.dropped_events.load(Ordering::Relaxed)
    }

    /// `GetTickCount` of the newest event either hook saw.
    pub fn last_hook_tick(&self) -> u32 {
        let mouse = self.last_mouse_tick.load(Ordering::Relaxed);
        let keyboard = self.last_keyboard_tick.load(Ordering::Relaxed);
        if keyboard.wrapping_sub(mouse).cast_signed() > 0 {
            keyboard
        } else {
            mouse
        }
    }

    pub(super) fn mark_hooks_alive(&self, tick: u32) {
        self.last_mouse_tick.store(tick, Ordering::Relaxed);
        self.last_keyboard_tick.store(tick, Ordering::Relaxed);
    }
}
