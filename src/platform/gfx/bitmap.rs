//! Planar bitmaps: allocation and single-pixel access.

use crate::amiga::*;
use std::os::raw::c_void;
use std::ptr::null_mut;

/// Rows are a whole number of 16 pixel words (2 bytes each); 8 pixels fit in a byte, and the leftmost
/// pixel of a byte is its top bit.
const WORD_PIXELS: i32 = 16;
const WORD_BYTES: i32 = 2;
const BYTE_PIXELS_SHIFT: u32 = 3;
const BYTE_PIXELS_MASK: i32 = 7;
const LEFTMOST_PIXEL: u8 = 0x80;

fn bytes_per_row(width: i32) -> usize {
    (((width + WORD_PIXELS - 1) / WORD_PIXELS) * WORD_BYTES) as usize
}

#[unsafe(no_mangle)]
pub extern "C" fn InitBitMap(bm: *mut BitMap, depth: LONG, width: LONG, height: LONG) {
    let bm = unsafe { &mut *bm };
    bm.BytesPerRow = bytes_per_row(width) as UWORD;
    bm.Rows = height as UWORD;
    bm.Flags = 0;
    bm.Depth = depth as UBYTE;
    bm.pad = 0;
    bm.Planes = [null_mut(); MAX_PLANES];
}

/// Allocates the planes too. Returns 1 on success (this one was assembly in the original).
#[unsafe(no_mangle)]
pub extern "C" fn MakeBitMap(bm: *mut BitMap, depth: LONG, width: LONG, height: LONG) -> LONG {
    InitBitMap(bm, depth, width, height);

    let b = unsafe { &mut *bm };

    for p in 0..depth as usize {
        b.Planes[p] = AllocRaster(width as ULONG, height as ULONG);

        if b.Planes[p].is_null() {
            UnMakeBitMap(bm);

            return 0;
        }
    }

    1
}

#[unsafe(no_mangle)]
pub extern "C" fn UnMakeBitMap(bm: *mut BitMap) {
    let b = unsafe { &mut *bm };

    for p in 0..b.Depth as usize {
        FreeRaster(b.Planes[p], 0, 0);
        b.Planes[p] = null_mut();
    }
}

/// Bitplanes get padding on both sides: the blitter routines (csrc/asm_blit.c) read and write a
/// word or two outside the visible area, as the Amiga's memory allowed.
const RASTER_PAD: usize = 256;

#[unsafe(no_mangle)]
pub extern "C" fn AllocRaster(width: ULONG, height: ULONG) -> PLANEPTR {
    let size = bytes_per_row(width as i32) * height as usize + 2 * RASTER_PAD;

    let base = unsafe { libc::calloc(1, size) as PLANEPTR };

    if base.is_null() {
        base
    } else {
        unsafe { base.add(RASTER_PAD) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn FreeRaster(p: PLANEPTR, _w: ULONG, _h: ULONG) {
    if !p.is_null() {
        unsafe { libc::free(p.sub(RASTER_PAD) as *mut c_void) }
    }
}

/// Returns bit `x` of row `y` in a plane.
pub(in crate::platform::gfx) unsafe fn get_bit(bm: &BitMap, plane: usize, x: i32, y: i32) -> bool {
    unsafe {
        let base = bm.Planes[plane];
        // Byte holding the pixel: 8 pixels to a byte.
        let off = y as usize * bm.BytesPerRow as usize + (x >> BYTE_PIXELS_SHIFT) as usize;

        // The leftmost pixel of a byte is its top bit.
        *base.add(off) & (LEFTMOST_PIXEL >> (x & BYTE_PIXELS_MASK)) != 0
    }
}

pub(in crate::platform::gfx) unsafe fn set_bit(
    bm: &BitMap,
    plane: usize,
    x: i32,
    y: i32,
    on: bool,
) {
    unsafe {
        let base = bm.Planes[plane];
        // Byte holding the pixel: 8 pixels to a byte.
        let off = y as usize * bm.BytesPerRow as usize + (x >> BYTE_PIXELS_SHIFT) as usize;
        // The leftmost pixel of a byte is its top bit.
        let mask = LEFTMOST_PIXEL >> (x & BYTE_PIXELS_MASK);

        if on {
            *base.add(off) |= mask
        } else {
            *base.add(off) &= !mask
        }
    }
}

/// Whether the pixel (`x`, `y`) is inside the bitmap.
pub(in crate::platform::gfx) fn in_bounds(bm: &BitMap, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && y < bm.Rows as i32 && x < bm.BytesPerRow as i32 * (1 << BYTE_PIXELS_SHIFT)
}

/// Draws one pixel with `pen` (or flips it, in COMPLEMENT mode).
pub(in crate::platform::gfx) unsafe fn plot(
    bm: &BitMap,
    x: i32,
    y: i32,
    pen: u8,
    complement: bool,
) {
    if !in_bounds(bm, x, y) {
        return;
    }

    for p in 0..bm.Depth as usize {
        if bm.Planes[p].is_null() {
            continue;
        }

        unsafe {
            if complement {
                let cur = get_bit(bm, p, x, y);
                set_bit(bm, p, x, y, !cur);
            } else {
                // Bitplane `p` gets bit `p` of the pen number.
                set_bit(bm, p, x, y, pen >> p & 1 != 0);
            }
        }
    }
}
