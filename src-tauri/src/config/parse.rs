use serde_json::Value;

use super::{CONFIG_VERSION, Config, ConfigError, MouseTrigger, Trigger};

/// Parses and validates raw config JSON. Never panics.
///
/// # Errors
///
/// - [`ConfigError::Invalid`] if the JSON is malformed or does not match the schema.
/// - [`ConfigError::NotAnObject`] if the root or `trigger` is not a JSON object.
/// - [`ConfigError::UnsupportedVersion`] if `version` is not [`CONFIG_VERSION`].
/// - [`ConfigError::DuplicateModifier`] / [`ConfigError::EmptyShortcut`] for rule violations.
pub fn parse(json: &str) -> Result<Config, ConfigError> {
    let value: Value = serde_json::from_str(json)?;
    // serde's derives also accept JSON arrays for structs and internally tagged enums
    // (`[1]` would become `{ version: 1 }`). Zod does not, so reject them here.
    if !value.is_object() {
        return Err(ConfigError::NotAnObject("config"));
    }
    if value
        .get("trigger")
        .is_some_and(|trigger| !trigger.is_object())
    {
        return Err(ConfigError::NotAnObject("trigger"));
    }

    let config: Config = serde_json::from_value(value)?;
    if config.version != CONFIG_VERSION {
        return Err(ConfigError::UnsupportedVersion {
            found: config.version,
            expected: CONFIG_VERSION,
        });
    }
    validate_trigger(&config.trigger)?;
    Ok(config)
}

fn validate_trigger(trigger: &Trigger) -> Result<(), ConfigError> {
    match trigger {
        Trigger::Mouse(MouseTrigger { modifiers, .. }) => {
            for (index, modifier) in modifiers.iter().enumerate() {
                if modifiers[..index].contains(modifier) {
                    return Err(ConfigError::DuplicateModifier(*modifier));
                }
            }
            Ok(())
        }
        Trigger::Keyboard(keyboard) if keyboard.shortcut.trim().is_empty() => {
            Err(ConfigError::EmptyShortcut)
        }
        Trigger::Keyboard(_) => Ok(()),
    }
}
