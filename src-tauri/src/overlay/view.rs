use serde::Serialize;

use crate::config::Ring;
use crate::core::{Aim, Ink};

/// What the overlay page needs to pre-render the ring. Sent as `ring:config`.
/// Mirrors `ringConfigSchema` in `src/shared/ipc/overlay-events.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RingView {
    pub slots: Vec<SlotView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotView {
    pub label: String,
    /// Shown under the label in the centre, e.g. "Ctrl+C" or "Open app".
    pub hint: String,
}

impl RingView {
    pub fn from_ring(ring: &Ring) -> Self {
        Self {
            slots: ring
                .slots
                .iter()
                .map(|slot| SlotView {
                    label: slot.label.trim().to_owned(),
                    hint: slot.action.hint(),
                })
                .collect(),
        }
    }
}

/// `ring:open`: show the ring centred at `(cx, cy)` in the window's CSS pixels, with each slot
/// at `angles[i]` (radians clockwise from the top; squeezed near screen edges).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPayload {
    pub seq: u64,
    pub cx: f64,
    pub cy: f64,
    pub angles: Vec<f64>,
    /// Angle between neighbouring slots; the focus zoom spreads relative to it.
    pub spacing: f64,
    /// Stretches lines and labels outward (1.0 normally, more when squeezed).
    pub radius_scale: f64,
    /// Wall-clock milliseconds when Rust emitted the event, for latency measurement.
    pub emitted_ms: f64,
}

/// `ring:ink`: dark or light ink per slot and for the centre, sent shortly after `ring:open`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InkPayload {
    pub seq: u64,
    pub inks: Vec<InkName>,
    pub centre_ink: InkName,
}

/// Ink as sent to the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InkName {
    Dark,
    Light,
}

impl From<Ink> for InkName {
    fn from(ink: Ink) -> Self {
        match ink {
            Ink::Dark => Self::Dark,
            Ink::Light => Self::Light,
        }
    }
}

/// `ring:aim`: the continuous direction (radians clockwise from the top) and the slot it picks.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AimPayload {
    pub seq: u64,
    pub angle: f64,
    pub nearest: Option<usize>,
}

impl AimPayload {
    pub fn new(seq: u64, aim: Aim) -> Self {
        Self {
            seq,
            angle: aim.angle,
            nearest: aim.nearest,
        }
    }
}

/// `ring:close`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosePayload {
    pub seq: u64,
    pub fired: Option<usize>,
}
