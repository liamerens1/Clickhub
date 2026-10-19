use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
};
use windows::Win32::UI::Shell::{
    QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetShellWindow, GetWindowRect,
};

/// True when the foreground app is full-screen: an exclusive Direct3D app, presentation mode,
/// or a window covering its whole monitor (borderless full-screen games and videos). The
/// desktop and the shell never count.
pub fn foreground_is_fullscreen() -> bool {
    // SAFETY: no arguments; an error just means "unknown", treated as not full-screen.
    if let Ok(state) = unsafe { SHQueryUserNotificationState() }
        && (state == QUNS_RUNNING_D3D_FULL_SCREEN || state == QUNS_PRESENTATION_MODE)
    {
        return true;
    }

    // SAFETY: no arguments.
    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() || is_shell_window(window) {
        return false;
    }
    let mut rect = RECT::default();
    // SAFETY: `window` is a window handle and `rect` a valid out pointer.
    if unsafe { GetWindowRect(window, &raw mut rect) }.is_err() {
        return false;
    }
    // SAFETY: MONITOR_DEFAULTTONEAREST always returns a monitor.
    let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: u32::try_from(size_of::<MONITORINFO>()).unwrap_or(0),
        ..MONITORINFO::default()
    };
    // SAFETY: `info` has cbSize set, as the API requires.
    if !unsafe { GetMonitorInfoW(monitor, &raw mut info) }.as_bool() {
        return false;
    }
    let screen = info.rcMonitor;
    rect.left <= screen.left
        && rect.top <= screen.top
        && rect.right >= screen.right
        && rect.bottom >= screen.bottom
}

fn is_shell_window(window: HWND) -> bool {
    // SAFETY: no arguments.
    if window == unsafe { GetShellWindow() } {
        return true;
    }
    let mut buffer = [0_u16; 64];
    // SAFETY: `buffer` is writable; the API writes at most its length.
    let length = unsafe { GetClassNameW(window, &mut buffer) };
    let class = String::from_utf16_lossy(&buffer[..usize::try_from(length).unwrap_or(0)]);
    // The desktop (Progman/WorkerW) covers the screen but is not a full-screen app.
    matches!(class.as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd")
}
