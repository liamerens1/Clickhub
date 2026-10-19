use std::f64::consts::{PI, TAU};

use clickhub_lib::core::{aim, angular_distance, full_ring};

const DEAD: f64 = 40.0;

fn eight() -> Vec<f64> {
    full_ring(8).angles
}

#[test]
fn directions_pick_slots_clockwise_from_the_top() {
    // 8 slots: 0 top, 2 right, 4 bottom, 6 left (screen y grows downward).
    let angles = eight();
    assert_eq!(aim(0.0, -100.0, &angles, DEAD).nearest, Some(0));
    assert_eq!(aim(100.0, 0.0, &angles, DEAD).nearest, Some(2));
    assert_eq!(aim(0.0, 100.0, &angles, DEAD).nearest, Some(4));
    assert_eq!(aim(-100.0, 0.0, &angles, DEAD).nearest, Some(6));
    assert_eq!(aim(70.0, -70.0, &angles, DEAD).nearest, Some(1));
}

#[test]
fn just_left_of_top_wraps_to_slot_zero() {
    let result = aim(-1.0, -100.0, &eight(), DEAD);
    assert_eq!(result.nearest, Some(0));
    assert!(result.angle > TAU - 0.05 && result.angle < TAU);
}

#[test]
fn reports_the_continuous_angle() {
    let result = aim(100.0, 0.0, &eight(), DEAD);
    assert!((result.angle - PI / 2.0).abs() < 1e-9);
}

#[test]
fn dead_zone_picks_nothing() {
    assert_eq!(aim(10.0, -10.0, &eight(), DEAD).nearest, None);
    assert_eq!(aim(0.0, -40.0, &eight(), DEAD).nearest, Some(0));
}

#[test]
fn picks_the_nearest_of_arbitrary_angles() {
    // A squeezed layout: three slots crowded on the left (radians clockwise from top).
    let angles = vec![4.0, 4.4, 4.8];
    // Left is 3*PI/2 (~4.71): closest is 4.8.
    assert_eq!(aim(-100.0, 0.0, &angles, DEAD).nearest, Some(2));
    // Down is PI (~3.14): closest is 4.0.
    assert_eq!(aim(0.0, 100.0, &angles, DEAD).nearest, Some(0));
    // Up-left is 7*PI/4 (~5.50): closest is 4.8; across the wrap the others are further.
    assert_eq!(aim(-70.0, -70.0, &angles, DEAD).nearest, Some(2));
    assert_eq!(aim(100.0, 0.0, &[], DEAD).nearest, None);
}

#[test]
fn angular_distance_wraps() {
    assert!((angular_distance(0.1, TAU - 0.1) - 0.2).abs() < 1e-9);
    assert!((angular_distance(0.0, PI) - PI).abs() < 1e-9);
}
