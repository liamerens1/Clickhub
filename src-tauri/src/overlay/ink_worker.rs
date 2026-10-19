use std::sync::mpsc::{Sender, channel};
use std::thread;

use tauri::{Emitter, WebviewWindow};

use super::{InkName, InkPayload};
use crate::core::{
    CENTRE_SAMPLE_POINTS, Ink, Layout, Room, average_brightness, ink_for, label_sample_points,
};
use crate::platform::{LumaGrid, qpc_now, qpc_to_micros, sample_luma};

/// Brightness sampling grid cell size, in physical pixels.
const SAMPLE_CELL_PX: i32 = 20;

/// One request to pick inks for a ring that just opened.
pub struct InkJob {
    pub seq: u64,
    pub window: WebviewWindow,
    pub label: String,
    /// Window rectangle in physical pixels: `(x, y, width, height)`.
    pub rect: (i32, i32, i32, i32),
    pub room: Room,
    pub layout: Layout,
    /// A resting label's width, in CSS pixels.
    pub label_width: f64,
}

/// Picks dark or light ink off the hot path. Reading screen pixels back from the compositor takes
/// 6-11 ms, measured live; doing it before `ring:open` pushed trigger-to-frame past 16 ms. The
/// ring opens first, and `ring:ink` follows ~10 ms later, while the lines have barely started
/// growing and well before any label pops in (~90 ms into the bloom).
pub struct InkWorker {
    jobs: Sender<InkJob>,
}

impl InkWorker {
    /// # Errors
    ///
    /// Fails only if the OS cannot start a thread.
    pub fn spawn() -> std::io::Result<Self> {
        let (jobs, receiver) = channel::<InkJob>();
        thread::Builder::new()
            .name("clickhub-ink".into())
            .spawn(move || {
                for job in receiver {
                    run(&job);
                }
            })?;
        Ok(Self { jobs })
    }

    /// Queues `job`. Never blocks.
    pub fn request(&self, job: InkJob) {
        if self.jobs.send(job).is_err() {
            tracing::warn!("ink thread is gone; ring keeps the theme ink");
        }
    }
}

fn run(job: &InkJob) {
    let (x, y, width, height) = job.rect;
    let started = qpc_now();
    let grid = sample_luma(x, y, width, height, cells(width), cells(height));
    let sample_us = qpc_to_micros(qpc_now() - started);
    let (inks, centre_ink) = choose_inks(grid.as_ref(), job.room, &job.layout, job.label_width);
    let payload = InkPayload {
        seq: job.seq,
        inks,
        centre_ink,
    };
    if let Err(error) = job.window.emit_to(job.label.as_str(), "ring:ink", payload) {
        tracing::warn!(%error, "could not send the ring ink");
    }
    tracing::info!(
        seq = job.seq,
        sample_us,
        sampled = grid.is_some(),
        "ring ink"
    );
}

/// Grid cells along a side of `pixels` physical pixels.
fn cells(pixels: i32) -> usize {
    usize::try_from((pixels / SAMPLE_CELL_PX).max(4)).unwrap_or(4)
}

/// Ink per slot and for the centre text. `room` doubles as the centre's position in the window
/// (CSS px); `label_width` is a resting label's width.
fn choose_inks(
    grid: Option<&LumaGrid>,
    room: Room,
    ring: &Layout,
    label_width: f64,
) -> (Vec<InkName>, InkName) {
    let Some(grid) = grid else {
        let fallback = InkName::from(Ink::Light);
        return (vec![fallback; ring.angles.len()], fallback);
    };
    let width = room.left + room.right;
    let height = room.top + room.bottom;
    let brightness_at =
        |dx: f64, dy: f64| grid.at((room.left + dx) / width, (room.top + dy) / height);
    let inks = ring
        .angles
        .iter()
        .map(|&angle| {
            let points = label_sample_points(angle, ring.radius_scale, label_width);
            InkName::from(ink_for(average_brightness(&points, brightness_at)))
        })
        .collect();
    let centre = InkName::from(ink_for(average_brightness(
        &CENTRE_SAMPLE_POINTS,
        brightness_at,
    )));
    (inks, centre)
}
