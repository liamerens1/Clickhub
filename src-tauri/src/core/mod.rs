//! Trigger matching, press classification, ring aim and session logic, latency statistics and
//! the watchdog policy. Pure logic: no Win32, no Tauri, no I/O, no threads. Everything here is
//! unit-testable.

mod button;
mod latency_stats;
mod modifier_set;
mod press_tracker;
mod ring_aim;
mod ring_ink;
mod ring_layout;
mod ring_session;
mod trigger_filter;
mod watchdog_policy;

pub use button::Button;
pub use latency_stats::{LatencyStats, LatencySummary};
pub use modifier_set::ModifierSet;
pub use press_tracker::{HOLD_THRESHOLD_MS, Press, PressKind, PressTracker};
pub use ring_aim::{Aim, aim, angular_distance};
pub use ring_ink::{
    CENTRE_SAMPLE_POINTS, DARK_INK_ABOVE, Ink, average_brightness, ink_for, label_sample_points,
};
pub use ring_layout::{
    CENTRED_BELOW, EDGE_MARGIN, LABEL_HALF_HEIGHT, LABEL_PUSH, LABEL_RADIUS, Layout,
    MAX_RADIUS_SCALE, Room, full_ring, label_reach, layout,
};
pub use ring_session::{IDLE_CLOSE_MS, RingEffect, RingInput, RingSession};
pub use trigger_filter::TriggerFilter;
pub use watchdog_policy::{BASE_BACKOFF_MS, MAX_BACKOFF_MS, MISSED_INPUT_MS, WatchdogPolicy};
