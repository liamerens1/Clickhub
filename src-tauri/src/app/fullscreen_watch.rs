use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::hook::SHARED;
use crate::platform::{foreground_app, foreground_is_fullscreen};

/// Full-screen state changes rarely; a quarter second keeps the check cheap and quick enough
/// that a game is suppressed before anyone reaches for the trigger.
const POLL_INTERVAL: Duration = Duration::from_millis(250);

/// Polls whether the foreground app is full-screen and sets [`SHARED`]'s suppression flag, so the
/// hook passes the trigger through untouched in games and videos.
pub struct FullscreenWatch {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl FullscreenWatch {
    /// # Errors
    ///
    /// Fails only if the OS cannot start a thread.
    pub fn spawn() -> std::io::Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let join = thread::Builder::new()
            .name("clickhub-fullscreen".into())
            .spawn(move || run(&flag))?;
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
            tracing::error!("full-screen watch thread panicked");
        }
    }
}

impl Drop for FullscreenWatch {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run(stop: &AtomicBool) {
    let mut last = false;
    while !stop.load(Ordering::Relaxed) {
        let fullscreen = foreground_is_fullscreen();
        if fullscreen != last {
            last = fullscreen;
            SHARED.set_suppressed(fullscreen);
            let app = foreground_app();
            tracing::info!(
                fullscreen,
                foreground_exe = app.as_ref().and_then(|app| app.exe.as_deref()),
                "full-screen state changed; trigger {}",
                if fullscreen { "suppressed" } else { "active" }
            );
        }
        thread::sleep(POLL_INTERVAL);
    }
    SHARED.set_suppressed(false);
}
