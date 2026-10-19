use serde::{Deserialize, Serialize};

/// What opens the ring. Mirrors `triggerSchema` in `src/shared/schema/trigger.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Trigger {
    Mouse(MouseTrigger),
    Keyboard(KeyboardTrigger),
}

impl Default for Trigger {
    fn default() -> Self {
        Self::Mouse(MouseTrigger::default())
    }
}

/// A mouse button trigger, read by the low-level hook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MouseTrigger {
    pub button: MouseButton,
    /// Exact set of modifiers that must be held. Empty means "no modifiers held".
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
    /// Hide the trigger click from the app underneath. On by default since Phase 2, so opening
    /// the ring does not also fire the button's normal action.
    #[serde(default = "swallow_by_default")]
    pub swallow: bool,
}

const fn swallow_by_default() -> bool {
    true
}

impl Default for MouseTrigger {
    fn default() -> Self {
        Self {
            button: MouseButton::X2,
            modifiers: Vec::new(),
            swallow: swallow_by_default(),
        }
    }
}

/// Buttons that may trigger the ring. `left` is deliberately absent: a bad config must never be
/// able to take over the primary button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MouseButton {
    Right,
    Middle,
    X1,
    X2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Win,
}

/// A keyboard chord, registered through `tauri-plugin-global-shortcut`
/// (format: `"Ctrl+Alt+Space"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyboardTrigger {
    pub shortcut: String,
}
