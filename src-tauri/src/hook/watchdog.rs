use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::shared::SHARED;
use crate::core::WatchdogPolicy;
use crate::platform::{foreground_app, last_input_tick, tick_count_64};

/// With two-poll confirmation, a lost hook is repaired within about 0.5 s of the next input.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Background thread that re-installs the hooks when Windows silently drops them.
/// See [`WatchdogPolicy`] for how loss is detected.
pub struct Watchdog {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl Watchdog {
    /// Starts polling. `reinstall` is called on the watchdog thread when the hooks look lost.
    ///
    /// # Errors
    ///
    /// Fails only if the OS cannot start a thread.
    pub fn spawn(reinstall: impl Fn() + Send + 'static) -> std::io::Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let join = thread::Builder::new()
            .name("clickhub-watchdog".into())
            .spawn(move || run(&stop_flag, &reinstall))?;
        Ok(Self {
            stop,
            join: Some(join),
        })
    }

    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take()
            && join.join().is_err()
        {
            tracing::error!("watchdog thread panicked");
        }
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run(stop: &AtomicBool, reinstall: &dyn Fn()) {
    let mut policy = WatchdogPolicy::default();
    let mut reinstalls = 0_u64;
    while !stop.load(Ordering::Relaxed) {
        thread::sleep(POLL_INTERVAL);
        let Some(last_input) = last_input_tick() else {
            continue;
        };
        let hook_last = SHARED.last_hook_tick();
        if policy.should_reinstall(tick_count_64(), last_input, hook_last) {
            reinstalls += 1;
            // Diagnostics: input to an elevated window is invisible to our hooks (UIPI), so
            // knowing the foreground app tells a real drop apart from that.
            let foreground = foreground_app();
            tracing::warn!(
                reinstalls,
                missed_ms = last_input.wrapping_sub(hook_last),
                next_backoff_ms = policy.backoff_ms(),
                foreground_exe = foreground.as_ref().and_then(|app| app.exe.as_deref()),
                foreground_pid = foreground.as_ref().map(|app| app.pid),
                foreground_elevated = ?foreground.as_ref().map(|app| app.elevated),
                "hooks missed input; re-installing"
            );
            reinstall();
        }
    }
}
