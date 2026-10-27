use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, ICON_BIG, IMAGE_ICON, LR_SHARED, LoadImageW, SM_CXICON, SM_CYICON,
    SendMessageW, WM_SETICON,
};
use windows::core::PCWSTR;

/// Resource ID under which `tauri-build` embeds the app icon in the exe.
const APP_ICON_RESOURCE: u16 = 32_512;

/// Gives a window the exe's own icon as its large icon. Tauri sets only the small (title bar)
/// icon, so the taskbar falls back to the shell's icon cache for the exe path, which keeps
/// showing an old icon after an update replaced the exe. With the large icon set, the taskbar
/// uses the window's icon directly. `hwnd` is the raw handle Tauri reports.
///
/// # Errors
///
/// A message when the icon cannot be loaded from the exe.
pub fn set_large_window_icon(hwnd: *mut core::ffi::c_void) -> Result<(), String> {
    // SAFETY: a null name returns this process's own module handle.
    let module = unsafe { GetModuleHandleW(PCWSTR::null()) }.map_err(|e| e.to_string())?;
    // SAFETY: plain metric queries.
    let (width, height) = unsafe { (GetSystemMetrics(SM_CXICON), GetSystemMetrics(SM_CYICON)) };
    // SAFETY: the resource ID is passed as an integer resource name (MAKEINTRESOURCE); a shared
    // icon is owned by the system and must not be destroyed, so nothing is leaked or freed.
    let icon = unsafe {
        LoadImageW(
            Some(module.into()),
            PCWSTR(usize::from(APP_ICON_RESOURCE) as *const u16),
            IMAGE_ICON,
            width,
            height,
            LR_SHARED,
        )
    }
    .map_err(|e| format!("could not load the app icon: {e}"))?;
    // SAFETY: `hwnd` is a live top-level window owned by this process; WM_SETICON only stores
    // the icon handle.
    unsafe {
        SendMessageW(
            HWND(hwnd),
            WM_SETICON,
            Some(WPARAM(ICON_BIG as usize)),
            Some(LPARAM(icon.0 as isize)),
        );
    }
    Ok(())
}
