use super::{Button, ModifierSet};
use crate::config::Trigger;

/// The mouse trigger in a form the hook callback can test in a few instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TriggerFilter {
    /// `None` when the trigger is not a mouse button (e.g. a keyboard chord).
    pub button: Option<Button>,
    pub modifiers: ModifierSet,
    pub swallow: bool,
}

impl TriggerFilter {
    /// A filter that never matches.
    pub const DISABLED: Self = Self {
        button: None,
        modifiers: ModifierSet::EMPTY,
        swallow: false,
    };

    pub fn from_trigger(trigger: &Trigger) -> Self {
        match trigger {
            Trigger::Mouse(mouse) => Self {
                button: Some(mouse.button.into()),
                modifiers: mouse.modifiers.iter().copied().collect(),
                swallow: mouse.swallow,
            },
            Trigger::Keyboard(_) => Self::DISABLED,
        }
    }

    /// True when `button` is the trigger button and exactly the configured modifiers are held.
    /// Exact matching keeps `X2` and `Ctrl+X2` distinct.
    pub fn matches(self, button: Button, held: ModifierSet) -> bool {
        self.button == Some(button) && self.modifiers == held
    }
}
