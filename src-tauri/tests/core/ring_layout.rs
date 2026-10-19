use std::f64::consts::TAU;

use clickhub_lib::core::{
    CENTRED_BELOW, EDGE_MARGIN, LABEL_HALF_HEIGHT, MAX_RADIUS_SCALE, Room, angular_distance,
    full_ring, label_reach, layout,
};

const WIDE: f64 = 2_000.0;
const LABEL: f64 = 120.0;

fn room(left: f64, right: f64, top: f64, bottom: f64) -> Room {
    Room {
        left,
        right,
        top,
        bottom,
    }
}

/// Mirrors the layout's own fit test, so the assertions check the result, not the maths.
fn fits(angle: f64, room: Room, radius_scale: f64) -> bool {
    let reach = label_reach(radius_scale);
    let (sin, cos) = angle.sin_cos();
    let (x, y) = (sin * reach, -cos * reach);
    let (l, r) = if sin.abs() < CENTRED_BELOW {
        (x - LABEL / 2.0, x + LABEL / 2.0)
    } else if sin > 0.0 {
        (x, x + LABEL)
    } else {
        (x - LABEL, x)
    };
    l >= -room.left + EDGE_MARGIN - 1e-6
        && r <= room.right - EDGE_MARGIN + 1e-6
        && y - LABEL_HALF_HEIGHT >= -room.top + EDGE_MARGIN - 1e-6
        && y + LABEL_HALF_HEIGHT <= room.bottom - EDGE_MARGIN + 1e-6
}

#[test]
fn plenty_of_room_gives_the_normal_ring() {
    let result = layout(8, room(WIDE, WIDE, WIDE, WIDE), LABEL);
    assert_eq!(result, full_ring(8));
}

#[test]
fn right_edge_squeezes_every_label_into_view() {
    let near_right = room(WIDE, 60.0, WIDE, WIDE);
    let result = layout(8, near_right, LABEL);
    assert!(result.spacing < TAU / 8.0);
    for angle in &result.angles {
        assert!(
            fits(*angle, near_right, result.radius_scale),
            "angle {angle} should fit"
        );
    }
}

#[test]
fn corner_squeezes_into_view() {
    let corner = room(60.0, WIDE, 60.0, WIDE); // top-left corner
    let result = layout(8, corner, LABEL);
    for angle in &result.angles {
        assert!(
            fits(*angle, corner, result.radius_scale),
            "angle {angle} should fit"
        );
    }
}

#[test]
fn squeezed_rings_grow_to_spread_labels_apart() {
    // Near the top edge of a large window: half the circle is blocked.
    let near_top = room(800.0, 800.0, 60.0, 800.0);
    let result = layout(8, near_top, LABEL);
    assert!(result.radius_scale > 1.0, "scale {}", result.radius_scale);
    assert!(result.radius_scale <= MAX_RADIUS_SCALE);
    for angle in &result.angles {
        assert!(fits(*angle, near_top, result.radius_scale));
    }
}

#[test]
fn the_normal_ring_is_not_scaled() {
    let scale = layout(8, room(WIDE, WIDE, WIDE, WIDE), LABEL).radius_scale;
    assert!((scale - 1.0).abs() < f64::EPSILON);
}

#[test]
fn squeezing_keeps_clockwise_order_and_even_spacing() {
    let result = layout(8, room(WIDE, 60.0, WIDE, WIDE), LABEL);
    // Walk the slots in clockwise order starting after the blocked (right) side.
    let mut sorted: Vec<(usize, f64)> = result.angles.iter().copied().enumerate().collect();
    let cut = TAU / 4.0; // blocked side is the right, around 90 degrees
    sorted.sort_by(|a, b| {
        (a.1 - cut)
            .rem_euclid(TAU)
            .total_cmp(&(b.1 - cut).rem_euclid(TAU))
    });
    let order: Vec<usize> = sorted.iter().map(|(slot, _)| *slot).collect();
    // Neighbours stay neighbours: the sequence is a rotation of 0..8.
    let start = order[0];
    for (position, slot) in order.iter().enumerate() {
        assert_eq!(*slot, (start + position) % 8, "order {order:?}");
    }
    for pair in sorted.windows(2) {
        let gap = angular_distance(pair[0].1, pair[1].1);
        assert!((gap - result.spacing).abs() < 1e-9);
    }
}

#[test]
fn no_room_at_all_falls_back_to_the_normal_ring() {
    assert_eq!(layout(8, room(0.0, 0.0, 0.0, 0.0), LABEL), full_ring(8));
}
