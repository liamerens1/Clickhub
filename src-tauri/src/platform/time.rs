use std::sync::OnceLock;

use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows::Win32::System::SystemInformation::{GetTickCount, GetTickCount64};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

/// High-resolution timestamp in QPC ticks. Cheap and allocation-free: safe in the hook callback.
#[inline]
pub fn qpc_now() -> i64 {
    let mut ticks = 0_i64;
    // SAFETY: `ticks` is a valid, writable i64. QPC cannot fail on Windows XP and later; if it
    // somehow did, 0 only distorts a latency measurement.
    let _ = unsafe { QueryPerformanceCounter(&raw mut ticks) };
    ticks
}

/// Converts a QPC tick delta to microseconds (0 for negative deltas).
pub fn qpc_to_micros(delta_ticks: i64) -> u64 {
    let frequency = qpc_frequency();
    let micros = i128::from(delta_ticks.max(0)) * 1_000_000 / i128::from(frequency);
    u64::try_from(micros).unwrap_or(u64::MAX)
}

fn qpc_frequency() -> i64 {
    static FREQUENCY: OnceLock<i64> = OnceLock::new();
    *FREQUENCY.get_or_init(|| {
        let mut frequency = 0_i64;
        // SAFETY: `frequency` is a valid, writable i64.
        let _ = unsafe { QueryPerformanceFrequency(&raw mut frequency) };
        // Never divide by zero, even if the call failed.
        frequency.max(1)
    })
}

/// Milliseconds since boot, wrapping every ~49.7 days. The same clock as `GetLastInputInfo`.
#[inline]
pub fn tick_count() -> u32 {
    // SAFETY: no arguments, no failure mode.
    unsafe { GetTickCount() }
}

/// Milliseconds since boot, non-wrapping.
#[inline]
pub fn tick_count_64() -> u64 {
    // SAFETY: no arguments, no failure mode.
    unsafe { GetTickCount64() }
}

/// [`tick_count`] of the most recent input event in this session, from any device and any
/// process, including input a hook never saw.
pub fn last_input_tick() -> Option<u32> {
    let mut info = LASTINPUTINFO {
        cbSize: u32::try_from(size_of::<LASTINPUTINFO>()).unwrap_or(u32::MAX),
        dwTime: 0,
    };
    // SAFETY: `info` is a valid LASTINPUTINFO with cbSize set, as the API requires.
    unsafe { GetLastInputInfo(&raw mut info) }
        .as_bool()
        .then_some(info.dwTime)
}
