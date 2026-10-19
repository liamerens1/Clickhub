use thiserror::Error;

/// Why a config could not be used. Shown to the user, so messages are plain English.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config is not valid: {0}")]
    Invalid(#[from] serde_json::Error),
    #[error("config must be a JSON object at the top level")]
    NotAnObject,
    #[error("config version {found} is not supported (this build reads version {expected})")]
    UnsupportedVersion { found: u32, expected: u32 },
}
