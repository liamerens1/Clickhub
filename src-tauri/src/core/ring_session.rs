use super::{Aim, HOLD_THRESHOLD_MS, aim, full_ring};

/// A ring left open this long without being used closes by itself, so a forgotten ring never
/// keeps capturing clicks.
pub const IDLE_CLOSE_MS: u64 = 30_000;

/// Something the outside world did while a ring may be open.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RingInput {
    /// The trigger went down at screen position `(x, y)`.
    TriggerDown {
        x: i32,
        y: i32,
    },
    TriggerUp,
    /// The cursor moved to screen position `(x, y)`.
    Move {
        x: i32,
        y: i32,
    },
    /// The primary (left) button was clicked while the ring was open.
    PrimaryClick,
    /// Right/middle/other button, or Esc: close without firing.
    Dismiss,
    /// Periodic tick, so idle rings can close.
    Tick,
}

/// What the caller must do in response.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RingEffect {
    /// Show the ring centred on `(x, y)`.
    Open { x: i32, y: i32 },
    /// The aim changed.
    Aim(Aim),
    /// Run the action of `slot`, then the ring closes (a `Close` follows).
    Fire { slot: usize },
    /// Hide the ring.
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Trigger still down; not yet known whether this is a tap or a hold.
    Pressed,
    /// Tapped: stays open until a click, Esc or timeout.
    Sticky,
}

#[derive(Debug, Clone, Copy)]
struct Open {
    mode: Mode,
    centre: (i32, i32),
    opened_at_ms: u64,
    last_activity_ms: u64,
    aim: Aim,
}

/// The tap/hold state machine for one ring. Pure: time and geometry are passed in.
///
/// - Trigger down opens the ring at the cursor.
/// - Released within [`HOLD_THRESHOLD_MS`]: a tap, the ring stays open (sticky). A primary click
///   then fires the aimed slot, or closes it when aiming at the centre. Dismiss closes it.
/// - Released later: a hold, which fires the aimed slot on release, or cancels in the centre.
#[derive(Debug)]
pub struct RingSession {
    slot_count: usize,
    dead_zone: f64,
    /// Slot angles for the current open (radians clockwise from top); see `set_layout`.
    angles: Vec<f64>,
    open: Option<Open>,
}

impl RingSession {
    pub fn new(slot_count: usize, dead_zone: f64) -> Self {
        Self {
            slot_count,
            dead_zone,
            angles: full_ring(slot_count).angles,
            open: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Sets the dead zone without closing (e.g. once the monitor's DPI is known after opening).
    pub fn set_dead_zone(&mut self, dead_zone: f64) {
        self.dead_zone = dead_zone;
    }

    /// Sets where the slots sit for the current open (edge-aware layout). Ignored if the count
    /// does not match.
    pub fn set_layout(&mut self, angles: Vec<f64>) {
        if angles.len() == self.slot_count {
            self.angles = angles;
        }
    }

    /// Changes the slot count (config reload). Closes an open ring first.
    pub fn reconfigure(&mut self, slot_count: usize, dead_zone: f64) -> Vec<RingEffect> {
        self.slot_count = slot_count;
        self.dead_zone = dead_zone;
        self.angles = full_ring(slot_count).angles;
        self.close()
    }

    /// Feeds one input at time `now_ms` and returns the effects, in order.
    pub fn handle(&mut self, input: RingInput, now_ms: u64) -> Vec<RingEffect> {
        match (input, self.open) {
            (RingInput::TriggerDown { x, y }, None) => {
                self.angles = full_ring(self.slot_count).angles;
                let centre_aim = aim(0.0, 0.0, &self.angles, self.dead_zone);
                self.open = Some(Open {
                    mode: Mode::Pressed,
                    centre: (x, y),
                    opened_at_ms: now_ms,
                    last_activity_ms: now_ms,
                    aim: centre_aim,
                });
                vec![RingEffect::Open { x, y }, RingEffect::Aim(centre_aim)]
            }
            // Trigger again while sticky: treat as a dismiss so it toggles closed.
            (RingInput::TriggerDown { .. }, Some(open)) if open.mode == Mode::Sticky => {
                self.close()
            }
            (RingInput::TriggerUp, Some(open)) if open.mode == Mode::Pressed => {
                if now_ms.saturating_sub(open.opened_at_ms) < HOLD_THRESHOLD_MS {
                    self.open = Some(Open {
                        mode: Mode::Sticky,
                        last_activity_ms: now_ms,
                        ..open
                    });
                    Vec::new()
                } else {
                    self.fire_or_close(open.aim)
                }
            }
            (RingInput::Move { x, y }, Some(open)) => {
                let new_aim = aim(
                    f64::from(x - open.centre.0),
                    f64::from(y - open.centre.1),
                    &self.angles,
                    self.dead_zone,
                );
                self.open = Some(Open {
                    aim: new_aim,
                    last_activity_ms: now_ms,
                    ..open
                });
                if new_aim == open.aim {
                    Vec::new()
                } else {
                    vec![RingEffect::Aim(new_aim)]
                }
            }
            (RingInput::PrimaryClick, Some(open)) => self.fire_or_close(open.aim),
            (RingInput::Dismiss, Some(_)) => self.close(),
            (RingInput::Tick, Some(open))
                if now_ms.saturating_sub(open.last_activity_ms) >= IDLE_CLOSE_MS =>
            {
                self.close()
            }
            _ => Vec::new(),
        }
    }

    fn fire_or_close(&mut self, current: Aim) -> Vec<RingEffect> {
        let mut effects = match current.nearest {
            Some(slot) => vec![RingEffect::Fire { slot }],
            None => Vec::new(),
        };
        effects.extend(self.close());
        effects
    }

    fn close(&mut self) -> Vec<RingEffect> {
        if self.open.take().is_some() {
            vec![RingEffect::Close]
        } else {
            Vec::new()
        }
    }
}
