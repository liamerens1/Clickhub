use std::f64::consts::TAU;

/// Ring geometry the layout must respect, in CSS pixels. Mirrors `GEOMETRY` in
/// `src/overlay/ring-view.ts`.
pub const LABEL_RADIUS: f64 = 150.0;
/// How far the focused label slides outward.
pub const LABEL_PUSH: f64 = 20.0;
/// Half the height of a label at full zoom, plus a little air.
pub const LABEL_HALF_HEIGHT: f64 = 14.0;
/// Keep labels this far from the visible edge.
pub const EDGE_MARGIN: f64 = 8.0;
/// A label is centred on its point only when its direction is this close to straight up or down
/// (|sin| below this, about 4.6 degrees); otherwise it hangs to the side away from the centre, so
/// neighbouring labels near the top or bottom never collide. Mirrored in `ring-view.ts`.
pub const CENTRED_BELOW: f64 = 0.08;
/// When squeezed, the ring grows up to this much so labels get room between them.
pub const MAX_RADIUS_SCALE: f64 = 1.6;
const SAMPLES: usize = 360;
const SAMPLES_F64: f64 = 360.0;

/// Room around the ring centre, in CSS pixels, to each visible edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Room {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

/// Where each slot sits: angles in radians, clockwise from the top. `spacing` is the angle
/// between neighbours (smaller when squeezed); the focus zoom scales with it. `radius_scale`
/// stretches the lines and labels outward (1.0 normally, more when squeezed).
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub angles: Vec<f64>,
    pub spacing: f64,
    pub radius_scale: f64,
}

/// The normal layout: slot 0 at the top, evenly spaced clockwise.
pub fn full_ring(slot_count: usize) -> Layout {
    #[allow(clippy::cast_precision_loss)] // slot counts are tiny (max 12).
    let spacing = TAU / slot_count.max(1) as f64;
    #[allow(clippy::cast_precision_loss)]
    let angles = (0..slot_count).map(|i| i as f64 * spacing).collect();
    Layout {
        angles,
        spacing,
        radius_scale: 1.0,
    }
}

/// Distance from the centre to a label's anchor at full focus, for a given radius scale.
pub fn label_reach(radius_scale: f64) -> f64 {
    LABEL_RADIUS * radius_scale + LABEL_PUSH
}

/// Lays the slots out so every label fits inside `room`. With room all round this is the
/// normal [`full_ring`]. Near an edge or corner the slots spread evenly over the largest arc
/// where labels fit, keeping their clockwise order and putting the cut on the blocked side, and
/// the ring grows (up to [`MAX_RADIUS_SCALE`]) when that gives the labels more room.
/// `max_label_width` is the widest label at full zoom.
pub fn layout(slot_count: usize, room: Room, max_label_width: f64) -> Layout {
    let normal = full_ring(slot_count);
    let Some(base) = arrange(slot_count, room, max_label_width, 1.0) else {
        return normal;
    };
    let scale = (normal.spacing / base.spacing).clamp(1.0, MAX_RADIUS_SCALE);
    // Bigger rings lose some arc to the edges; keep the larger one only if neighbouring labels
    // really end up further apart.
    match arrange(slot_count, room, max_label_width, scale) {
        Some(grown) if label_reach(scale) * grown.spacing > label_reach(1.0) * base.spacing => {
            grown
        }
        _ => base,
    }
}

/// The squeezed arrangement for one radius scale, or `None` when the full ring fits (or nothing
/// fits at all, in which case the full ring is the least-bad option).
fn arrange(slot_count: usize, room: Room, width: f64, radius_scale: f64) -> Option<Layout> {
    let reach = label_reach(radius_scale);
    let usable: Vec<bool> = (0..SAMPLES)
        .map(|i| label_fits(sample_angle(i), room, width, reach))
        .collect();
    if slot_count == 0 || usable.iter().all(|&fits| fits) || !usable.iter().any(|&fits| fits) {
        return None;
    }

    // Largest run of usable samples, walking the circle once from a blocked sample.
    let blocked = usable.iter().position(|&fits| !fits)?;
    let (mut best_start, mut best_len, mut run_start, mut run_len) = (0, 0, 0, 0);
    for step in 1..=SAMPLES {
        let index = (blocked + step) % SAMPLES;
        if usable[index] {
            if run_len == 0 {
                run_start = index;
            }
            run_len += 1;
            if run_len > best_len {
                (best_start, best_len) = (run_start, run_len);
            }
        } else {
            run_len = 0;
        }
    }

    let arc_start = sample_angle(best_start);
    let arc = f64::from(u16::try_from(best_len).unwrap_or(u16::MAX)) / SAMPLES_F64 * TAU;
    // The cut goes in the middle of the blocked side, opposite the free arc.
    let cut = (arc_start + arc / 2.0 + TAU / 2.0) % TAU;

    let normal = full_ring(slot_count);
    let mut order: Vec<usize> = (0..slot_count).collect();
    order.sort_by(|&a, &b| {
        let ka = (normal.angles[a] - cut).rem_euclid(TAU);
        let kb = (normal.angles[b] - cut).rem_euclid(TAU);
        ka.total_cmp(&kb)
    });

    #[allow(clippy::cast_precision_loss)]
    let spacing = arc / slot_count as f64;
    let mut angles = vec![0.0; slot_count];
    for (position, &slot) in order.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let offset = (position as f64 + 0.5) * spacing;
        angles[slot] = (arc_start + offset) % TAU;
    }
    Some(Layout {
        angles,
        spacing,
        radius_scale,
    })
}

fn sample_angle(index: usize) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let angle = index as f64 / SAMPLES as f64 * TAU;
    angle
}

/// Whether a label at `angle`, `reach` from the centre, fits inside `room`. Screen x grows
/// right, y grows down.
fn label_fits(angle: f64, room: Room, width: f64, reach: f64) -> bool {
    let (sin, cos) = angle.sin_cos();
    let x = sin * reach;
    let y = -cos * reach;
    let (left, right) = if sin.abs() < CENTRED_BELOW {
        (x - width / 2.0, x + width / 2.0)
    } else if sin > 0.0 {
        (x, x + width)
    } else {
        (x - width, x)
    };
    left >= -room.left + EDGE_MARGIN
        && right <= room.right - EDGE_MARGIN
        && y - LABEL_HALF_HEIGHT >= -room.top + EDGE_MARGIN
        && y + LABEL_HALF_HEIGHT <= room.bottom - EDGE_MARGIN
}
