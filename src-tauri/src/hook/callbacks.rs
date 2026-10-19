//! The two low-level hook procedures. Hard rule 5: only match and post a message.
//! No allocation, no I/O, no logging, no locks. Everything here reads and writes atomics.

use std::sync::atomic::Ordering;

use windows::Win32::Foundation::{LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLMHF_INJECTED, MSLLHOOKSTRUCT,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
    XBUTTON1, XBUTTON2,
};

use super::shared::{SENDER, SHARED};
use super::{ButtonAction, HookEvent, HookEventKind};
use crate::core::Button;
use crate::platform::{held_modifiers, qpc_now, tick_count};

/// Returning a non-zero value from a low-level hook hides the event from every other app.
const SWALLOW: LRESULT = LRESULT(1);

/// `WH_MOUSE_LL` procedure.
pub(super) unsafe extern "system" fn mouse_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let started = qpc_now();
    let mut swallow = false;

    if code == HC_ACTION.cast_signed() {
        SHARED
            .last_mouse_tick
            .store(tick_count(), Ordering::Relaxed);
        // SAFETY: for HC_ACTION, Windows passes a pointer to an MSLLHOOKSTRUCT that is valid
        // for the duration of this call. We only read it.
        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        let injected = info.flags & LLMHF_INJECTED != 0;
        if !injected {
            SHARED.store_point(info.pt.x, info.pt.y);
            if u32::try_from(wparam.0).is_ok_and(|message| message == WM_MOUSEMOVE) {
                if SHARED.ring_capturing() {
                    post(HookEventKind::Move, info.pt, started);
                }
            } else if let Some((button, action)) = classify(wparam, info.mouseData) {
                swallow = on_button(button, action, info.pt, started);
            }
        }
    }

    SHARED.record_duration(qpc_now() - started);
    if swallow {
        SWALLOW
    } else {
        // SAFETY: forwarding the unchanged arguments to the next hook, as required.
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }
}

/// `WH_KEYBOARD_LL` procedure: a liveness signal for the watchdog, and Esc for an open ring.
pub(super) unsafe extern "system" fn keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let mut swallow = false;
    if code == HC_ACTION.cast_signed() {
        let started = qpc_now();
        SHARED
            .last_keyboard_tick
            .store(tick_count(), Ordering::Relaxed);
        // SAFETY: for HC_ACTION, lparam points to a KBDLLHOOKSTRUCT valid for this call.
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let injected = info.flags.0 & LLKHF_INJECTED.0 != 0;
        if !injected && info.vkCode == u32::from(VK_ESCAPE.0) {
            let message = u32::try_from(wparam.0).unwrap_or(0);
            if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                if SHARED.ring_capturing() {
                    SHARED.escape_swallowed.store(true, Ordering::Relaxed);
                    let (x, y) = SHARED.point();
                    post(HookEventKind::Escape, POINT { x, y }, started);
                    swallow = true;
                }
            } else if message == WM_KEYUP || message == WM_SYSKEYUP {
                swallow = SHARED.escape_swallowed.swap(false, Ordering::Relaxed);
            }
        }
    }
    if swallow {
        SWALLOW
    } else {
        // SAFETY: forwarding the unchanged arguments to the next hook, as required.
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }
}

/// Decides whether to swallow, then posts the event. Returns true to swallow.
fn on_button(button: Button, action: ButtonAction, pt: POINT, qpc: i64) -> bool {
    let filter = SHARED.filter();
    let blocked = SHARED.is_paused() || SHARED.is_suppressed();
    let is_trigger_button = !blocked && filter.button == Some(button);
    let bit = 1_u8 << button.code();

    let (matched, swallowed) = match action {
        ButtonAction::Down => {
            // Only query modifier state for the trigger button: ordinary clicks stay cheap.
            let matched = is_trigger_button && filter.matches(button, held_modifiers());
            let swallowed = if matched {
                filter.swallow && SHARED.core_alive()
            } else {
                // Any other button while a ring is open belongs to the ring.
                !blocked && SHARED.ring_capturing()
            };
            if swallowed {
                SHARED.swallowed_buttons.fetch_or(bit, Ordering::Relaxed);
            }
            (matched, swallowed)
        }
        // Swallow an Up exactly when its Down was swallowed, even if the state changed in
        // between: an app that saw no Down must not see a lone Up, and vice versa.
        ButtonAction::Up => {
            let previous = SHARED.swallowed_buttons.fetch_and(!bit, Ordering::Relaxed);
            (is_trigger_button, previous & bit != 0)
        }
    };

    post(
        HookEventKind::Button {
            button,
            action,
            matched,
            swallowed,
        },
        pt,
        qpc,
    );
    swallowed
}

fn post(kind: HookEventKind, pt: POINT, qpc: i64) {
    let event = HookEvent {
        kind,
        x: pt.x,
        y: pt.y,
        qpc,
    };
    let delivered = SENDER
        .get()
        .is_some_and(|sender| sender.try_send(event).is_ok());
    if !delivered {
        SHARED.dropped_events.fetch_add(1, Ordering::Relaxed);
    }
}

fn classify(wparam: WPARAM, mouse_data: u32) -> Option<(Button, ButtonAction)> {
    let message = u32::try_from(wparam.0).ok()?;
    let x_button = || match (mouse_data >> 16) as u16 {
        XBUTTON1 => Some(Button::X1),
        XBUTTON2 => Some(Button::X2),
        _ => None,
    };
    match message {
        WM_LBUTTONDOWN => Some((Button::Left, ButtonAction::Down)),
        WM_LBUTTONUP => Some((Button::Left, ButtonAction::Up)),
        WM_RBUTTONDOWN => Some((Button::Right, ButtonAction::Down)),
        WM_RBUTTONUP => Some((Button::Right, ButtonAction::Up)),
        WM_MBUTTONDOWN => Some((Button::Middle, ButtonAction::Down)),
        WM_MBUTTONUP => Some((Button::Middle, ButtonAction::Up)),
        WM_XBUTTONDOWN => x_button().map(|button| (button, ButtonAction::Down)),
        WM_XBUTTONUP => x_button().map(|button| (button, ButtonAction::Up)),
        _ => None,
    }
}
