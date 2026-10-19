use std::path::PathBuf;
use std::sync::mpsc::sync_channel;
use std::sync::{Mutex, MutexGuard, PoisonError};

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use super::pipeline::Pipeline;
use crate::config::{self, Fallback, LoadStatus, Trigger};
use crate::core::{PressTracker, TriggerFilter};
use crate::hook::{HookThread, SHARED, Watchdog};
use crate::platform::tick_count_64;
use crate::tray;

/// Hook events buffered between the callback and the pipeline. Preallocated; if it ever fills,
/// the callback drops events (counted) rather than block.
const EVENT_QUEUE: usize = 1_024;
const STALL_MS: u64 = 5_000;

/// Owns the background threads and the current config. Managed as Tauri state.
pub struct Runtime {
    config_dir: PathBuf,
    inner: Mutex<Inner>,
    keyboard_press: Mutex<PressTracker>,
}

struct Inner {
    hook: Option<HookThread>,
    watchdog: Option<Watchdog>,
    pipeline: Option<Pipeline>,
    /// A problem that config reloads cannot fix (e.g. the hook could not be installed).
    hook_problem: Option<String>,
    config_problem: Option<String>,
}

impl Runtime {
    /// Starts the pipeline, hook thread and watchdog, then loads the config.
    ///
    /// # Errors
    ///
    /// Fails if the config directory cannot be resolved or the pipeline thread cannot start.
    /// A hook that cannot be installed is reported in the tray instead of failing startup.
    pub fn start(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = app.path().app_config_dir()?;
        let (sender, receiver) = sync_channel(EVENT_QUEUE);
        SHARED.heartbeat();
        let pipeline = Pipeline::spawn(receiver)?;

        let mut hook_problem = None;
        let hook = match HookThread::start(sender) {
            Ok(hook) => Some(hook),
            Err(error) => {
                tracing::error!(%error, "mouse trigger unavailable");
                hook_problem = Some("mouse trigger unavailable (see log)".to_owned());
                None
            }
        };
        let watchdog = hook.as_ref().and_then(|hook| {
            let remote = hook.remote();
            Watchdog::spawn(move || remote.request_reinstall())
                .inspect_err(|error| tracing::error!(%error, "hook watchdog unavailable"))
                .ok()
        });

        let runtime = Self {
            config_dir,
            inner: Mutex::new(Inner {
                hook,
                watchdog,
                pipeline: Some(pipeline),
                hook_problem,
                config_problem: None,
            }),
            keyboard_press: Mutex::new(PressTracker::default()),
        };
        runtime.apply_config(app);
        Ok(runtime)
    }

    /// (Re)loads `config.json` and applies the trigger. Never fails: problems are logged and
    /// shown in the tray, and the app keeps running on the fallback config.
    pub fn apply_config(&self, app: &AppHandle) {
        let loaded = config::load(&self.config_dir);
        let mut problem = report_load_status(&loaded.status, &self.config_dir);

        SHARED.set_filter(TriggerFilter::from_trigger(&loaded.config.trigger));
        if let Err(error) = app.global_shortcut().unregister_all() {
            tracing::warn!(%error, "could not clear the previous keyboard trigger");
        }
        match &loaded.config.trigger {
            Trigger::Mouse(mouse) => tracing::info!(
                button = ?mouse.button,
                modifiers = ?mouse.modifiers,
                swallow = mouse.swallow,
                "mouse trigger active"
            ),
            Trigger::Keyboard(keyboard) => {
                let registered = app.global_shortcut().on_shortcut(
                    keyboard.shortcut.as_str(),
                    |app, _shortcut, event| {
                        on_keyboard_trigger(app, event.state());
                    },
                );
                if let Err(error) = registered {
                    tracing::error!(shortcut = keyboard.shortcut, %error, "keyboard trigger unavailable");
                    problem = Some("keyboard trigger unavailable (see log)".to_owned());
                } else {
                    tracing::info!(shortcut = keyboard.shortcut, "keyboard trigger active");
                }
            }
        }

        self.lock().config_problem = problem;
        self.show_status(app);
    }

    pub fn toggle_pause(&self, app: &AppHandle) {
        let paused = !SHARED.is_paused();
        SHARED.set_paused(paused);
        if paused {
            self.keyboard_press().reset();
        }
        tracing::info!(paused, "trigger pause toggled");
        self.show_status(app);
    }

    pub fn simulate_hook_loss(&self) {
        if let Some(hook) = &self.lock().hook {
            hook.remote().simulate_loss();
        } else {
            tracing::warn!("no hook to remove");
        }
    }

    pub fn stall_core(&self) {
        if let Some(pipeline) = &self.lock().pipeline {
            pipeline.stall_for(STALL_MS);
        }
    }

    /// Stops the watchdog, removes the hooks, then stops the pipeline (in that order, so the
    /// watchdog never re-installs a hook that is being removed).
    pub fn stop(&self) {
        let mut inner = self.lock();
        if let Some(watchdog) = inner.watchdog.take() {
            watchdog.stop();
        }
        if let Some(hook) = inner.hook.take() {
            hook.stop();
        }
        if let Some(pipeline) = inner.pipeline.take() {
            pipeline.stop();
        }
    }

    fn show_status(&self, app: &AppHandle) {
        let inner = self.lock();
        let problem = inner
            .hook_problem
            .as_deref()
            .or(inner.config_problem.as_deref());
        tray::show_status(app, SHARED.is_paused(), problem);
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn keyboard_press(&self) -> MutexGuard<'_, PressTracker> {
        self.keyboard_press
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

fn on_keyboard_trigger(app: &AppHandle, state: ShortcutState) {
    let Some(runtime) = app.try_state::<Runtime>() else {
        return;
    };
    if SHARED.is_paused() {
        return;
    }
    let mut tracker = runtime.keyboard_press();
    match state {
        ShortcutState::Pressed => {
            tracker.on_down(tick_count_64());
            tracing::info!("keyboard trigger pressed");
        }
        ShortcutState::Released => {
            if let Some(press) = tracker.on_up(tick_count_64()) {
                tracing::info!(kind = ?press.kind, held_ms = press.held_ms, "keyboard trigger released");
            }
        }
    }
}

/// Logs how the config was obtained and returns a short tray message for problems.
fn report_load_status(status: &LoadStatus, dir: &std::path::Path) -> Option<String> {
    let path = dir.join(config::CONFIG_FILE);
    match status {
        LoadStatus::Loaded => {
            tracing::info!(path = %path.display(), "config loaded");
            None
        }
        LoadStatus::CreatedDefault { write_error: None } => {
            tracing::info!(path = %path.display(), "no config found; wrote the defaults");
            None
        }
        LoadStatus::CreatedDefault {
            write_error: Some(error),
        } => {
            tracing::warn!(path = %path.display(), %error, "no config found and could not write one; using defaults");
            None
        }
        LoadStatus::FellBack {
            error,
            backup,
            fallback,
        } => {
            let using = match fallback {
                Fallback::LastGood => "the last good config",
                Fallback::Defaults => "defaults",
            };
            tracing::error!(
                path = %path.display(),
                %error,
                backup = ?backup.as_ref().map(|backup| backup.display().to_string()),
                "config is invalid; running on {using}. Your file was left unchanged"
            );
            Some(format!("config error, using {using} (see log)"))
        }
    }
}
