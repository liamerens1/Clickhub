/// Presses shorter than this are taps; longer ones are holds.
pub const HOLD_THRESHOLD_MS: u64 = 250;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressKind {
    Tap,
    Hold,
}

/// A completed trigger press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Press {
    pub kind: PressKind,
    pub held_ms: u64,
}

/// Turns trigger down/up events into taps and holds. Times are in milliseconds from any
/// monotonic clock.
#[derive(Debug, Default)]
pub struct PressTracker {
    down_at_ms: Option<u64>,
}

impl PressTracker {
    pub fn on_down(&mut self, now_ms: u64) {
        // A repeated down without an up (lost event) restarts the press.
        self.down_at_ms = Some(now_ms);
    }

    /// Completes the press. Returns `None` for an up without a matching down.
    pub fn on_up(&mut self, now_ms: u64) -> Option<Press> {
        let down_at_ms = self.down_at_ms.take()?;
        let held_ms = now_ms.saturating_sub(down_at_ms);
        let kind = if held_ms < HOLD_THRESHOLD_MS {
            PressKind::Tap
        } else {
            PressKind::Hold
        };
        Some(Press { kind, held_ms })
    }

    pub fn is_pressed(&self) -> bool {
        self.down_at_ms.is_some()
    }

    /// Forgets an in-progress press (e.g. when pausing).
    pub fn reset(&mut self) {
        self.down_at_ms = None;
    }
}
