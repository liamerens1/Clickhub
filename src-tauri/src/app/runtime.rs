use std::path::{Path, PathBuf};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_notification::NotificationExt;

use super::fullscreen_watch::FullscreenWatch;
use super::pipeline::{Pipeline, PipelineCommand};
use crate::config::{self, Config, ConfigError, Fallback, LoadStatus, Loaded, Trigger};
use crate::core::TriggerFilter;
use crate::hook::{HookThread, SHARED, Watchdog};
use crate::ipc;
use crate::overlay::{Overlays, RingView};
use crate::tray;

/// Hook events buffered between the callback and the pipeline. Preallocated; if it ever fills,
/// the callback drops events (counted) rather than block.
const EVENT_QUEUE: usize = 1_024;
const STALL_MS: u64 = 5_000;

/// Owns the background threads and the current config. Managed as Tauri state.
pub struct Runtime {
    config_dir: PathBuf,
    overlays: Arc<Overlays>,
    inner: Mutex<Inner>,
}

struct Inner {
    hook: Option<HookThread>,
    watchdog: Option<Watchdog>,
    fullscreen: Option<FullscreenWatch>,
    pipeline: Option<Pipeline>,
    /// A problem that config reloads cannot fix (e.g. the hook could not be installed).
    hook_problem: Option<String>,
    config_problem: Option<String>,
    /// The config in use and how it was obtained (shown in settings).
    loaded: Option<Loaded>,
    /// Paused by the user (tray or settings).
    paused: bool,
    /// The settings window is listening for a new trigger: the current one is switched off so
    /// the press reaches the window instead of opening the ring.
    capturing: bool,
}

impl Runtime {
    /// Starts the pipeline, hook thread and watchdog, then loads the config.
    ///
    /// # Errors
    ///
    /// Fails if the config directory cannot be resolved or the pipeline thread cannot start.
    /// A hook that cannot be installed is reported in the tray instead of failing startup.
    pub fn start(
        app: &AppHandle,
        overlays: Arc<Overlays>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = app.path().app_config_dir()?;
        let (sender, receiver) = sync_channel(EVENT_QUEUE);
        SHARED.heartbeat();
        let pipeline = Pipeline::spawn(receiver, Arc::clone(&overlays))?;

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
        let fullscreen = FullscreenWatch::spawn()
            .inspect_err(|error| tracing::error!(%error, "full-screen detection unavailable"))
            .ok();

        let runtime = Self {
            config_dir,
            overlays,
            inner: Mutex::new(Inner {
                hook,
                watchdog,
                fullscreen,
                pipeline: Some(pipeline),
                hook_problem,
                config_problem: None,
                loaded: None,
                paused: false,
                capturing: false,
            }),
        };
        runtime.apply_config(app);
        Ok(runtime)
    }

    /// (Re)loads `config.json` and applies the trigger and ring. Never fails: problems are logged and
    /// shown in the tray, and the app keeps running on the fallback config.
    pub fn apply_config(&self, app: &AppHandle) {
        let loaded = config::load(&self.config_dir);
        let mut problem = report_load_status(&loaded.status, &self.config_dir);

        SHARED.set_filter(TriggerFilter::from_trigger(&loaded.config.trigger));
        let capturing = self.lock().capturing;
        if !capturing
            && let Some(trigger_problem) = register_keyboard_trigger(app, &loaded.config.trigger)
        {
            problem = Some(trigger_problem);
        }

        self.overlays
            .set_view(RingView::from_ring(&loaded.config.ring));
        let actions = loaded
            .config
            .ring
            .slots
            .iter()
            .map(|slot| slot.action.clone())
            .collect();
        tracing::info!(slots = loaded.config.ring.slots.len(), "ring configured");
        if let LoadStatus::FellBack { error, .. } = &loaded.status {
            notify(
                app,
                "ClickHub config error",
                &format!(
                    "{error}. Your file was left unchanged; ClickHub is using the last good config or the defaults."
                ),
            );
        }
        let mut inner = self.lock();
        if let Some(pipeline) = &inner.pipeline {
            pipeline.send(PipelineCommand::SetRing(actions));
        }
        inner.config_problem = problem;
        inner.loaded = Some(loaded);
        drop(inner);
        self.show_status(app);
        ipc::emit_config_changed(app);
    }

    /// Validates and writes `config` (from the settings window), then applies it.
    ///
    /// # Errors
    ///
    /// The [`ConfigError`] if `config` breaks a rule or cannot be written; nothing is applied.
    pub fn save_config(&self, app: &AppHandle, config: &Config) -> Result<(), ConfigError> {
        config::save(&self.config_dir, config)?;
        tracing::info!(
            slots = config.ring.slots.len(),
            "config saved from settings"
        );
        self.apply_config(app);
        Ok(())
    }

    /// The config in use and how it was obtained.
    pub fn loaded(&self) -> Option<Loaded> {
        self.lock().loaded.clone()
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join(config::CONFIG_FILE)
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn toggle_pause(&self, app: &AppHandle) {
        let paused = !self.lock().paused;
        self.set_paused(app, paused);
    }

    pub fn set_paused(&self, app: &AppHandle, paused: bool) {
        let mut inner = self.lock();
        inner.paused = paused;
        SHARED.set_paused(inner.paused || inner.capturing);
        drop(inner);
        tracing::info!(paused, "trigger pause set");
        self.show_status(app);
    }

    /// Switches the trigger off while the settings window listens for a new one, so the press
    /// reaches the window (a paused hook passes every button through; an unregistered keyboard
    /// shortcut reaches the focused window). Switching it back on restores the user's pause
    /// state and the configured keyboard trigger.
    pub fn set_capturing(&self, app: &AppHandle, capturing: bool) {
        let mut inner = self.lock();
        if inner.capturing == capturing {
            return;
        }
        inner.capturing = capturing;
        SHARED.set_paused(inner.paused || capturing);
        let trigger = inner
            .loaded
            .as_ref()
            .map(|loaded| loaded.config.trigger.clone());
        drop(inner);
        tracing::info!(capturing, "trigger capture");
        if capturing {
            if let Err(error) = app.global_shortcut().unregister_all() {
                tracing::warn!(%error, "could not switch off the keyboard trigger for capture");
            }
        } else if let Some(trigger) = trigger
            && let Some(problem) = register_keyboard_trigger(app, &trigger)
        {
            self.lock().config_problem = Some(problem);
            self.show_status(app);
        }
    }

    /// Whether the user paused the trigger, and the problem shown in the tray, if any.
    pub fn status(&self) -> (bool, Option<String>) {
        let inner = self.lock();
        let problem = inner
            .hook_problem
            .clone()
            .or_else(|| inner.config_problem.clone());
        (inner.paused, problem)
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
            pipeline.send(PipelineCommand::Stall(STALL_MS));
        }
    }

    /// Stops the watchdog, removes the hooks, then stops the pipeline (in that order, so the
    /// watchdog never re-installs a hook that is being removed).
    pub fn stop(&self) {
        let mut inner = self.lock();
        if let Some(watchdog) = inner.watchdog.take() {
            watchdog.stop();
        }
        if let Some(fullscreen) = inner.fullscreen.take() {
            fullscreen.stop();
        }
        if let Some(hook) = inner.hook.take() {
            hook.stop();
        }
        if let Some(pipeline) = inner.pipeline.take() {
            pipeline.stop();
        }
    }

    fn show_status(&self, app: &AppHandle) {
        let (paused, problem) = self.status();
        tray::show_status(app, paused, problem.as_deref());
        ipc::emit_status_changed(app, paused, problem);
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn send(&self, command: PipelineCommand) {
        if let Some(pipeline) = &self.lock().pipeline {
            pipeline.send(command);
        }
    }
}

/// Replaces any registered keyboard trigger with `trigger`'s, if it is one. Returns a short tray
/// message if the shortcut could not be registered.
fn register_keyboard_trigger(app: &AppHandle, trigger: &Trigger) -> Option<String> {
    if let Err(error) = app.global_shortcut().unregister_all() {
        tracing::warn!(%error, "could not clear the previous keyboard trigger");
    }
    match trigger {
        Trigger::Mouse(mouse) => {
            tracing::info!(
                button = ?mouse.button,
                modifiers = ?mouse.modifiers,
                swallow = mouse.swallow,
                "mouse trigger active"
            );
            None
        }
        Trigger::Keyboard(keyboard) => {
            let registered = app.global_shortcut().on_shortcut(
                keyboard.shortcut.as_str(),
                |app, _shortcut, event| {
                    on_keyboard_trigger(app, event.state());
                },
            );
            if let Err(error) = registered {
                tracing::error!(shortcut = keyboard.shortcut, %error, "keyboard trigger unavailable");
                Some("keyboard trigger unavailable (see log)".to_owned())
            } else {
                tracing::info!(shortcut = keyboard.shortcut, "keyboard trigger active");
                None
            }
        }
    }
}

fn on_keyboard_trigger(app: &AppHandle, state: ShortcutState) {
    let Some(runtime) = app.try_state::<Runtime>() else {
        return;
    };
    if SHARED.is_paused() || SHARED.is_suppressed() {
        return;
    }
    runtime.send(PipelineCommand::KeyboardTrigger {
        pressed: state == ShortcutState::Pressed,
    });
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

/// Shows a Windows notification. Failures are logged only: the tray tooltip still shows the
/// problem.
fn notify(app: &AppHandle, title: &str, body: &str) {
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        tracing::warn!(%error, "could not show a notification");
    }
}
