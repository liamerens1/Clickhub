use clickhub_lib::core::{BASE_BACKOFF_MS, MAX_BACKOFF_MS, MISSED_INPUT_MS, WatchdogPolicy};

/// Polls are 100 ms apart in the real watchdog.
const POLL_MS: u64 = 100;

/// Two consecutive polls with the same view of input, as a persistent loss looks.
fn poll_twice(policy: &mut WatchdogPolicy, now_ms: u64, last_input: u32, hook_last: u32) -> bool {
    let first = policy.should_reinstall(now_ms, last_input, hook_last);
    let second = policy.should_reinstall(now_ms + POLL_MS, last_input, hook_last);
    first || second
}

#[test]
fn healthy_hooks_are_left_alone() {
    let mut policy = WatchdogPolicy::default();
    // Hooks saw the latest input, or input only slightly newer than their last event.
    assert!(!poll_twice(&mut policy, 0, 1_000, 1_000));
    assert!(!poll_twice(
        &mut policy,
        1_000,
        1_000 + MISSED_INPUT_MS,
        1_000
    ));
}

#[test]
fn persistent_missed_input_triggers_a_reinstall_on_the_second_poll() {
    let mut policy = WatchdogPolicy::default();
    let last_input = 1_000 + MISSED_INPUT_MS + 1;
    assert!(
        !policy.should_reinstall(0, last_input, 1_000),
        "first sighting only arms"
    );
    assert!(policy.should_reinstall(POLL_MS, last_input, 1_000));
}

#[test]
fn a_late_callback_that_catches_up_is_not_a_loss() {
    let mut policy = WatchdogPolicy::default();
    // CPU starvation: the callback is 328 ms behind on one poll...
    assert!(!policy.should_reinstall(0, 1_328, 1_000));
    // ...and has caught up by the next one.
    assert!(!policy.should_reinstall(POLL_MS, 1_400, 1_400));
    // A later, separate hiccup must again be confirmed from scratch.
    assert!(!policy.should_reinstall(2 * POLL_MS, 1_800, 1_400));
}

#[test]
fn does_not_reinstall_again_for_the_same_missed_input() {
    let mut policy = WatchdogPolicy::default();
    assert!(poll_twice(&mut policy, 0, 5_000, 1_000));
    // Same last input, well after the backoff: nothing new was missed.
    assert!(!poll_twice(&mut policy, 60_000, 5_000, 1_000));
}

#[test]
fn backs_off_while_reinstalling_does_not_help() {
    let mut policy = WatchdogPolicy::default();
    assert!(poll_twice(&mut policy, 0, 5_000, 1_000));
    // The re-install happened on the second poll, at POLL_MS.
    let reinstalled_at = POLL_MS;

    // More input missed, but still inside the first backoff window.
    assert!(!poll_twice(&mut policy, reinstalled_at + 10, 9_000, 1_000));
    // After it, re-install again and double the wait.
    assert!(poll_twice(
        &mut policy,
        reinstalled_at + BASE_BACKOFF_MS,
        9_000,
        1_000
    ));
    assert_eq!(policy.backoff_ms(), BASE_BACKOFF_MS * 4);
}

#[test]
fn backoff_is_capped() {
    let mut policy = WatchdogPolicy::default();
    let mut now = 0_u64;
    let mut input = 1_000_u32;
    for _ in 0..20 {
        input += 10_000;
        now += MAX_BACKOFF_MS + POLL_MS;
        poll_twice(&mut policy, now, input, 0);
    }
    assert_eq!(policy.backoff_ms(), MAX_BACKOFF_MS);
}

#[test]
fn backoff_resets_once_hook_events_resume() {
    let mut policy = WatchdogPolicy::default();
    assert!(poll_twice(&mut policy, 0, 5_000, 1_000));
    assert!(poll_twice(
        &mut policy,
        BASE_BACKOFF_MS + POLL_MS,
        9_000,
        1_000
    ));

    // The hooks saw an event after the last re-install: back to the base backoff.
    assert!(!policy.should_reinstall(BASE_BACKOFF_MS * 3, 9_100, 9_100));
    assert_eq!(policy.backoff_ms(), BASE_BACKOFF_MS);
}

#[test]
fn handles_tick_count_wraparound() {
    let mut policy = WatchdogPolicy::default();
    let hook_last = u32::MAX - 100;
    // Last input is 50 ms after hook_last, across the wrap: fine.
    assert!(!poll_twice(&mut policy, 0, 49, hook_last));
    // 600 ms after hook_last, across the wrap: missed.
    assert!(poll_twice(&mut policy, 1_000, 499, hook_last));
}
