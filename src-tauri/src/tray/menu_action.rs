/// Every item in the tray menu. `id` is the string Tauri hands back on click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    OpenSettings,
    Pause,
    ReloadConfig,
    SimulateHookLoss,
    StallCore,
    Quit,
}

impl MenuAction {
    /// Menu order, top to bottom. Debug items only appear when debug items are enabled; a
    /// separator sits before `Quit`.
    pub const ALL: [Self; 6] = [
        Self::OpenSettings,
        Self::Pause,
        Self::ReloadConfig,
        Self::SimulateHookLoss,
        Self::StallCore,
        Self::Quit,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::OpenSettings => "open_settings",
            Self::Pause => "pause",
            Self::ReloadConfig => "reload_config",
            Self::SimulateHookLoss => "simulate_hook_loss",
            Self::StallCore => "stall_core",
            Self::Quit => "quit",
        }
    }

    /// Label in the menu. `Pause` reads "Resume" while paused (see [`MenuAction::pause_label`]).
    pub const fn label(self) -> &'static str {
        match self {
            Self::OpenSettings => "Open settings",
            Self::Pause => Self::pause_label(false),
            Self::ReloadConfig => "Reload config",
            Self::SimulateHookLoss => "Debug: simulate hook loss",
            Self::StallCore => "Debug: stall core for 5 s",
            Self::Quit => "Quit",
        }
    }

    pub const fn pause_label(paused: bool) -> &'static str {
        if paused { "Resume" } else { "Pause" }
    }

    /// Debug items exist to verify the watchdog and fail-open behaviour by hand.
    pub const fn is_debug(self) -> bool {
        matches!(self, Self::SimulateHookLoss | Self::StallCore)
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }
}
