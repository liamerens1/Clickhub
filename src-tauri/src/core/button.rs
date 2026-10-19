use crate::config::MouseButton;

/// Every physical mouse button the hook reports. Wider than [`MouseButton`] (the buttons a
/// trigger may use) because the hook also sees, and must pass through, `Left`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Button {
    Left = 1,
    Right = 2,
    Middle = 3,
    X1 = 4,
    X2 = 5,
}

impl Button {
    /// Compact code for storing in an atomic. `0` is reserved for "no button".
    pub const fn code(self) -> u8 {
        self as u8
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Left),
            2 => Some(Self::Right),
            3 => Some(Self::Middle),
            4 => Some(Self::X1),
            5 => Some(Self::X2),
            _ => None,
        }
    }
}

impl From<MouseButton> for Button {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Right => Self::Right,
            MouseButton::Middle => Self::Middle,
            MouseButton::X1 => Self::X1,
            MouseButton::X2 => Self::X2,
        }
    }
}
