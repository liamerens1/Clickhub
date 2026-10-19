use std::f64::consts::TAU;

/// Where the cursor points relative to the ring centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aim {
    /// Direction in radians, clockwise from the top, in `0.0..TAU`.
    pub angle: f64,
    /// The slot whose angle is closest to the direction, or `None` inside the dead zone.
    pub nearest: Option<usize>,
}

/// Computes the aim for a cursor offset `(dx, dy)` from the centre (same pixels as
/// `dead_zone`; screen `dy` grows downward) against slot `angles` (radians clockwise from top).
pub fn aim(dx: f64, dy: f64, angles: &[f64], dead_zone: f64) -> Aim {
    // atan2 with x/y swapped and dy negated: 0 at the top, increasing clockwise.
    let angle = dx.atan2(-dy).rem_euclid(TAU);
    let nearest = if angles.is_empty() || dx.hypot(dy) < dead_zone {
        None
    } else {
        angles
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                angular_distance(angle, **a).total_cmp(&angular_distance(angle, **b))
            })
            .map(|(index, _)| index)
    };
    Aim { angle, nearest }
}

/// Shortest distance between two angles, in `0.0..=PI`.
pub fn angular_distance(a: f64, b: f64) -> f64 {
    let difference = (a - b).rem_euclid(TAU);
    difference.min(TAU - difference)
}
