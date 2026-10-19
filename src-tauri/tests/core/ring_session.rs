use clickhub_lib::core::{HOLD_THRESHOLD_MS, IDLE_CLOSE_MS, RingEffect, RingInput, RingSession};

const DEAD: f64 = 40.0;

fn opened() -> RingSession {
    let mut session = RingSession::new(8, DEAD);
    let effects = session.handle(RingInput::TriggerDown { x: 500, y: 500 }, 0);
    assert!(matches!(effects[0], RingEffect::Open { x: 500, y: 500 }));
    session
}

fn fired(effects: &[RingEffect]) -> Option<usize> {
    effects.iter().find_map(|effect| match effect {
        RingEffect::Fire { slot } => Some(*slot),
        _ => None,
    })
}

#[test]
fn hold_aim_release_fires_the_aimed_slot() {
    let mut session = opened();
    session.handle(RingInput::Move { x: 600, y: 500 }, 50); // right = slot 2
    let effects = session.handle(RingInput::TriggerUp, HOLD_THRESHOLD_MS + 10);
    assert_eq!(fired(&effects), Some(2));
    assert_eq!(effects.last(), Some(&RingEffect::Close));
    assert!(!session.is_open());
}

#[test]
fn hold_released_in_the_centre_cancels() {
    let mut session = opened();
    let effects = session.handle(RingInput::TriggerUp, HOLD_THRESHOLD_MS + 10);
    assert_eq!(effects, vec![RingEffect::Close]);
}

#[test]
fn tap_keeps_the_ring_open_until_a_click() {
    let mut session = opened();
    assert!(session.handle(RingInput::TriggerUp, 80).is_empty());
    assert!(session.is_open());

    session.handle(RingInput::Move { x: 500, y: 650 }, 400); // down = slot 4
    let effects = session.handle(RingInput::PrimaryClick, 900);
    assert_eq!(fired(&effects), Some(4));
    assert!(!session.is_open());
}

#[test]
fn clicking_the_centre_of_a_sticky_ring_just_closes_it() {
    let mut session = opened();
    session.handle(RingInput::TriggerUp, 80);
    assert_eq!(
        session.handle(RingInput::PrimaryClick, 300),
        vec![RingEffect::Close]
    );
}

#[test]
fn dismiss_closes_without_firing() {
    let mut session = opened();
    session.handle(RingInput::TriggerUp, 80);
    session.handle(RingInput::Move { x: 600, y: 500 }, 200);
    assert_eq!(
        session.handle(RingInput::Dismiss, 300),
        vec![RingEffect::Close]
    );
}

#[test]
fn pressing_the_trigger_again_closes_a_sticky_ring() {
    let mut session = opened();
    session.handle(RingInput::TriggerUp, 80);
    assert_eq!(
        session.handle(RingInput::TriggerDown { x: 1, y: 1 }, 500),
        vec![RingEffect::Close]
    );
}

#[test]
fn aim_is_reported_only_when_it_changes() {
    let mut session = opened();
    let first = session.handle(RingInput::Move { x: 600, y: 500 }, 10);
    assert!(matches!(first.as_slice(), [RingEffect::Aim(aim)] if aim.nearest == Some(2)));
    assert!(
        session
            .handle(RingInput::Move { x: 600, y: 500 }, 20)
            .is_empty()
    );
}

#[test]
fn idle_sticky_ring_closes_itself() {
    let mut session = opened();
    session.handle(RingInput::TriggerUp, 80);
    assert!(session.handle(RingInput::Tick, IDLE_CLOSE_MS).is_empty());
    assert_eq!(
        session.handle(RingInput::Tick, IDLE_CLOSE_MS + 80),
        vec![RingEffect::Close]
    );
}

#[test]
fn inputs_while_closed_do_nothing() {
    let mut session = RingSession::new(8, DEAD);
    for input in [
        RingInput::TriggerUp,
        RingInput::Move { x: 1, y: 1 },
        RingInput::PrimaryClick,
        RingInput::Dismiss,
        RingInput::Tick,
    ] {
        assert!(session.handle(input, 0).is_empty());
    }
}

#[test]
fn reconfigure_closes_an_open_ring() {
    let mut session = opened();
    assert_eq!(session.reconfigure(6, DEAD), vec![RingEffect::Close]);
    assert!(!session.is_open());
}
