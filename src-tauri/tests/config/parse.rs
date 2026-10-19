//! Mirrors `src/config/parse.rs`. Uses the same fixtures as
//! `tests/shared/schema/config-fixtures.test.ts`: both parsers must agree.

use std::fs;
use std::path::{Path, PathBuf};

use clickhub_lib::config::{
    self, CONFIG_VERSION, Config, Modifier, MouseButton, MouseTrigger, Trigger,
};

fn fixtures(kind: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/config")
        .join(kind);
    let entries = fs::read_dir(&dir)
        .and_then(Iterator::collect::<Result<Vec<_>, _>>)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()));
    let mut paths: Vec<PathBuf> = entries
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no fixtures in {}", dir.display());
    paths
}

#[test]
fn accepts_every_valid_fixture() {
    for path in fixtures("valid") {
        let json = fs::read_to_string(&path).expect("fixture is readable");
        let result = config::parse(&json);
        assert!(
            result.is_ok(),
            "{} should be valid: {result:?}",
            path.display()
        );
    }
}

#[test]
fn rejects_every_invalid_fixture() {
    for path in fixtures("invalid") {
        let json = fs::read_to_string(&path).expect("fixture is readable");
        let result = config::parse(&json);
        assert!(
            result.is_err(),
            "{} should be rejected: {result:?}",
            path.display()
        );
    }
}

#[test]
fn reports_unsupported_version_clearly() {
    let error = config::parse(r#"{ "version": 99 }"#).expect_err("version 99 is unsupported");
    let message = error.to_string();
    assert!(message.contains("99"), "{message}");
    assert!(message.contains(&CONFIG_VERSION.to_string()), "{message}");
}

#[test]
fn missing_trigger_defaults_to_x2_without_swallowing() {
    let config = config::parse(r#"{ "version": 1 }"#).expect("minimal config is valid");
    assert_eq!(config, Config::default());
    assert_eq!(
        config.trigger,
        Trigger::Mouse(MouseTrigger {
            button: MouseButton::X2,
            modifiers: Vec::new(),
            swallow: false,
        })
    );
}

#[test]
fn reads_modifiers_and_swallow() {
    let json = r#"{ "version": 1, "trigger": { "kind": "mouse", "button": "right",
        "modifiers": ["ctrl", "shift"], "swallow": true } }"#;
    let config = config::parse(json).expect("valid mouse trigger");
    assert_eq!(
        config.trigger,
        Trigger::Mouse(MouseTrigger {
            button: MouseButton::Right,
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            swallow: true,
        })
    );
}

#[test]
fn default_config_round_trips_through_json() {
    let json = serde_json::to_string(&Config::default()).expect("serializable");
    assert_eq!(config::parse(&json).expect("re-parses"), Config::default());
}
