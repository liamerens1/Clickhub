use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::platform;

/// Name of the per-user Run value. Must equal `productName` in `tauri.conf.json`: the NSIS
/// uninstaller deletes `HKCU\...\Run\${PRODUCTNAME}` (and keeps it during updates).
const RUN_VALUE: &str = "ClickHub";
/// Passed when Windows starts ClickHub at sign-in (it starts hidden in the tray either way).
const AUTOSTART_ARG: &str = "--autostart";
/// Written once the "on by default" choice has been applied, so it never overrides the user.
const DEFAULT_APPLIED_MARKER: &str = "autostart-default-applied";

/// Whether ClickHub can start with Windows, and whether it does. Mirrors `autostartSchema` in
/// `src/shared/ipc/settings-commands.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutostartState {
    /// Only an installed copy registers itself; a build run from the repo never does.
    pub available: bool,
    pub enabled: bool,
}

/// The running exe, if it is an installed copy (the NSIS installer puts `uninstall.exe` beside
/// it).
pub fn installed_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    dir.join("uninstall.exe").is_file().then_some(exe)
}

/// Reads whether ClickHub starts at sign-in.
///
/// # Errors
///
/// A message when the registry cannot be read.
pub fn state() -> Result<AutostartState, String> {
    Ok(AutostartState {
        available: installed_exe().is_some(),
        enabled: platform::run_entry_active(RUN_VALUE)?,
    })
}

/// Switches starting at sign-in on or off for this user.
///
/// # Errors
///
/// A message when this copy is not installed (switching on) or the registry cannot be written.
pub fn set_enabled(enabled: bool) -> Result<AutostartState, String> {
    if enabled {
        let exe =
            installed_exe().ok_or("only an installed copy of ClickHub can start with Windows")?;
        platform::set_run_entry(RUN_VALUE, &format!("\"{}\" {AUTOSTART_ARG}", exe.display()))?;
    } else {
        platform::remove_run_entry(RUN_VALUE)?;
    }
    tracing::info!(enabled, "start with Windows set");
    state()
}

/// On the first run of an installed copy, switches starting at sign-in on (the chosen default),
/// then never again: after that only the user decides. On later runs an active entry is
/// rewritten so it points at this exe even if the install moved. Never fails startup.
pub fn apply_default(state_dir: &Path) {
    if installed_exe().is_none() {
        return;
    }
    let marker = state_dir.join(DEFAULT_APPLIED_MARKER);
    let first_run = !marker.exists();
    let enable = first_run || platform::run_entry_active(RUN_VALUE).unwrap_or(false);
    if enable && let Err(error) = set_enabled(true) {
        tracing::warn!(%error, "could not register ClickHub to start with Windows");
    }
    if first_run {
        let written = fs::create_dir_all(state_dir).and_then(|()| fs::write(&marker, b""));
        if let Err(error) = written {
            tracing::warn!(%error, "could not record that the autostart default was applied");
        }
    }
}
