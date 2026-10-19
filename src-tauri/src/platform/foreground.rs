use std::ffi::c_void;
use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows::core::PWSTR;

/// The app that owns the foreground window. Used for diagnostics now and for full-screen
/// detection and per-app rings later. Not for the hook callback: it opens handles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundApp {
    pub pid: u32,
    /// Executable file name (e.g. `notepad.exe`), if Windows lets us read it.
    pub exe: Option<String>,
    /// Whether the process runs elevated. `None` when its token cannot be read, which for a
    /// non-elevated caller usually means the target *is* elevated.
    pub elevated: Option<bool>,
}

/// Looks up the foreground window's process. `None` when there is no foreground window.
pub fn foreground_app() -> Option<ForegroundApp> {
    // SAFETY: no arguments; returns a null handle when nothing is in the foreground.
    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() {
        return None;
    }
    let mut pid = 0_u32;
    // SAFETY: `window` came from GetForegroundWindow and `pid` is a valid out pointer.
    unsafe { GetWindowThreadProcessId(window, Some(&raw mut pid)) };
    if pid == 0 {
        return None;
    }

    // SAFETY: PROCESS_QUERY_LIMITED_INFORMATION may be requested for any process, including
    // elevated ones; failure (e.g. protected processes) is handled below.
    let Ok(process) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) })
    else {
        return Some(ForegroundApp {
            pid,
            exe: None,
            elevated: None,
        });
    };
    let app = ForegroundApp {
        pid,
        exe: image_name(process),
        elevated: is_elevated(process),
    };
    close(process);
    Some(app)
}

fn image_name(process: HANDLE) -> Option<String> {
    let mut buffer = [0_u16; 1024];
    let mut length = u32::try_from(buffer.len()).ok()?;
    // SAFETY: `buffer` holds `length` UTF-16 units and `length` is updated to the chars written.
    unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &raw mut length,
        )
    }
    .ok()?;
    let path = String::from_utf16_lossy(buffer.get(..usize::try_from(length).ok()?)?);
    Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

fn is_elevated(process: HANDLE) -> Option<bool> {
    let mut token = HANDLE::default();
    // SAFETY: `process` is a valid process handle and `token` a valid out pointer.
    unsafe { OpenProcessToken(process, TOKEN_QUERY, &raw mut token) }.ok()?;
    let mut elevation = TOKEN_ELEVATION::default();
    let mut returned = 0_u32;
    let size = u32::try_from(size_of::<TOKEN_ELEVATION>()).ok()?;
    // SAFETY: `elevation` is a TOKEN_ELEVATION of `size` bytes, as TokenElevation requires.
    let result = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some((&raw mut elevation).cast::<c_void>()),
            size,
            &raw mut returned,
        )
    };
    close(token);
    result.ok()?;
    Some(elevation.TokenIsElevated != 0)
}

fn close(handle: HANDLE) {
    // SAFETY: `handle` was opened by this module and is closed exactly once.
    let _ = unsafe { CloseHandle(handle) };
}
