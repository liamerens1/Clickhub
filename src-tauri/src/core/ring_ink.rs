use super::{LABEL_RADIUS, ring_layout::CENTRED_BELOW};

/// Text colour for one part of the ring, chosen against what is behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// Near-black ink, for light backgrounds.
    Dark,
    /// Near-white ink, for dark backgrounds.
    Light,
}

/// Backgrounds brighter than this (0..1 luma) get dark ink. Slightly above the midpoint, because
/// light text on mid-grey reads better than dark text on mid-grey.
pub const DARK_INK_ABOVE: f32 = 0.55;

pub fn ink_for(brightness: f32) -> Ink {
    if brightness > DARK_INK_ABOVE {
        Ink::Dark
    } else {
        Ink::Light
    }
}

/// Points (offsets from the ring centre, CSS px, y down) covering a resting label at `angle`:
/// its start, middle and end. `label_width` is the label's resting width.
pub fn label_sample_points(angle: f64, radius_scale: f64, label_width: f64) -> [(f64, f64); 3] {
    let (sin, cos) = angle.sin_cos();
    let radius = LABEL_RADIUS * radius_scale;
    let (x, y) = (sin * radius, -cos * radius);
    let (start, end) = if sin.abs() < CENTRED_BELOW {
        (x - label_width / 2.0, x + label_width / 2.0)
    } else if sin > 0.0 {
        (x, x + label_width)
    } else {
        (x - label_width, x)
    };
    [(start, y), (f64::midpoint(start, end), y), (end, y)]
}

/// Points around the centre text (two lines, up to ~140 px wide).
pub const CENTRE_SAMPLE_POINTS: [(f64, f64); 5] = [
    (0.0, 0.0),
    (-50.0, -10.0),
    (50.0, -10.0),
    (-50.0, 12.0),
    (50.0, 12.0),
];

/// Average brightness over `points` using `brightness_at(dx, dy)`.
pub fn average_brightness(points: &[(f64, f64)], brightness_at: impl Fn(f64, f64) -> f32) -> f32 {
    if points.is_empty() {
        return 0.5;
    }
    let sum: f32 = points.iter().map(|&(dx, dy)| brightness_at(dx, dy)).sum();
    #[allow(clippy::cast_precision_loss)] // at most a handful of points.
    let count = points.len() as f32;
    sum / count
}
