use serde::{Deserialize, Serialize};

use super::Shortcut;

/// Slot count limits. Fewer than 2 is not a ring; more than 12 stops being readable.
pub const MIN_SLOTS: usize = 2;
pub const MAX_SLOTS: usize = 12;
/// Longest label, in Unicode code points.
pub const MAX_LABEL_CHARS: usize = 32;

/// The ring's slots, clockwise from the top. Mirrors `ringSchema` in
/// `src/shared/schema/ring.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ring {
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Slot {
    pub label: String,
    pub action: Action,
}

/// What a slot does when picked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Action {
    /// Sends a keyboard chord to the app in front.
    Shortcut(ShortcutAction),
    /// Starts an app or opens a file or folder.
    Launch(LaunchAction),
    /// Opens an `http(s)` URL in the default browser.
    Url(UrlAction),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShortcutAction {
    pub keys: Shortcut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LaunchAction {
    pub path: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UrlAction {
    pub url: String,
}

impl Default for Ring {
    /// The everyday set chosen for v1 (Undo and Task Manager removed at the user's request;
    /// slots become editable in the Phase 3 settings window).
    fn default() -> Self {
        let shortcut = |label: &str, keys: &str| Slot {
            label: label.to_owned(),
            action: Action::Shortcut(ShortcutAction {
                // Literal, known-good chords: covered by the default-config tests.
                keys: Shortcut::parse(keys).unwrap_or(Shortcut {
                    modifiers: Vec::new(),
                    key: super::Key::Named(super::NamedKey::Esc),
                }),
            }),
        };
        Self {
            slots: vec![
                shortcut("Copy", "Ctrl+C"),
                shortcut("Paste", "Ctrl+V"),
                shortcut("Screenshot", "Win+Shift+S"),
                Slot {
                    label: "File Explorer".to_owned(),
                    action: Action::Launch(LaunchAction {
                        path: "explorer.exe".to_owned(),
                        args: Vec::new(),
                    }),
                },
                Slot {
                    label: "Browser".to_owned(),
                    action: Action::Url(UrlAction {
                        url: "https://www.google.com".to_owned(),
                    }),
                },
                shortcut("Show desktop", "Win+D"),
            ],
        }
    }
}

impl Action {
    /// Short description shown under the label in the ring's centre.
    pub fn hint(&self) -> String {
        match self {
            Self::Shortcut(shortcut) => shortcut.keys.to_string(),
            Self::Launch(_) => "Open app".to_owned(),
            Self::Url(_) => "Open link".to_owned(),
        }
    }
}
