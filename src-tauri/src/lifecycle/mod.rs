//! Installed-app concerns: starting with Windows, update checks and installs, and launching the
//! uninstaller. Never on the hot path; Win32 only through `platform/`.

mod autostart;
mod uninstall;
mod updates;

pub use autostart::{
    AutostartState, apply_default as apply_autostart_default, installed_exe,
    set_enabled as set_autostart, state as autostart_state,
};
pub use uninstall::launch_uninstaller;
pub use updates::{UpdateState, Updates};
