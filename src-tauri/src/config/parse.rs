use serde_json::Value;

use super::{
    Action, CONFIG_VERSION, Config, ConfigError, MAX_LABEL_CHARS, MAX_SLOTS, MIN_SLOTS,
    MouseTrigger, Ring, Trigger,
};

/// Parses and validates raw config JSON. Never panics.
///
/// # Errors
///
/// - [`ConfigError::Invalid`] if the JSON is malformed or does not match the schema.
/// - [`ConfigError::NotAnObject`] if an object is expected but something else is found.
/// - [`ConfigError::UnsupportedVersion`] if `version` is not [`CONFIG_VERSION`].
/// - [`ConfigError::Rule`] (and the specific variants) for rule violations.
pub fn parse(json: &str) -> Result<Config, ConfigError> {
    let value: Value = serde_json::from_str(json)?;
    require_objects(&value)?;

    let config: Config = serde_json::from_value(value)?;
    if config.version != CONFIG_VERSION {
        return Err(ConfigError::UnsupportedVersion {
            found: config.version,
            expected: CONFIG_VERSION,
        });
    }
    validate_trigger(&config.trigger)?;
    validate_ring(&config.ring)?;
    Ok(config)
}

/// serde's derives also accept JSON arrays for structs and internally tagged enums (`[1]` would
/// become `{ version: 1 }`). Zod does not, so every place that expects an object is checked.
fn require_objects(root: &Value) -> Result<(), ConfigError> {
    let object_at = |value: Option<&Value>, name: &'static str| match value {
        Some(value) if !value.is_object() => Err(ConfigError::NotAnObject(name)),
        _ => Ok(()),
    };
    object_at(Some(root), "config")?;
    object_at(root.get("trigger"), "trigger")?;
    let ring = root.get("ring");
    object_at(ring, "ring")?;
    if let Some(slots) = ring
        .and_then(|ring| ring.get("slots"))
        .and_then(Value::as_array)
    {
        for slot in slots {
            object_at(Some(slot), "ring slot")?;
            object_at(slot.get("action"), "slot action")?;
        }
    }
    Ok(())
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

fn validate_ring(ring: &Ring) -> Result<(), ConfigError> {
    let count = ring.slots.len();
    if !(MIN_SLOTS..=MAX_SLOTS).contains(&count) {
        return Err(ConfigError::Rule(format!(
            "the ring needs {MIN_SLOTS} to {MAX_SLOTS} slots, found {count}"
        )));
    }
    for (index, slot) in ring.slots.iter().enumerate() {
        let number = index + 1;
        let label = slot.label.trim();
        if label.is_empty() {
            return Err(ConfigError::Rule(format!(
                "slot {number} has an empty label"
            )));
        }
        if label.chars().count() > MAX_LABEL_CHARS {
            return Err(ConfigError::Rule(format!(
                "slot {number} label is longer than {MAX_LABEL_CHARS} characters"
            )));
        }
        match &slot.action {
            Action::Launch(launch) if launch.path.trim().is_empty() => {
                return Err(ConfigError::Rule(format!(
                    "slot {number} launch path is empty"
                )));
            }
            Action::Url(url) if !is_web_url(&url.url) => {
                return Err(ConfigError::Rule(format!(
                    "slot {number} URL must start with http:// or https:// and contain no spaces"
                )));
            }
            Action::Shortcut(_) | Action::Launch(_) | Action::Url(_) => {}
        }
    }
    Ok(())
}

/// Same rule as the Zod schema: `^https?://\S+$`, case-insensitive.
fn is_web_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"));
    rest.is_some_and(|rest| !rest.is_empty() && !rest.chars().any(char::is_whitespace))
}
