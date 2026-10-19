use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

use crate::config::Action;
use crate::platform::{init_shell_thread, send_shortcut, shell_open};

/// Owns the action thread. Dropping it lets the thread finish queued actions and exit.
pub struct ActionRunner {
    sender: Option<Sender<Action>>,
    join: Option<JoinHandle<()>>,
}

impl ActionRunner {
    /// # Errors
    ///
    /// Fails only if the OS cannot start a thread.
    pub fn spawn() -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel::<Action>();
        let join = thread::Builder::new()
            .name("clickhub-actions".into())
            .spawn(move || {
                init_shell_thread();
                for action in receiver {
                    run(&action);
                }
            })?;
        Ok(Self {
            sender: Some(sender),
            join: Some(join),
        })
    }

    /// Queues `action`. Never blocks.
    pub fn run(&self, action: Action) {
        let sent = self
            .sender
            .as_ref()
            .is_some_and(|sender| sender.send(action).is_ok());
        if !sent {
            tracing::error!("action thread is gone; action dropped");
        }
    }
}

impl Drop for ActionRunner {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(join) = self.join.take()
            && join.join().is_err()
        {
            tracing::error!("action thread panicked");
        }
    }
}

fn run(action: &Action) {
    let result = match action {
        Action::Shortcut(shortcut) => send_shortcut(&shortcut.keys),
        Action::Launch(launch) => shell_open(launch.path.trim(), &launch.args),
        Action::Url(url) => shell_open(&url.url, &[]),
    };
    match result {
        Ok(()) => tracing::info!(action = %action.hint(), "action ran"),
        Err(error) => tracing::warn!(action = %action.hint(), %error, "action failed"),
    }
}
