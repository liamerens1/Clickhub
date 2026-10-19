//! Mirrors `src/config/save.rs`. Each test works in its own fresh temp directory.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use clickhub_lib::config::{
    Action, CONFIG_FILE, Config, ConfigError, LoadStatus, Slot, UrlAction, load, save,
};

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "clickhub-save-test-{name}-{}-{unique}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp dir is creatable");
        Self(path)
    }

    fn files(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.0)
            .expect("temp dir is readable")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn edited() -> Config {
    let mut config = Config::default();
    config.ring.slots.push(Slot {
        label: "Docs".to_owned(),
        action: Action::Url(UrlAction {
            url: "https://example.com/docs".to_owned(),
        }),
    });
    config
}

#[test]
fn saved_config_loads_back_unchanged() {
    let dir = TempDir::new("roundtrip");
    save(&dir.0, &edited()).expect("valid config saves");

    let loaded = load(&dir.0);
    assert_eq!(loaded.status, LoadStatus::Loaded);
    assert_eq!(loaded.config, edited());
}

#[test]
fn creates_the_directory_and_leaves_no_temp_file() {
    let dir = TempDir::new("create");
    let nested = dir.0.join("nested");
    save(&nested, &Config::default()).expect("saves into a new directory");

    let mut names: Vec<String> = fs::read_dir(&nested)
        .expect("created")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    assert_eq!(names, ["config.json", "config.last-good.json"]);
}

#[test]
fn rejects_a_config_that_breaks_a_rule_and_writes_nothing() {
    let dir = TempDir::new("reject");
    let mut config = Config::default();
    config.ring.slots.truncate(1);

    let error = save(&dir.0, &config).expect_err("one slot is not a ring");
    assert!(matches!(error, ConfigError::Rule(_)), "{error}");
    assert!(dir.files().is_empty());
}

#[test]
fn replacing_a_valid_file_makes_no_backup() {
    let dir = TempDir::new("replace-valid");
    save(&dir.0, &Config::default()).expect("first save");
    save(&dir.0, &edited()).expect("second save");

    assert_eq!(dir.files(), ["config.json", "config.last-good.json"]);
}

#[test]
fn backs_up_an_invalid_file_before_replacing_it() {
    let dir = TempDir::new("replace-invalid");
    let broken = r#"{ "version": 1, "ring": { "slots": [] } }"#;
    fs::write(dir.0.join(CONFIG_FILE), broken).expect("write broken file");

    save(&dir.0, &edited()).expect("saves over the broken file");

    let backups: Vec<String> = dir
        .files()
        .into_iter()
        .filter(|name| name.starts_with("config.invalid-"))
        .collect();
    assert_eq!(backups.len(), 1, "{:?}", dir.files());
    let kept = fs::read_to_string(dir.0.join(&backups[0])).expect("backup readable");
    assert_eq!(kept, broken);
    assert_eq!(load(&dir.0).config, edited());
}
