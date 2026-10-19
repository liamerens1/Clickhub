use std::sync::mpsc::{self, SyncSender};
use std::thread::{self, JoinHandle};

use windows::Win32::Foundation::{HINSTANCE, LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    GetCurrentThread, GetCurrentThreadId, SetThreadPriority, THREAD_PRIORITY_HIGHEST,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, HHOOK, MSG, PM_NOREMOVE, PeekMessageW, PostThreadMessageW,
    SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_APP, WM_QUIT,
};

use super::callbacks::{keyboard_proc, mouse_proc};
use super::shared::{SENDER, SHARED};
use super::{HookError, HookEvent};
use crate::platform::tick_count;

const WM_REINSTALL: u32 = WM_APP + 1;
const WM_SIMULATE_LOSS: u32 = WM_APP + 2;

/// Owns the hook thread. Low-level hook callbacks run on the thread that installed them, and
/// only while it pumps messages, so this thread does nothing else.
pub struct HookThread {
    remote: HookRemote,
    join: Option<JoinHandle<()>>,
}

/// A cheap, copyable handle for signalling the hook thread from any thread.
#[derive(Debug, Clone, Copy)]
pub struct HookRemote {
    thread_id: u32,
}

impl HookRemote {
    /// Asks the hook thread to remove and re-install both hooks.
    pub fn request_reinstall(self) {
        self.post(WM_REINSTALL);
    }

    /// Debug aid: removes the hooks without telling the watchdog, the way Windows does when a
    /// callback times out.
    pub fn simulate_loss(self) {
        self.post(WM_SIMULATE_LOSS);
    }

    fn post(self, message: u32) {
        // SAFETY: posting a parameterless message to a thread id we own.
        let posted = unsafe { PostThreadMessageW(self.thread_id, message, WPARAM(0), LPARAM(0)) };
        if let Err(error) = posted {
            tracing::warn!(%error, message, "could not post to the hook thread");
        }
    }
}

impl HookThread {
    /// Starts the hook thread and installs both hooks. Events go to `sender`.
    ///
    /// # Errors
    ///
    /// [`HookError::AlreadyStarted`] on a second call, [`HookError::Spawn`] if the thread cannot
    /// start, [`HookError::Install`] if Windows refuses the hooks.
    pub fn start(sender: SyncSender<HookEvent>) -> Result<Self, HookError> {
        SENDER.set(sender).map_err(|_| HookError::AlreadyStarted)?;
        let (ready_tx, ready_rx) = mpsc::channel();
        let join = thread::Builder::new()
            .name("clickhub-hook".into())
            .spawn(move || run(&ready_tx))
            .map_err(HookError::Spawn)?;
        match ready_rx.recv() {
            Ok(Ok(thread_id)) => Ok(Self {
                remote: HookRemote { thread_id },
                join: Some(join),
            }),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(HookError::ThreadDied),
        }
    }

    pub fn remote(&self) -> HookRemote {
        self.remote
    }

    /// Stops the thread and removes the hooks.
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        if let Some(join) = self.join.take() {
            self.remote.post(WM_QUIT);
            if join.join().is_err() {
                tracing::error!("hook thread panicked during shutdown");
            }
        }
    }
}

impl Drop for HookThread {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run(ready: &mpsc::Sender<Result<u32, HookError>>) {
    // SAFETY: no preconditions.
    let thread_id = unsafe { GetCurrentThreadId() };
    // The thread is idle except for callbacks, which must never run late: Windows drops hooks
    // whose callbacks exceed LowLevelHooksTimeout, and CPU starvation is a common cause.
    // SAFETY: the pseudo-handle for the current thread is always valid.
    if let Err(error) = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST) } {
        tracing::warn!(%error, "could not raise the hook thread priority");
    }
    // Make sure this thread has a message queue before anyone posts to it.
    let mut msg = MSG::default();
    // SAFETY: `msg` is a valid MSG; PM_NOREMOVE leaves any message in the queue.
    let _ = unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_NOREMOVE) };

    let mut hooks = Hooks::default();
    if let Err(error) = hooks.install() {
        let _ = ready.send(Err(error));
        return;
    }
    let _ = ready.send(Ok(thread_id));
    tracing::info!("low-level hooks installed");

    loop {
        // SAFETY: `msg` is a valid MSG. Returns 0 on WM_QUIT and -1 on error.
        let result = unsafe { GetMessageW(&raw mut msg, None, 0, 0) };
        if result.0 == 0 {
            break;
        }
        if result.0 == -1 {
            tracing::error!("GetMessageW failed on the hook thread; stopping it");
            break;
        }
        match msg.message {
            WM_REINSTALL => {
                hooks.remove();
                match hooks.install() {
                    Ok(()) => tracing::warn!("low-level hooks re-installed"),
                    Err(error) => tracing::error!(%error, "could not re-install the hooks"),
                }
            }
            WM_SIMULATE_LOSS => {
                hooks.remove();
                tracing::warn!("debug: hooks removed to simulate Windows dropping them");
            }
            _ => {
                // SAFETY: dispatching a message this thread received.
                unsafe { DispatchMessageW(&raw const msg) };
            }
        }
    }
    hooks.remove();
    tracing::info!("low-level hooks removed");
}

#[derive(Default)]
struct Hooks {
    mouse: Option<HHOOK>,
    keyboard: Option<HHOOK>,
}

impl Hooks {
    fn install(&mut self) -> Result<(), HookError> {
        // SAFETY: a null name returns the handle of this executable, which stays loaded.
        let module = unsafe { GetModuleHandleW(None) }.map_err(HookError::Install)?;
        let instance = HINSTANCE(module.0);
        // SAFETY: both procedures are `extern "system"` functions with the HOOKPROC signature
        // and live for the whole program. Thread id 0 = global low-level hook.
        let mouse = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), Some(instance), 0) }
            .map_err(HookError::Install)?;
        self.mouse = Some(mouse);
        // SAFETY: as above.
        let keyboard =
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), Some(instance), 0) }
                .map_err(HookError::Install)?;
        self.keyboard = Some(keyboard);
        // Fresh hooks count as alive now, so the watchdog measures from here.
        SHARED.mark_hooks_alive(tick_count());
        Ok(())
    }

    fn remove(&mut self) {
        for hook in [self.mouse.take(), self.keyboard.take()]
            .into_iter()
            .flatten()
        {
            // SAFETY: `hook` came from SetWindowsHookExW and is removed at most once. Failure
            // means Windows already removed it, which is fine.
            let _ = unsafe { UnhookWindowsHookEx(hook) };
        }
    }
}
