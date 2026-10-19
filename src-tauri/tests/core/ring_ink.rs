use std::f64::consts::FRAC_PI_2;

use clickhub_lib::core::{
    DARK_INK_ABOVE, Ink, LABEL_RADIUS, average_brightness, ink_for, label_sample_points,
};

#[test]
fn light_backgrounds_get_dark_ink() {
    assert_eq!(ink_for(0.95), Ink::Dark);
    assert_eq!(ink_for(DARK_INK_ABOVE + 0.01), Ink::Dark);
}

#[test]
fn dark_and_mid_backgrounds_get_light_ink() {
    assert_eq!(ink_for(0.05), Ink::Light);
    assert_eq!(ink_for(0.5), Ink::Light);
}

#[test]
fn samples_cover_a_right_hand_label_from_its_anchor_outward() {
    let points = label_sample_points(FRAC_PI_2, 1.0, 100.0);
    assert!((points[0].0 - LABEL_RADIUS).abs() < 1e-9);
    assert!((points[2].0 - (LABEL_RADIUS + 100.0)).abs() < 1e-9);
    assert!(points.iter().all(|point| point.1.abs() < 1e-9));
}

#[test]
fn samples_scale_with_the_ring() {
    let points = label_sample_points(FRAC_PI_2, 1.5, 100.0);
    assert!((points[0].0 - LABEL_RADIUS * 1.5).abs() < 1e-9);
}

#[test]
fn averages_brightness_over_points() {
    let points = [(0.0, 0.0), (10.0, 0.0)];
    let average = average_brightness(&points, |dx, _| if dx > 5.0 { 1.0 } else { 0.0 });
    assert!((average - 0.5).abs() < f32::EPSILON);
    assert!((average_brightness(&[], |_, _| 1.0) - 0.5).abs() < f32::EPSILON);
}
