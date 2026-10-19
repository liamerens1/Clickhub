use std::ffi::c_void;

use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, GetDC, HALFTONE, HDC, HGDIOBJ, ReleaseDC, SRCCOPY, SelectObject,
    SetBrushOrgEx, SetStretchBltMode, StretchBlt,
};

/// Brightness of a screen area on a coarse grid: `values[row * cols + col]` in `0.0..=1.0`.
#[derive(Debug, Clone)]
pub struct LumaGrid {
    pub cols: usize,
    pub rows: usize,
    pub values: Vec<f32>,
}

impl LumaGrid {
    /// Brightness at a point given as fractions of the sampled area (clamped to it).
    pub fn at(&self, fx: f64, fy: f64) -> f32 {
        let col = cell(fx, self.cols);
        let row = cell(fy, self.rows);
        self.values
            .get(row * self.cols + col)
            .copied()
            .unwrap_or(0.5)
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn cell(fraction: f64, count: usize) -> usize {
    ((fraction.clamp(0.0, 0.999_999) * count as f64) as usize).min(count.saturating_sub(1))
}

/// Downscales the physical screen rectangle `(x, y, width, height)` into a `cols` x `rows` grid
/// in one GDI `StretchBlt` (with averaging) and returns each cell's brightness. Nothing is kept or
/// written anywhere; the pixels are only averaged. `None` if any GDI call fails.
pub fn sample_luma(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    cols: usize,
    rows: usize,
) -> Option<LumaGrid> {
    let (c, r) = (i32::try_from(cols).ok()?, i32::try_from(rows).ok()?);
    if c <= 0 || r <= 0 || width <= 0 || height <= 0 {
        return None;
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).ok()?,
            biWidth: c,
            biHeight: -r, // negative: top-down rows
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..BITMAPINFOHEADER::default()
        },
        ..BITMAPINFO::default()
    };

    // SAFETY: standard GDI sequence; every handle created here is released below on all paths.
    unsafe {
        let screen = GetDC(None);
        if screen.is_invalid() {
            return None;
        }
        let memory = CreateCompatibleDC(Some(screen));
        let mut bits: *mut c_void = std::ptr::null_mut();
        let result = CreateDIBSection(
            Some(memory),
            &raw const info,
            DIB_RGB_COLORS,
            &raw mut bits,
            None,
            0,
        )
        .ok()
        .and_then(|bitmap| {
            let previous = SelectObject(memory, HGDIOBJ(bitmap.0));
            let grid = copy_and_read(memory, screen, (x, y, width, height), (c, r), bits);
            SelectObject(memory, previous);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            grid
        });
        let _ = DeleteDC(memory);
        ReleaseDC(None, screen);
        result.map(|values| LumaGrid { cols, rows, values })
    }
}

/// # Safety
///
/// `memory` must have a `size.0` x `size.1` 32-bit top-down DIB selected whose pixels are at
/// `bits`; `screen` must be a valid screen DC.
unsafe fn copy_and_read(
    memory: HDC,
    screen: HDC,
    area: (i32, i32, i32, i32),
    grid: (i32, i32),
    bits: *mut c_void,
) -> Option<Vec<f32>> {
    let (left, top, width, height) = area;
    let (cols, rows) = grid;
    // SAFETY: guaranteed by the caller.
    unsafe {
        SetStretchBltMode(memory, HALFTONE);
        let _ = SetBrushOrgEx(memory, 0, 0, None);
        if !StretchBlt(
            memory,
            0,
            0,
            cols,
            rows,
            Some(screen),
            left,
            top,
            width,
            height,
            SRCCOPY,
        )
        .as_bool()
            || bits.is_null()
        {
            return None;
        }
        let count = usize::try_from(cols * rows).ok()?;
        let pixels = std::slice::from_raw_parts(bits.cast::<u8>(), count * 4);
        Some(
            pixels
                .chunks_exact(4)
                .map(|bgra| {
                    // Rec. 709 luma from 8-bit sRGB channels (BGRA order).
                    let (blue, green, red) =
                        (f32::from(bgra[0]), f32::from(bgra[1]), f32::from(bgra[2]));
                    (0.2126 * red + 0.7152 * green + 0.0722 * blue) / 255.0
                })
                .collect(),
        )
    }
}
