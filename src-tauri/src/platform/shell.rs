use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, w};

/// Prepares the calling thread for [`shell_open`]. Call once per thread before using it.
pub fn init_shell_thread() {
    // SAFETY: initialises COM for this thread only; a repeated call just returns S_FALSE.
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
}

/// Opens `target` (an executable, file, folder or URL) the way Explorer would, with `args`.
///
/// # Errors
///
/// A message with Windows' error code when the target cannot be opened.
pub fn shell_open(target: &str, args: &[String]) -> Result<(), String> {
    let file = HSTRING::from(target);
    let parameters = HSTRING::from(join_args(args));
    // SAFETY: all strings are valid, null-terminated wide strings that outlive the call.
    let result =
        unsafe { ShellExecuteW(None, w!("open"), &file, &parameters, None, SW_SHOWNORMAL) };
    // ShellExecuteW returns a value greater than 32 on success, an error code otherwise.
    let code = result.0 as usize;
    if code > 32 {
        Ok(())
    } else {
        Err(format!("Windows could not open `{target}` (error {code})"))
    }
}

/// Joins arguments into one command line, quoting any that contain spaces or quotes.
fn join_args(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.is_empty() || arg.contains([' ', '\t', '"']) {
                format!("\"{}\"", arg.replace('"', "\\\""))
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
