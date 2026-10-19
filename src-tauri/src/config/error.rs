use thiserror::Error;

use super::Modifier;

/// Why a config could not be used. Shown to the user, so messages are plain English.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config is not valid: {0}")]
    Invalid(#[from] serde_json::Error),
    #[error("`{0}` must be a JSON object")]
    NotAnObject(&'static str),
    #[error("config version {found} is not supported (this build reads version {expected})")]
    UnsupportedVersion { found: u32, expected: u32 },
    #[error("trigger modifier `{0:?}` is listed more than once")]
    DuplicateModifier(Modifier),
    #[error("keyboard trigger shortcut must not be empty")]
    EmptyShortcut,
}
