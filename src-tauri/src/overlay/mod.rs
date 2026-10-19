//! Overlay windows: one per monitor, created once, never hidden, resized or focused, moved only
//! within their own monitor. Sends ring events to the overlay page; never waits on it.

mod controller;
mod view;

pub use controller::Overlays;
pub use view::{AimPayload, ClosePayload, InkName, OpenPayload, RingView};
