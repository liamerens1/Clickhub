use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use tauri::{
    AppHandle, Emitter, Listener, LogicalSize, PhysicalPosition, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

use super::{AimPayload, ClosePayload, InkName, OpenPayload, RingView};
use crate::core::{
    Aim, CENTRE_SAMPLE_POINTS, Ink, Layout, Room, average_brightness, ink_for, label_sample_points,
    layout,
};
use crate::platform::{
    LumaGrid, keep_on_top, make_overlay_window, place_overlay, qpc_now, qpc_to_micros, sample_luma,
};

/// Widest label at full zoom, in CSS pixels, until the page measures the real one.
const DEFAULT_LABEL_WIDTH: f64 = 160.0;
/// Labels are measured at full zoom; at rest they are this much smaller (`maxZoom` in the page).
const MAX_ZOOM: f64 = 1.32;
/// Brightness sampling grid cell size, in physical pixels.
const SAMPLE_CELL_PX: i32 = 20;

/// Overlay window size in CSS pixels: room for labels left and right of a ~150 px ring plus the
/// focus zoom. Never changes after creation.
const WINDOW_WIDTH: f64 = 720.0;
const WINDOW_HEIGHT: f64 = 460.0;
const ENTRY: &str = "overlay/index.html";

/// One pre-created overlay window per monitor (`docs/architecture.md`, hard rule 2). The ring
/// opens in the window of the monitor under the cursor; windows only ever move within their own
/// monitor, so no DPI change can resize them.
pub struct Overlays {
    windows: Vec<OverlayWindow>,
    view: Mutex<RingView>,
    open: Mutex<Option<OpenRing>>,
    seq: AtomicU64,
}

struct OverlayWindow {
    label: String,
    window: WebviewWindow,
    /// Monitor rectangle in physical pixels.
    monitor: (i32, i32, i32, i32),
    /// Window size in physical pixels, fixed at creation.
    size: (i32, i32),
    scale: f64,
    /// Widest label at full zoom (CSS px), reported by the page on `overlay:extent`.
    label_width: Mutex<f64>,
}

/// What the pipeline needs back from [`Overlays::open`].
pub struct Opened {
    /// DPI scale of the monitor the ring opened on.
    pub scale: f64,
    pub layout: Layout,
}

#[derive(Debug, Clone, Copy)]
struct OpenRing {
    seq: u64,
    window: usize,
    hook_to_emit_us: u64,
    sample_us: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadyPayload {
    label: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtentPayload {
    label: String,
    max_label_width: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FramePayload {
    seq: u64,
    emit_to_frame_ms: f64,
}

impl Overlays {
    /// Creates one overlay per connected monitor. Call once, from setup.
    ///
    /// # Errors
    ///
    /// Fails if monitors cannot be listed or a window cannot be created.
    pub fn create(app: &AppHandle, view: RingView) -> tauri::Result<Arc<Self>> {
        let mut windows = Vec::new();
        for (index, monitor) in app.available_monitors()?.into_iter().enumerate() {
            let label = format!("overlay-{index}");
            let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(ENTRY.into()))
                .title("ClickHub ring")
                .transparent(true)
                .decorations(false)
                .shadow(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .focused(false)
                .focusable(false)
                .resizable(false)
                // Created hidden only once, so it can be moved onto its monitor and sized at that
                // monitor's DPI before it is ever shown. From then on it is never hidden.
                .visible(false)
                .build()?;
            let position = monitor.position();
            let monitor_size = monitor.size();
            window.set_position(PhysicalPosition::new(
                position.x + i32::try_from(monitor_size.width / 4).unwrap_or(0),
                position.y + i32::try_from(monitor_size.height / 4).unwrap_or(0),
            ))?;
            window.set_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))?;
            window.set_ignore_cursor_events(true)?;
            if let Err(error) = window
                .hwnd()
                .map_err(|e| e.to_string())
                .and_then(|hwnd| make_overlay_window(hwnd.0))
            {
                tracing::warn!(%error, label, "could not set the overlay window styles");
            }
            window.show()?;
            let size = window.outer_size()?;
            tracing::info!(
                label,
                monitor = ?monitor.name(),
                scale = monitor.scale_factor(),
                size = ?size,
                "overlay created"
            );
            windows.push(OverlayWindow {
                label,
                window,
                monitor: (
                    position.x,
                    position.y,
                    i32::try_from(monitor_size.width).unwrap_or(i32::MAX),
                    i32::try_from(monitor_size.height).unwrap_or(i32::MAX),
                ),
                size: (
                    i32::try_from(size.width).unwrap_or(0),
                    i32::try_from(size.height).unwrap_or(0),
                ),
                scale: monitor.scale_factor(),
                label_width: Mutex::new(DEFAULT_LABEL_WIDTH),
            });
        }

        let overlays = Arc::new(Self {
            windows,
            view: Mutex::new(view),
            open: Mutex::new(None),
            seq: AtomicU64::new(0),
        });
        overlays.listen(app);
        Ok(overlays)
    }

    fn listen(self: &Arc<Self>, app: &AppHandle) {
        let ready = Arc::clone(self);
        app.listen("overlay:ready", move |event| {
            match serde_json::from_str::<ReadyPayload>(event.payload()) {
                Ok(payload) => ready.send_view_to(&payload.label),
                Err(error) => tracing::warn!(%error, "bad overlay:ready payload"),
            }
        });
        let extents = Arc::clone(self);
        app.listen("overlay:extent", move |event| {
            match serde_json::from_str::<ExtentPayload>(event.payload()) {
                Ok(payload) => {
                    if let Some(window) = extents.window(&payload.label)
                        && payload.max_label_width.is_finite()
                        && payload.max_label_width > 0.0
                    {
                        *lock(&window.label_width) = payload.max_label_width;
                    }
                }
                Err(error) => tracing::warn!(%error, "bad overlay:extent payload"),
            }
        });
        let frames = Arc::clone(self);
        app.listen("overlay:frame", move |event| {
            match serde_json::from_str::<FramePayload>(event.payload()) {
                Ok(payload) => frames.log_frame(&payload),
                Err(error) => tracing::warn!(%error, "bad overlay:frame payload"),
            }
        });
    }

    /// Replaces the ring contents (config reload) and re-sends them to every overlay.
    pub fn set_view(&self, view: RingView) {
        *lock(&self.view) = view;
        for window in &self.windows {
            self.send_view_to(&window.label);
        }
    }

    fn send_view_to(&self, label: &str) {
        let view = lock(&self.view).clone();
        if let Err(error) = self.window(label).map_or(Ok(()), |window| {
            window.window.emit_to(label, "ring:config", view)
        }) {
            tracing::warn!(%error, label, "could not send the ring config");
        }
    }

    /// Opens the ring centred on the physical screen point `(x, y)`, laid out so every label
    /// fits inside the visible window (squeezed near edges). `hook_qpc` is the hook timestamp of
    /// the triggering click.
    pub fn open(&self, x: i32, y: i32, hook_qpc: i64) -> Option<Opened> {
        let (index, overlay) = self.window_for(x, y)?;
        let (mx, my, mw, mh) = overlay.monitor;
        let (w, h) = overlay.size;
        let wx = (x - w / 2).clamp(mx, (mx + mw - w).max(mx));
        let wy = (y - h / 2).clamp(my, (my + mh - h).max(my));
        // One SetWindowPos: move within the monitor and re-assert topmost (see place_overlay).
        let placed = overlay
            .window
            .hwnd()
            .map_err(|error| error.to_string())
            .and_then(|hwnd| place_overlay(hwnd.0, wx, wy));
        if let Err(error) = placed {
            tracing::warn!(%error, "could not place the overlay; falling back to Tauri");
            if let Err(error) = overlay.window.set_position(PhysicalPosition::new(wx, wy)) {
                tracing::warn!(%error, "could not move the overlay");
            }
        }

        // Room from the ring centre to each edge of the window (which lies inside the monitor).
        let scale = overlay.scale;
        let room = Room {
            left: f64::from(x - wx) / scale,
            right: f64::from(wx + w - x) / scale,
            top: f64::from(y - wy) / scale,
            bottom: f64::from(wy + h - y) / scale,
        };
        let slot_count = lock(&self.view).slots.len();
        let label_width = *lock(&overlay.label_width);
        let ring_layout = layout(slot_count, room, label_width);

        // Pick dark or light ink per slot from the screen behind the ring (the overlay itself is
        // transparent at this moment). Falls back to light ink if sampling fails.
        let sample_started = qpc_now();
        let grid = sample_luma(wx, wy, w, h, cells(w), cells(h));
        let sample_us = qpc_to_micros(qpc_now() - sample_started);
        let (inks, centre_ink) =
            choose_inks(grid.as_ref(), room, &ring_layout, label_width / MAX_ZOOM);

        let seq = self.seq.fetch_add(1, Ordering::Relaxed) + 1;
        let payload = OpenPayload {
            seq,
            cx: room.left,
            cy: room.top,
            angles: ring_layout.angles.clone(),
            spacing: ring_layout.spacing,
            radius_scale: ring_layout.radius_scale,
            inks,
            centre_ink,
            emitted_ms: epoch_ms(),
        };
        if let Err(error) = overlay
            .window
            .emit_to(overlay.label.as_str(), "ring:open", payload)
        {
            tracing::warn!(%error, "could not open the ring");
        }
        let hook_to_emit_us = qpc_to_micros(qpc_now() - hook_qpc);
        *lock(&self.open) = Some(OpenRing {
            seq,
            window: index,
            hook_to_emit_us,
            sample_us,
        });
        Some(Opened {
            scale,
            layout: ring_layout,
        })
    }

    /// Re-asserts the open ring's window on top of everything (called periodically while open).
    pub fn keep_open_ring_on_top(&self) {
        let Some(open) = *lock(&self.open) else {
            return;
        };
        if let Some(overlay) = self.windows.get(open.window)
            && let Err(error) = overlay
                .window
                .hwnd()
                .map_err(|error| error.to_string())
                .and_then(|hwnd| keep_on_top(hwnd.0))
        {
            tracing::debug!(%error, "could not keep the ring on top");
        }
    }

    pub fn aim(&self, aim: Aim) {
        let Some(open) = *lock(&self.open) else {
            return;
        };
        self.emit_open(open, "ring:aim", AimPayload::new(open.seq, aim));
    }

    pub fn close(&self, fired: Option<usize>) {
        let Some(open) = lock(&self.open).take() else {
            return;
        };
        self.emit_open(
            open,
            "ring:close",
            ClosePayload {
                seq: open.seq,
                fired,
            },
        );
    }

    fn emit_open<P: serde::Serialize + Clone>(&self, open: OpenRing, event: &str, payload: P) {
        if let Some(overlay) = self.windows.get(open.window)
            && let Err(error) = overlay
                .window
                .emit_to(overlay.label.as_str(), event, payload)
        {
            tracing::warn!(%error, event, "could not message the overlay");
        }
    }

    fn log_frame(&self, frame: &FramePayload) {
        let open = lock(&self.open).filter(|open| open.seq == frame.seq);
        tracing::info!(
            seq = frame.seq,
            hook_to_emit_us = open.map(|open| open.hook_to_emit_us),
            sample_us = open.map(|open| open.sample_us),
            emit_to_frame_ms = format!("{:.2}", frame.emit_to_frame_ms),
            "ring opened"
        );
    }

    fn window_for(&self, x: i32, y: i32) -> Option<(usize, &OverlayWindow)> {
        self.windows
            .iter()
            .enumerate()
            .find(|(_, window)| {
                let (mx, my, mw, mh) = window.monitor;
                x >= mx && x < mx + mw && y >= my && y < my + mh
            })
            .or_else(|| self.windows.iter().enumerate().next())
    }

    fn window(&self, label: &str) -> Option<&OverlayWindow> {
        self.windows.iter().find(|window| window.label == label)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn epoch_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |elapsed| elapsed.as_secs_f64() * 1_000.0)
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
