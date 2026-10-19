use crate::core::Button;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    Down,
    Up,
}

/// One physical (non-injected) mouse button event, posted by the hook callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookEvent {
    pub button: Button,
    pub action: ButtonAction,
    /// On `Down`: it matched the trigger (button + exact modifiers) while not paused.
    /// On `Up`: it is the trigger button while not paused (modifiers may already be released).
    pub matched: bool,
    /// The callback hid this event from the app underneath.
    pub swallowed: bool,
    /// Cursor position in physical screen pixels.
    pub x: i32,
    pub y: i32,
    /// QPC timestamp taken when the callback started.
    pub qpc: i64,
}
