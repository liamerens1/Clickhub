/// Input the hooks missed for longer than this means they were probably removed.
pub const MISSED_INPUT_MS: u32 = 250;
/// First wait between re-installs; doubles while re-installing does not help.
pub const BASE_BACKOFF_MS: u64 = 2_000;
pub const MAX_BACKOFF_MS: u64 = 30_000;

/// Decides when to re-install the low-level hooks.
///
/// Windows silently removes a low-level hook whose callback is too slow, without telling the
/// app. The only symptom is that input stops arriving. So the watchdog compares the newest
/// input Windows saw in the session (`GetLastInputInfo`) with the newest event the hooks saw.
/// If Windows has seen input the hooks missed, they are re-installed. No synthetic input needed.
///
/// A loss must be seen on two consecutive polls. Under heavy CPU load a callback can run late
/// and catch up a moment later; that is not a loss. A real loss persists.
///
/// Re-installing does not help when input goes to an elevated window (UIPI hides it from the
/// hooks), so repeated re-installs back off exponentially until hook events resume.
///
/// Ticks are `GetTickCount` milliseconds and may wrap; `now_ms` is any non-wrapping clock.
#[derive(Debug)]
pub struct WatchdogPolicy {
    /// Input tick at the last re-install, until hook events show up after it.
    reinstalled_at_input: Option<u32>,
    backoff_ms: u64,
    next_allowed_ms: u64,
    /// The previous poll already saw missed input.
    suspect: bool,
}

impl Default for WatchdogPolicy {
    fn default() -> Self {
        Self {
            reinstalled_at_input: None,
            backoff_ms: BASE_BACKOFF_MS,
            next_allowed_ms: 0,
            suspect: false,
        }
    }
}

impl WatchdogPolicy {
    /// Call once per poll. Returns true when the hooks should be re-installed now.
    pub fn should_reinstall(&mut self, now_ms: u64, last_input: u32, hook_last: u32) -> bool {
        let mut reference = hook_last;
        if let Some(reinstalled_at) = self.reinstalled_at_input {
            if is_newer(hook_last, reinstalled_at) {
                // Hook events resumed after the re-install: it worked.
                self.reinstalled_at_input = None;
                self.backoff_ms = BASE_BACKOFF_MS;
            } else {
                reference = reinstalled_at;
            }
        }

        let missed_ms = last_input.wrapping_sub(reference).cast_signed();
        if missed_ms <= MISSED_INPUT_MS.cast_signed() {
            self.suspect = false;
            return false;
        }
        if now_ms < self.next_allowed_ms {
            return false;
        }
        if !self.suspect {
            // First sighting: confirm on the next poll.
            self.suspect = true;
            return false;
        }

        self.suspect = false;
        self.reinstalled_at_input = Some(last_input);
        self.next_allowed_ms = now_ms.saturating_add(self.backoff_ms);
        self.backoff_ms = (self.backoff_ms * 2).min(MAX_BACKOFF_MS);
        true
    }

    /// Current wait before the next allowed re-install, for logging.
    pub fn backoff_ms(&self) -> u64 {
        self.backoff_ms
    }
}

/// Wrapping "a is later than b" for `GetTickCount` values.
fn is_newer(a: u32, b: u32) -> bool {
    a.wrapping_sub(b).cast_signed() > 0
}
