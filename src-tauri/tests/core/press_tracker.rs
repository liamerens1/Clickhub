use clickhub_lib::core::{HOLD_THRESHOLD_MS, Press, PressKind, PressTracker};

#[test]
fn short_press_is_a_tap() {
    let mut tracker = PressTracker::default();
    tracker.on_down(1_000);
    assert!(tracker.is_pressed());
    assert_eq!(
        tracker.on_up(1_000 + HOLD_THRESHOLD_MS - 1),
        Some(Press {
            kind: PressKind::Tap,
            held_ms: HOLD_THRESHOLD_MS - 1
        })
    );
    assert!(!tracker.is_pressed());
}

#[test]
fn press_at_or_beyond_threshold_is_a_hold() {
    let mut tracker = PressTracker::default();
    tracker.on_down(0);
    assert_eq!(
        tracker.on_up(HOLD_THRESHOLD_MS),
        Some(Press {
            kind: PressKind::Hold,
            held_ms: HOLD_THRESHOLD_MS
        })
    );
}

#[test]
fn up_without_down_is_ignored() {
    let mut tracker = PressTracker::default();
    assert_eq!(tracker.on_up(500), None);
}

#[test]
fn reset_forgets_the_press() {
    let mut tracker = PressTracker::default();
    tracker.on_down(0);
    tracker.reset();
    assert_eq!(tracker.on_up(10), None);
}

#[test]
fn clock_going_backwards_does_not_underflow() {
    let mut tracker = PressTracker::default();
    tracker.on_down(1_000);
    let press = tracker.on_up(900).expect("press completes");
    assert_eq!(press.held_ms, 0);
}
