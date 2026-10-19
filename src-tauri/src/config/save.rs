use std::fs;
use std::path::Path;

use super::load::{LAST_GOOD_FILE, back_up};
use super::{CONFIG_FILE, Config, ConfigError, parse};

/// Validates `config` and writes it to `config.json` in `dir`.
///
/// The JSON is checked with the same [`parse`] that loading uses, so a saved file always loads.
/// The write goes to a temporary file first and then replaces `config.json`, so a crash
/// mid-write never leaves a half-written config. If the file being replaced is itself invalid
/// (the user's hand edit that the app fell back from), it is backed up first, never silently
/// overwritten.
///
/// # Errors
///
/// - Any [`parse`] error if `config` breaks a rule (nothing is written).
/// - [`ConfigError::Write`] if the directory or file cannot be written.
pub fn save(dir: &Path, config: &Config) -> Result<(), ConfigError> {
    let json = serde_json::to_string_pretty(config)? + "\n";
    parse(&json)?;

    let write_error = |path: &Path| {
        let path = path.display().to_string();
        move |source| ConfigError::Write { path, source }
    };
    fs::create_dir_all(dir).map_err(write_error(dir))?;
    let path = dir.join(CONFIG_FILE);
    let replaces_invalid = fs::read_to_string(&path).is_ok_and(|old| parse(&old).is_err());
    if replaces_invalid && back_up(dir, &path).is_none() {
        tracing::warn!(path = %path.display(), "could not back up the invalid config before saving");
    }

    let temp = dir.join(format!("{CONFIG_FILE}.tmp"));
    fs::write(&temp, &json).map_err(write_error(&temp))?;
    // Replaces an existing file in one step (MoveFileEx with MOVEFILE_REPLACE_EXISTING).
    fs::rename(&temp, &path).map_err(write_error(&path))?;
    // Best effort: losing the last-good copy only weakens a future fallback.
    let _ = fs::write(dir.join(LAST_GOOD_FILE), &json);
    Ok(())
}
