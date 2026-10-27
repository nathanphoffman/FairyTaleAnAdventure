//! Rastports and drawing: pens, fills, lines, blits and scrolling.

use super::bitmap::plot;
use crate::amiga::*;
use std::ptr::null_mut;

pub fn init_rast_port(rp: *mut RastPort) {
    let rp = unsafe { &mut *rp };
    rp.Layer = null_mut();
    rp.BitMap = null_mut();
    rp.AreaInfo = null_mut();
    rp.TmpRas = null_mut();
    rp.FgPen = 1;
    rp.BgPen = 0;
    rp.DrawMode = JAM2;
    rp.cp_x = 0;
    rp.cp_y = 0;
    rp.Font = null_mut();
}

pub fn set_a_pen(rp: *mut RastPort, pen: ULONG) {
    unsafe { (*rp).FgPen = pen as u8 }
}

pub fn set_b_pen(rp: *mut RastPort, pen: ULONG) {
    unsafe { (*rp).BgPen = pen as u8 }
}

pub fn set_draw_mode(rp: *mut RastPort, mode: ULONG) {
    unsafe { (*rp).DrawMode = mode as u8 }
}

/// Fills the whole bitmap with one pen.
pub fn set_rast(rp: *mut RastPort, pen: ULONG) {
    let rp = unsafe { &*rp };

    if rp.BitMap.is_null() {
        return;
    }

    let bm = unsafe { &*rp.BitMap };
    let size = bm.BytesPerRow as usize * bm.Rows as usize;

    for p in 0..bm.Depth as usize {
        if !bm.Planes[p].is_null() {
            // A plane is filled with all ones if the pen has that bit set, and zeros otherwise.
            let v = if pen >> p & 1 != 0 { 0xff } else { 0 };

            unsafe { std::ptr::write_bytes(bm.Planes[p], v, size) };
        }
    }
}

/// The Amiga `RectFill`: fills a rectangle with the pen colour (or flips it, in COMPLEMENT mode),
/// cut to the bitmap.
pub fn rect_fill(rp: *mut RastPort, x0: LONG, y0: LONG, x1: LONG, y1: LONG) {
    let rp = unsafe { &*rp };

    if rp.BitMap.is_null() {
        return;
    }

    let bm = unsafe { &*rp.BitMap };

    // Masks with COMPLEMENT, keeping only the bits that matter here.
    let complement = rp.DrawMode & COMPLEMENT != 0;

    for y in y0.max(0)..=y1.min(bm.Rows as i32 - 1) {
        for x in x0.max(0)..=x1 {
            unsafe { plot(bm, x, y, rp.FgPen, complement) };
        }
    }
}

pub fn move_pen(rp: *mut RastPort, x: LONG, y: LONG) {
    let rp = unsafe { &mut *rp };
    rp.cp_x = x as WORD;
    rp.cp_y = y as WORD;
}

/// Line from the current pen position to (x, y), then moves the pen there.
pub fn draw_line(rp: *mut RastPort, x: LONG, y: LONG) {
    let rp = unsafe { &mut *rp };

    if !rp.BitMap.is_null() {
        let bm = unsafe { &*rp.BitMap };

        // Masks with COMPLEMENT, keeping only the bits that matter here.
        let complement = rp.DrawMode & COMPLEMENT != 0;
        let (mut cx, mut cy) = (rp.cp_x as i32, rp.cp_y as i32);
        let (dx, dy) = ((x - cx).abs(), -(y - cy).abs());

        let (sx, sy) = (if cx < x { 1 } else { -1 }, if cy < y { 1 } else { -1 });
        let mut err = dx + dy;

        loop {
            unsafe { plot(bm, cx, cy, rp.FgPen, complement) };

            if cx == x && cy == y {
                break;
            }

            let e2 = 2 * err;

            if e2 >= dy {
                err += dy;
                cx += sx;
            }

            if e2 <= dx {
                err += dx;
                cy += sy;
            }
        }
    }

    rp.cp_x = x as WORD;
    rp.cp_y = y as WORD;
}
