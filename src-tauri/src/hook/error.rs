use thiserror::Error;

#[derive(Debug, Error)]
pub enum HookError {
    #[error("the hook thread was already started")]
    AlreadyStarted,
    #[error("could not start the hook thread: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("Windows refused to install the mouse/keyboard hook: {0}")]
    Install(#[source] windows::core::Error),
    #[error("the hook thread stopped before reporting whether the hooks were installed")]
    ThreadDied,
}
