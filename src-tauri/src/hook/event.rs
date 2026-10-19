use crate::core::Button;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEventKind {
    /// A physical (non-injected) mouse button event.
    Button {
        button: Button,
        action: ButtonAction,
        /// On `Down`: it matched the trigger (button + exact modifiers) while not paused or
        /// suppressed. On `Up`: it is the trigger button while not paused or suppressed.
        matched: bool,
        /// The callback hid this event from the app underneath.
        swallowed: bool,
    },
    /// The cursor moved. Only posted while a ring is open.
    Move,
    /// Esc was pressed. Only posted (and swallowed) while a ring is open.
    Escape,
}

/// One event posted by a hook callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookEvent {
    pub kind: HookEventKind,
    /// Cursor position in physical screen pixels (for `Escape`: the last known position).
    pub x: i32,
    pub y: i32,
    /// QPC timestamp taken when the callback started.
    pub qpc: i64,
}
