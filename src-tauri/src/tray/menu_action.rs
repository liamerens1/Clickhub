/// Every item in the tray menu. `id` is the string Tauri hands back on click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    OpenSettings,
    Pause,
    Quit,
}

impl MenuAction {
    /// Menu order, top to bottom. A separator sits before `Quit`.
    pub const ALL: [Self; 3] = [Self::OpenSettings, Self::Pause, Self::Quit];

    pub const fn id(self) -> &'static str {
        match self {
            Self::OpenSettings => "open_settings",
            Self::Pause => "pause",
            Self::Quit => "quit",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::OpenSettings => "Open settings",
            Self::Pause => "Pause",
            Self::Quit => "Quit",
        }
    }

    /// Pause stays disabled until there is a trigger to pause (Phase 1).
    pub const fn enabled(self) -> bool {
        !matches!(self, Self::Pause)
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }
}
