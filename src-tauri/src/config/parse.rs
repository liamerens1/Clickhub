use serde_json::Value;

use super::{CONFIG_VERSION, Config, ConfigError};

/// Parses and validates raw config JSON. Never panics.
///
/// # Errors
///
/// [`ConfigError::Invalid`] if the JSON is malformed or does not match the schema,
/// [`ConfigError::NotAnObject`] if the root is not a JSON object,
/// [`ConfigError::UnsupportedVersion`] if `version` is not [`CONFIG_VERSION`].
pub fn parse(json: &str) -> Result<Config, ConfigError> {
    let value: Value = serde_json::from_str(json)?;
    // serde's derived Deserialize also accepts a JSON array for a struct
    // (`[1]` would become `{ version: 1 }`). Zod does not, so reject it here.
    if !value.is_object() {
        return Err(ConfigError::NotAnObject);
    }
    let config: Config = serde_json::from_value(value)?;
    if config.version != CONFIG_VERSION {
        return Err(ConfigError::UnsupportedVersion {
            found: config.version,
            expected: CONFIG_VERSION,
        });
    }
    Ok(config)
}
