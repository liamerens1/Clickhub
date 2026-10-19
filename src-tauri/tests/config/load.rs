//! Mirrors `src/config/load.rs`. Each test works in its own fresh temp directory.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use clickhub_lib::config::{CONFIG_FILE, Config, Fallback, LoadStatus, Trigger, load};

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "clickhub-test-{name}-{}-{unique}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp dir is creatable");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const VALID_KEYBOARD: &str =
    r#"{ "version": 1, "trigger": { "kind": "keyboard", "shortcut": "Ctrl+Alt+Space" } }"#;

#[test]
fn creates_a_default_config_when_none_exists() {
    let dir = TempDir::new("create");
    let loaded = load(&dir.0);

    assert_eq!(loaded.config, Config::default());
    assert_eq!(
        loaded.status,
        LoadStatus::CreatedDefault { write_error: None }
    );
    let written = fs::read_to_string(dir.0.join(CONFIG_FILE)).expect("default was written");
    assert_eq!(
        clickhub_lib::config::parse(&written).expect("valid"),
        Config::default()
    );
}

#[test]
fn loads_a_valid_config() {
    let dir = TempDir::new("valid");
    fs::write(dir.0.join(CONFIG_FILE), VALID_KEYBOARD).expect("writable");

    let loaded = load(&dir.0);
    assert_eq!(loaded.status, LoadStatus::Loaded);
    assert!(matches!(loaded.config.trigger, Trigger::Keyboard(_)));
}

#[test]
fn falls_back_to_defaults_and_backs_up_an_invalid_config() {
    let dir = TempDir::new("invalid");
    let broken = r#"{ "version": 1, "trigger": { "kind": "mouse", "button": "left" } }"#;
    fs::write(dir.0.join(CONFIG_FILE), broken).expect("writable");

    let loaded = load(&dir.0);
    assert_eq!(loaded.config, Config::default());
    let LoadStatus::FellBack {
        error,
        backup,
        fallback,
    } = loaded.status
    else {
        panic!("expected a fallback, got {:?}", loaded.status);
    };
    assert_eq!(fallback, Fallback::Defaults);
    assert!(error.contains("left"), "error names the bad value: {error}");

    let backup = backup.expect("a backup was made");
    assert_eq!(fs::read_to_string(backup).expect("backup readable"), broken);
    // The user's file is never overwritten.
    assert_eq!(
        fs::read_to_string(dir.0.join(CONFIG_FILE)).expect("original readable"),
        broken
    );
}

#[test]
fn falls_back_to_the_last_good_config() {
    let dir = TempDir::new("last-good");
    fs::write(dir.0.join(CONFIG_FILE), VALID_KEYBOARD).expect("writable");
    assert_eq!(load(&dir.0).status, LoadStatus::Loaded);

    fs::write(dir.0.join(CONFIG_FILE), "{ not json").expect("writable");
    let loaded = load(&dir.0);

    assert!(matches!(loaded.config.trigger, Trigger::Keyboard(_)));
    assert!(matches!(
        loaded.status,
        LoadStatus::FellBack {
            fallback: Fallback::LastGood,
            ..
        }
    ));
}
