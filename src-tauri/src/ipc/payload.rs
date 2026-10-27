use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::{Config, Fallback, LoadStatus, Loaded};

/// The config in use, as the settings window sees it. Mirrors `configSnapshotSchema` in
/// `src/shared/ipc/settings-commands.ts`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSnapshot {
    pub config: Config,
    pub path: String,
    /// Why `config.json` is not what is running, if it is not.
    pub problem: Option<ConfigProblem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigProblem {
    pub error: String,
    /// Where the broken file was copied to, if it was.
    pub backup: Option<String>,
    pub fallback: FallbackKind,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FallbackKind {
    LastGood,
    Defaults,
}

impl ConfigSnapshot {
    pub fn new(loaded: Loaded, path: &Path) -> Self {
        let problem = match loaded.status {
            LoadStatus::Loaded | LoadStatus::CreatedDefault { write_error: None } => None,
            LoadStatus::CreatedDefault {
                write_error: Some(error),
            } => Some(ConfigProblem {
                error,
                backup: None,
                fallback: FallbackKind::Defaults,
            }),
            LoadStatus::FellBack {
                error,
                backup,
                fallback,
            } => Some(ConfigProblem {
                error,
                backup: backup.map(|backup| backup.display().to_string()),
                fallback: match fallback {
                    Fallback::LastGood => FallbackKind::LastGood,
                    Fallback::Defaults => FallbackKind::Defaults,
                },
            }),
        };
        Self {
            config: loaded.config,
            path: path.display().to_string(),
            problem,
        }
    }
}

/// Pause state and the problem shown in the tray. Mirrors `appStatusSchema`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub paused: bool,
    pub problem: Option<String>,
}

/// Static facts for the General and About pages. Mirrors `appInfoSchema`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    /// Installed with the installer (so it can start with Windows, update and uninstall).
    pub installed: bool,
    pub config_path: String,
    pub log_dir: Option<String>,
}

/// Folders the settings window may ask Windows to open. Nothing else can be opened from the
/// frontend.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Folder {
    Config,
    Logs,
}
