//! Copying and scrolling rectangles of pixels.

use super::bitmap::{get_bit, in_bounds, plot, set_bit};
use crate::amiga::*;

/// The only minterm the game uses: plain copy of the source.
const MINTERM_COPY: ULONG = 0xC0;

/// Where a rectangle comes from, where it goes and how big it is.
struct BlitRect {
    src_x: i32,
    src_y: i32,
    dst_x: i32,
    dst_y: i32,
    width: i32,
    height: i32,
}

/// Copies one plane. Everything is read first so overlapping copies within one bitmap work.
fn copy_plane(src: &BitMap, dst: &BitMap, plane: usize, rect: &BlitRect) {
    let mut buf = vec![false; (rect.width.max(0) * rect.height.max(0)) as usize];

    for y in 0..rect.height {
        for x in 0..rect.width {
            let (px, py) = (rect.src_x + x, rect.src_y + y);
            buf[(y * rect.width + x) as usize] =
                in_bounds(src, px, py) && unsafe { get_bit(src, plane, px, py) };
        }
    }

    for y in 0..rect.height {
        for x in 0..rect.width {
            let (px, py) = (rect.dst_x + x, rect.dst_y + y);

            if in_bounds(dst, px, py) {
                unsafe { set_bit(dst, plane, px, py, buf[(y * rect.width + x) as usize]) };
            }
        }
    }
}

/// Copies the rectangle from src to dst for the planes selected by `mask`.
/// The argument list is fixed by the Amiga ABI the C code calls; it is grouped into a `BlitRect`
/// straight away.
#[unsafe(no_mangle)]
pub extern "C" fn BltBitMap(
    src: *mut BitMap,
    sx: LONG,
    sy: LONG,
    dst: *mut BitMap,
    dx: LONG,
    dy: LONG,
    w: LONG,
    h: LONG,
    minterm: ULONG,
    mask: ULONG,
    _temp: PLANEPTR,
) -> LONG {
    debug_assert_eq!(minterm, MINTERM_COPY, "only plain copy is implemented");

    let (src, dst) = unsafe { (&*src, &*dst) };
    let rect = BlitRect {
        src_x: sx,
        src_y: sy,
        dst_x: dx,
        dst_y: dy,
        width: w,
        height: h,
    };

    let mut copied = 0;

    for plane in 0..src.Depth.min(dst.Depth) as usize {
        // The plane mask says which planes to copy.
        if mask >> plane & 1 == 0 || src.Planes[plane].is_null() || dst.Planes[plane].is_null() {
            continue;
        }

        copy_plane(src, dst, plane, &rect);
        copied += 1;
    }

    copied
}

/// Reads a rectangle into one byte per pixel (bit `p` of the byte is plane `p`).
fn read_pixels(bm: &BitMap, x0: i32, y0: i32, w: i32, h: i32) -> Vec<u8> {
    let mut pixels = vec![0u8; (w * h) as usize];

    for y in 0..h {
        for x in 0..w {
            for p in 0..bm.Depth as usize {
                if in_bounds(bm, x0 + x, y0 + y) && unsafe { get_bit(bm, p, x0 + x, y0 + y) } {
                    // Builds the pixel's colour index one bitplane at a time.
                    pixels[(y * w + x) as usize] |= 1 << p;
                }
            }
        }
    }

    pixels
}

/// Scrolls the rectangle's contents; the exposed strip is filled with the background pen.
#[unsafe(no_mangle)]
pub extern "C" fn ScrollRaster(
    rp: *mut RastPort,
    dx: LONG,
    dy: LONG,
    x0: LONG,
    y0: LONG,
    x1: LONG,
    y1: LONG,
) {
    let rp = unsafe { &*rp };

    if rp.BitMap.is_null() {
        return;
    }

    let bm = unsafe { &*rp.BitMap };
    let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
    let pixels = read_pixels(bm, x0, y0, w, h);

    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = (x + dx, y + dy);
            let inside = sx >= 0 && sy >= 0 && sx < w && sy < h;
            let v = if inside {
                pixels[(sy * w + sx) as usize]
            } else {
                rp.BgPen
            };

            unsafe { plot(bm, x0 + x, y0 + y, v, false) };
        }
    }
}
