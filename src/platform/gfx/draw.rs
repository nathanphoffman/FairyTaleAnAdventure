//! Rastports and drawing: pens, fills, lines, blits and scrolling.

use super::bitmap::plot;
use crate::amiga::*;
use std::ptr::null_mut;

#[unsafe(no_mangle)]
pub extern "C" fn InitRastPort(rp: *mut RastPort) {
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

#[unsafe(no_mangle)]
pub extern "C" fn SetAPen(rp: *mut RastPort, pen: ULONG) {
    unsafe { (*rp).FgPen = pen as u8 }
}

#[unsafe(no_mangle)]
pub extern "C" fn SetBPen(rp: *mut RastPort, pen: ULONG) {
    unsafe { (*rp).BgPen = pen as u8 }
}

#[unsafe(no_mangle)]
pub extern "C" fn SetDrMd(rp: *mut RastPort, mode: ULONG) {
    unsafe { (*rp).DrawMode = mode as u8 }
}

/// Fills the whole bitmap with one pen.
#[unsafe(no_mangle)]
pub extern "C" fn SetRast(rp: *mut RastPort, pen: ULONG) {
    let rp = unsafe { &*rp };
    if rp.BitMap.is_null() {
        return;
    }
    let bm = unsafe { &*rp.BitMap };
    let size = bm.BytesPerRow as usize * bm.Rows as usize;
    for p in 0..bm.Depth as usize {
        if !bm.Planes[p].is_null() {
            let v = if pen >> p & 1 != 0 { 0xff } else { 0 };
            unsafe { std::ptr::write_bytes(bm.Planes[p], v, size) };
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn RectFill(rp: *mut RastPort, x0: LONG, y0: LONG, x1: LONG, y1: LONG) {
    let rp = unsafe { &*rp };
    if rp.BitMap.is_null() {
        return;
    }
    let bm = unsafe { &*rp.BitMap };
    let complement = rp.DrawMode & COMPLEMENT != 0;
    for y in y0.max(0)..=y1.min(bm.Rows as i32 - 1) {
        for x in x0.max(0)..=x1 {
            unsafe { plot(bm, x, y, rp.FgPen, complement) };
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Move(rp: *mut RastPort, x: LONG, y: LONG) {
    let rp = unsafe { &mut *rp };
    rp.cp_x = x as WORD;
    rp.cp_y = y as WORD;
}

/// Line from the current pen position to (x, y), then moves the pen there.
#[unsafe(no_mangle)]
pub extern "C" fn Draw(rp: *mut RastPort, x: LONG, y: LONG) {
    let rp = unsafe { &mut *rp };
    if !rp.BitMap.is_null() {
        let bm = unsafe { &*rp.BitMap };
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
