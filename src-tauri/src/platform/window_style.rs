use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW, HWND_TOPMOST, IsIconic, SW_SHOWNOACTIVATE,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
};

/// Prepares an overlay window: never takes focus, never in the taskbar or Alt-Tab
/// (`WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`), and cannot be minimized or maximized, so
/// "show desktop" (Win+D) has nothing to minimize. `hwnd` is the raw handle Tauri reports.
///
/// # Errors
///
/// A message when the styles could not be read back as set.
pub fn make_overlay_window(hwnd: *mut core::ffi::c_void) -> Result<(), String> {
    let window = HWND(hwnd);
    let ex_wanted =
        isize::try_from(WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0).map_err(|e| e.to_string())?;
    let unwanted =
        isize::try_from(WS_MINIMIZEBOX.0 | WS_MAXIMIZEBOX.0).map_err(|e| e.to_string())?;
    // SAFETY: `window` is a live top-level window owned by this process; only style bits change.
    unsafe {
        let ex = GetWindowLongPtrW(window, GWL_EXSTYLE);
        SetWindowLongPtrW(window, GWL_EXSTYLE, ex | ex_wanted);
        let style = GetWindowLongPtrW(window, GWL_STYLE);
        SetWindowLongPtrW(window, GWL_STYLE, style & !unwanted);
    }
    // SAFETY: as above.
    let (ex, style) = unsafe {
        (
            GetWindowLongPtrW(window, GWL_EXSTYLE),
            GetWindowLongPtrW(window, GWL_STYLE),
        )
    };
    if ex & ex_wanted == ex_wanted && style & unwanted == 0 {
        Ok(())
    } else {
        Err(format!("styles are {style:#x} / {ex:#x}"))
    }
}

/// Moves an overlay to physical screen position `(x, y)` and puts it back on top of every
/// other window, without activating it. Re-asserting topmost on every open matters: "show
/// desktop" and some full-screen apps leave topmost tool windows underneath when they restore.
///
/// # Errors
///
/// A message with Windows' error when the window could not be placed.
pub fn place_overlay(hwnd: *mut core::ffi::c_void, x: i32, y: i32) -> Result<(), String> {
    let window = HWND(hwnd);
    // SAFETY: `window` is a live top-level window owned by this process.
    if unsafe { IsIconic(window) }.as_bool() {
        // SAFETY: as above; SW_SHOWNOACTIVATE restores without taking focus.
        let _ = unsafe { ShowWindow(window, SW_SHOWNOACTIVATE) };
    }
    // SAFETY: as above; SWP_NOSIZE keeps the size (overlays are never resized).
    unsafe {
        SetWindowPos(
            window,
            Some(HWND_TOPMOST),
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )
    }
    .map_err(|error| error.to_string())
}

/// Puts an overlay back on top of every other window without moving or activating it. Called
/// periodically while a ring is open, so switching apps (Alt+Tab) cannot bury it.
///
/// # Errors
///
/// A message with Windows' error when the call fails.
pub fn keep_on_top(hwnd: *mut core::ffi::c_void) -> Result<(), String> {
    // SAFETY: `hwnd` is a live top-level window owned by this process; position and size kept.
    unsafe {
        SetWindowPos(
            HWND(hwnd),
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )
    }
    .map_err(|error| error.to_string())
}
