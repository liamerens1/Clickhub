use tauri::AppHandle;

use super::autostart::installed_exe;
use crate::platform;

/// Starts the NSIS uninstaller of this installed copy, then quits ClickHub so the uninstaller
/// can remove its files. The uninstaller's own "Delete the application data" box decides whether
/// the user's settings are kept.
///
/// # Errors
///
/// A message when this copy was not installed with the installer or Windows cannot start the
/// uninstaller. ClickHub keeps running in that case.
pub fn launch_uninstaller(app: &AppHandle) -> Result<(), String> {
    let uninstaller = installed_exe()
        .and_then(|exe| exe.parent().map(|dir| dir.join("uninstall.exe")))
        .ok_or("this copy of ClickHub was not installed with the installer, so there is nothing to uninstall")?;
    let target = uninstaller.display().to_string();
    // ShellExecute wants a COM-initialised thread; wait for it so a failure can be reported.
    let opened = std::thread::Builder::new()
        .name("clickhub-uninstall".to_owned())
        .spawn(move || {
            platform::init_shell_thread();
            platform::shell_open(&target, &[])
        })
        .map_err(|error| format!("could not start the uninstaller: {error}"))?
        .join()
        .map_err(|_| "could not start the uninstaller".to_owned())?;
    opened?;
    tracing::info!(path = %uninstaller.display(), "uninstaller started; quitting");
    app.exit(0);
    Ok(())
}
