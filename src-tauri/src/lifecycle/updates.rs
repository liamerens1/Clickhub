use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager, Url};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::{Update, UpdaterExt};

use super::autostart::installed_exe;
use crate::app::Runtime;
use crate::ipc;

/// The first automatic check waits this long, so it never competes with startup.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(30);
/// Automatic checks after that.
const CHECK_INTERVAL: Duration = Duration::from_hours(24);
/// Test aid: an `https` URL of a `latest.json` to check instead of the GitHub release (e.g. a
/// pre-release's). Updates are still verified against the built-in public key.
const ENDPOINT_ENV: &str = "CLICKHUB_UPDATE_ENDPOINT";
/// Download progress is reported at most this often (in percent steps).
const PROGRESS_STEP: u64 = 2;

/// Where updating stands. Sent to the settings window as `update:changed`; mirrors
/// `updateStateSchema` in `src/shared/ipc/settings-commands.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum UpdateState {
    Idle,
    Checking,
    UpToDate {
        checked_at_ms: u64,
    },
    Available {
        version: String,
        notes: Option<String>,
    },
    Downloading {
        downloaded: u64,
        total: Option<u64>,
    },
    /// The installer is starting; ClickHub exits and the installer restarts it.
    Installing,
    Failed {
        error: String,
    },
}

/// Checks GitHub Releases for a newer signed ClickHub and installs it when the user asks.
/// Never installs on its own. Managed as Tauri state.
pub struct Updates {
    inner: Mutex<Inner>,
}

struct Inner {
    state: UpdateState,
    /// The update found by the last check, kept so Install does not need to check again.
    found: Option<Update>,
    /// The version the user was last notified about, so each version notifies once.
    notified: Option<String>,
}

impl Updates {
    /// Starts the automatic checks for an installed copy: once shortly after startup, then
    /// daily. A build run from the repo only checks when asked.
    pub fn start(app: &AppHandle) {
        app.manage(Self {
            inner: Mutex::new(Inner {
                state: UpdateState::Idle,
                found: None,
                notified: None,
            }),
        });
        if installed_exe().is_none() {
            return;
        }
        let app = app.clone();
        let spawned = std::thread::Builder::new()
            .name("clickhub-updates".to_owned())
            .spawn(move || {
                std::thread::sleep(FIRST_CHECK_DELAY);
                loop {
                    if let Some(updates) = app.try_state::<Self>() {
                        tauri::async_runtime::block_on(updates.check(&app, false));
                    }
                    std::thread::sleep(CHECK_INTERVAL);
                }
            });
        if let Err(error) = spawned {
            tracing::error!(%error, "automatic update checks unavailable");
        }
    }

    pub fn state(&self) -> UpdateState {
        self.lock().state.clone()
    }

    /// Looks for a newer version. A check the user asked for reports failures; an automatic
    /// one only logs them (being offline is not worth a message).
    pub async fn check(&self, app: &AppHandle, asked: bool) -> UpdateState {
        if matches!(
            self.state(),
            UpdateState::Checking | UpdateState::Downloading { .. } | UpdateState::Installing
        ) {
            return self.state();
        }
        let before = self.state();
        self.set(app, UpdateState::Checking);
        let result = match updater(app) {
            Ok(updater) => updater.check().await.map_err(|error| match error {
                // Every endpoint failed: offline, GitHub unreachable, or nothing published yet.
                tauri_plugin_updater::Error::ReleaseNotFound => {
                    "couldn't reach the update server, or no release is published yet".to_owned()
                }
                other => other.to_string(),
            }),
            Err(error) => Err(error),
        };
        let next = match result {
            Ok(Some(update)) => {
                let version = update.version.clone();
                let notes = update.body.clone();
                tracing::info!(version, "update available");
                self.lock().found = Some(update);
                if !asked {
                    self.notify_once(app, &version);
                }
                UpdateState::Available { version, notes }
            }
            Ok(None) => {
                tracing::info!("no update available");
                self.lock().found = None;
                UpdateState::UpToDate {
                    checked_at_ms: now_ms(),
                }
            }
            Err(error) => {
                tracing::warn!(%error, asked, "update check failed");
                if asked {
                    UpdateState::Failed { error }
                } else {
                    before
                }
            }
        };
        self.set(app, next.clone());
        next
    }

    /// Downloads and installs the update found by the last check. On success the installer
    /// takes over: ClickHub stops its threads and exits, and the installer starts it again.
    ///
    /// # Errors
    ///
    /// A message when there is no update to install or the download or install fails.
    pub async fn install(&self, app: &AppHandle) -> Result<(), String> {
        let Some(update) = self.lock().found.take() else {
            return Err("there is no update to install; check for updates first".to_owned());
        };
        tracing::info!(version = update.version, "installing update");
        let mut downloaded = 0_u64;
        let mut reported = 0_u64;
        self.set(
            app,
            UpdateState::Downloading {
                downloaded: 0,
                total: None,
            },
        );
        let result = update
            .download_and_install(
                |chunk, total| {
                    downloaded += chunk as u64;
                    let percent = total.map_or(0, |total| downloaded * 100 / total.max(1));
                    if percent >= reported + PROGRESS_STEP {
                        reported = percent;
                        self.set(app, UpdateState::Downloading { downloaded, total });
                    }
                },
                || self.set(app, UpdateState::Installing),
            )
            .await;
        if let Err(error) = result {
            let error = error.to_string();
            tracing::error!(%error, "update failed");
            self.set(
                app,
                UpdateState::Failed {
                    error: error.clone(),
                },
            );
            return Err(error);
        }
        Ok(())
    }

    fn notify_once(&self, app: &AppHandle, version: &str) {
        let mut inner = self.lock();
        if inner.notified.as_deref() == Some(version) {
            return;
        }
        inner.notified = Some(version.to_owned());
        drop(inner);
        let shown = app
            .notification()
            .builder()
            .title("ClickHub update available")
            .body(format!(
                "Version {version} is ready. Open ClickHub settings to install it."
            ))
            .show();
        if let Err(error) = shown {
            tracing::warn!(%error, "could not show the update notification");
        }
    }

    fn set(&self, app: &AppHandle, state: UpdateState) {
        ipc::emit_update_changed(app, &state);
        self.lock().state = state;
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The updater for this app: the configured GitHub endpoint (or the test override), stopping
/// ClickHub's threads before the installer runs.
fn updater(app: &AppHandle) -> Result<tauri_plugin_updater::Updater, String> {
    let mut builder = app.updater_builder();
    if let Some(endpoint) = endpoint_override() {
        tracing::info!(%endpoint, "checking a test update endpoint");
        builder = builder
            .endpoints(vec![endpoint])
            .map_err(|error| error.to_string())?;
    }
    let handle = app.clone();
    builder
        .on_before_exit(move || {
            tracing::info!("exiting for the update installer");
            if let Some(runtime) = handle.try_state::<Runtime>() {
                runtime.stop();
            }
        })
        .build()
        .map_err(|error| error.to_string())
}

fn endpoint_override() -> Option<Url> {
    let raw = std::env::var(ENDPOINT_ENV).ok()?;
    match Url::parse(&raw) {
        Ok(url) if url.scheme() == "https" => Some(url),
        _ => {
            tracing::warn!(
                value = raw,
                "{ENDPOINT_ENV} must be an https URL; ignoring it"
            );
            None
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
