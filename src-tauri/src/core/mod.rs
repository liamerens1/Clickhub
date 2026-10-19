//! Trigger matching, press classification and latency statistics. Pure logic: no Win32, no
//! Tauri, no I/O, no threads. Everything here is unit-testable.

mod button;
mod latency_stats;
mod modifier_set;
mod press_tracker;
mod trigger_filter;
mod watchdog_policy;

pub use button::Button;
pub use latency_stats::{LatencyStats, LatencySummary};
pub use modifier_set::ModifierSet;
pub use press_tracker::{HOLD_THRESHOLD_MS, Press, PressKind, PressTracker};
pub use trigger_filter::TriggerFilter;
pub use watchdog_policy::{BASE_BACKOFF_MS, MAX_BACKOFF_MS, MISSED_INPUT_MS, WatchdogPolicy};
