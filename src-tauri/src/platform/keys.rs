use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};

use crate::config::Modifier;
use crate::core::ModifierSet;

/// Modifier keys held right now. Allocation-free: safe in the hook callback.
pub fn held_modifiers() -> ModifierSet {
    let mut held = ModifierSet::EMPTY;
    if is_down(VK_CONTROL) {
        held = held.with(Modifier::Ctrl);
    }
    if is_down(VK_SHIFT) {
        held = held.with(Modifier::Shift);
    }
    if is_down(VK_MENU) {
        held = held.with(Modifier::Alt);
    }
    if is_down(VK_LWIN) || is_down(VK_RWIN) {
        held = held.with(Modifier::Win);
    }
    held
}

fn is_down(key: VIRTUAL_KEY) -> bool {
    // SAFETY: GetAsyncKeyState takes any virtual-key code and has no failure mode beyond
    // returning 0. The high bit set (negative i16) means "currently down".
    let state = unsafe { GetAsyncKeyState(i32::from(key.0)) };
    state < 0
}
