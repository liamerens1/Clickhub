use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{Config, parse};

pub const CONFIG_FILE: &str = "config.json";
pub(super) const LAST_GOOD_FILE: &str = "config.last-good.json";

/// The config to run with, plus how it was obtained so the caller can report it.
#[derive(Debug, Clone)]
pub struct Loaded {
    pub config: Config,
    pub status: LoadStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadStatus {
    /// `config.json` was read and is valid.
    Loaded,
    /// There was no `config.json`; a default one was written (or `write_error` says why not).
    CreatedDefault { write_error: Option<String> },
    /// `config.json` could not be used. The broken file is left in place untouched, a copy is
    /// saved as `backup`, and the app runs on `fallback`.
    FellBack {
        error: String,
        backup: Option<PathBuf>,
        fallback: Fallback,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fallback {
    LastGood,
    Defaults,
}

/// Loads `config.json` from `dir`. Never fails and never panics: every problem becomes a
/// [`LoadStatus`] and the app always gets a usable config. Never overwrites the user's file.
pub fn load(dir: &Path) -> Loaded {
    let path = dir.join(CONFIG_FILE);
    let error = match fs::read_to_string(&path) {
        Ok(json) => match parse(&json) {
            Ok(config) => {
                // Best effort: losing the last-good copy only weakens a future fallback.
                let _ = fs::write(dir.join(LAST_GOOD_FILE), json);
                return Loaded {
                    config,
                    status: LoadStatus::Loaded,
                };
            }
            Err(error) => error.to_string(),
        },
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Loaded {
                config: Config::default(),
                status: LoadStatus::CreatedDefault {
                    write_error: write_default(dir, &path).err(),
                },
            };
        }
        Err(error) => format!("could not read {}: {error}", path.display()),
    };

    let backup = back_up(dir, &path);
    let (config, fallback) = match read_last_good(dir) {
        Some(config) => (config, Fallback::LastGood),
        None => (Config::default(), Fallback::Defaults),
    };
    Loaded {
        config,
        status: LoadStatus::FellBack {
            error,
            backup,
            fallback,
        },
    }
}

fn write_default(dir: &Path, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&Config::default()).map_err(|e| e.to_string())?;
    fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    fs::write(path, json + "\n").map_err(|e| format!("could not write {}: {e}", path.display()))
}

/// Copies the file at `path` to `config.invalid-<unix-seconds>.json` and returns the copy.
pub(super) fn back_up(dir: &Path, path: &Path) -> Option<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let backup = dir.join(format!("config.invalid-{stamp}.json"));
    fs::copy(path, &backup).ok().map(|_| backup)
}

fn read_last_good(dir: &Path) -> Option<Config> {
    let json = fs::read_to_string(dir.join(LAST_GOOD_FILE)).ok()?;
    parse(&json).ok()
}
