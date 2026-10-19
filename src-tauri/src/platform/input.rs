use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END,
    VK_ESCAPE, VK_F1, VK_HOME, VK_INSERT, VK_LEFT, VK_LWIN, VK_MEDIA_NEXT_TRACK,
    VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP, VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN,
    VK_RIGHT, VK_SHIFT, VK_SNAPSHOT, VK_SPACE, VK_TAB, VK_UP, VK_VOLUME_DOWN, VK_VOLUME_MUTE,
    VK_VOLUME_UP,
};

use crate::config::{Key, Modifier, NamedKey, Shortcut};

/// Sends `shortcut` to whatever app has keyboard focus: modifiers down, key down, key up,
/// modifiers up in reverse order, as one atomic `SendInput` batch.
///
/// # Errors
///
/// A message when Windows inserted fewer events than requested (for example when the
/// foreground app runs elevated and UIPI blocks injected input).
pub fn send_shortcut(shortcut: &Shortcut) -> Result<(), String> {
    let modifiers: Vec<VIRTUAL_KEY> = shortcut.modifiers.iter().map(|m| modifier_vk(*m)).collect();
    let key = key_vk(shortcut.key);

    let mut inputs = Vec::with_capacity(modifiers.len() * 2 + 2);
    inputs.extend(modifiers.iter().map(|vk| key_event(*vk, false)));
    inputs.push(key_event(key, false));
    inputs.push(key_event(key, true));
    inputs.extend(modifiers.iter().rev().map(|vk| key_event(*vk, true)));

    let size = i32::try_from(size_of::<INPUT>()).map_err(|e| e.to_string())?;
    // SAFETY: `inputs` is a valid slice of fully initialised keyboard INPUTs and `size` is the
    // size of one INPUT, as the API requires.
    let sent = unsafe { SendInput(&inputs, size) };
    if usize::try_from(sent).unwrap_or(0) == inputs.len() {
        Ok(())
    } else {
        Err(format!(
            "Windows accepted {sent} of {} key events (the app in front may be running as administrator)",
            inputs.len()
        ))
    }
}

fn key_event(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if is_extended(vk) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Keys that live on the extended part of the keyboard; without the flag some apps read them
/// as their numpad twins.
fn is_extended(vk: VIRTUAL_KEY) -> bool {
    [
        VK_INSERT,
        VK_DELETE,
        VK_HOME,
        VK_END,
        VK_PRIOR,
        VK_NEXT,
        VK_LEFT,
        VK_RIGHT,
        VK_UP,
        VK_DOWN,
        VK_LWIN,
        VK_SNAPSHOT,
    ]
    .contains(&vk)
}

fn modifier_vk(modifier: Modifier) -> VIRTUAL_KEY {
    match modifier {
        Modifier::Ctrl => VK_CONTROL,
        Modifier::Shift => VK_SHIFT,
        Modifier::Alt => VK_MENU,
        Modifier::Win => VK_LWIN,
    }
}

fn key_vk(key: Key) -> VIRTUAL_KEY {
    match key {
        // Virtual-key codes for A-Z and 0-9 are their ASCII codes.
        Key::Letter(letter) => VIRTUAL_KEY(u16::from(u8::try_from(letter).unwrap_or(b'A'))),
        Key::Digit(digit) => VIRTUAL_KEY(u16::from(b'0' + digit.min(9))),
        Key::Function(number) => VIRTUAL_KEY(VK_F1.0 + u16::from(number.clamp(1, 24) - 1)),
        Key::Named(named) => match named {
            NamedKey::Esc => VK_ESCAPE,
            NamedKey::Tab => VK_TAB,
            NamedKey::Enter => VK_RETURN,
            NamedKey::Space => VK_SPACE,
            NamedKey::Backspace => VK_BACK,
            NamedKey::Delete => VK_DELETE,
            NamedKey::Insert => VK_INSERT,
            NamedKey::Home => VK_HOME,
            NamedKey::End => VK_END,
            NamedKey::PageUp => VK_PRIOR,
            NamedKey::PageDown => VK_NEXT,
            NamedKey::Up => VK_UP,
            NamedKey::Down => VK_DOWN,
            NamedKey::Left => VK_LEFT,
            NamedKey::Right => VK_RIGHT,
            NamedKey::PrintScreen => VK_SNAPSHOT,
            NamedKey::VolumeUp => VK_VOLUME_UP,
            NamedKey::VolumeDown => VK_VOLUME_DOWN,
            NamedKey::VolumeMute => VK_VOLUME_MUTE,
            NamedKey::MediaPlayPause => VK_MEDIA_PLAY_PAUSE,
            NamedKey::MediaNext => VK_MEDIA_NEXT_TRACK,
            NamedKey::MediaPrevious => VK_MEDIA_PREV_TRACK,
            NamedKey::MediaStop => VK_MEDIA_STOP,
        },
    }
}
