//! Drawing text with the loaded font.

use super::font::{BitmapFont, FontData};
use crate::amiga::*;
use crate::platform::gfx::bitmap::plot;

/// Width and height of the built-in font's cells, and how far its baseline sits below the top.
const BUILTIN_SIZE: i32 = 8;
const BUILTIN_BASELINE: i32 = 6;
const ASCII_MASK: u8 = 0x7f;
const LEFTMOST_BIT_FIRST: u8 = 0x80;

/// Draws one character of the built-in 8x8 font and returns how far the pen moves.
fn draw_builtin_glyph(rp: &RastPort, bm: &BitMap, ch: u8) -> WORD {
    let top = rp.cp_y as i32 - BUILTIN_BASELINE;

    // Masks with BASIC_LEGACY, keeping only the bits that matter here.
    let glyph = font8x8::legacy::BASIC_LEGACY[(ch & ASCII_MASK) as usize];
    let jam2 = rp.DrawMode & JAM2 != 0;

    for row in 0..BUILTIN_SIZE {
        for col in 0..BUILTIN_SIZE {
            let on = glyph[row as usize] >> col & 1 != 0; // bit 0 is leftmost
            let (x, y) = (rp.cp_x as i32 + col, top + row);

            if on {
                unsafe { plot(bm, x, y, rp.FgPen, false) };
            } else if jam2 {
                unsafe { plot(bm, x, y, rp.BgPen, false) };
            }
        }
    }

    BUILTIN_SIZE as WORD
}

/// Draws one character of a disk font and returns how far the pen moves.
fn draw_bitmap_glyph(rp: &RastPort, bm: &BitMap, f: &BitmapFont, ch: u8) -> WORD {
    let idx = if ch >= f.lo && ch <= f.hi {
        (ch - f.lo) as usize
    } else {
        (f.hi - f.lo) as usize + 1 // the "default" glyph sits after the last real one
    };

    let (bit_off, bit_w) = f.loc[idx];
    let advance = f.space[idx] as i32;
    let kern = f.kern[idx] as i32;
    let top = rp.cp_y as i32 - f.baseline;
    let jam2 = rp.DrawMode & JAM2 != 0;

    for row in 0..f.height {
        if jam2 {
            for x in 0..advance {
                unsafe { plot(bm, rp.cp_x as i32 + x, top + row, rp.BgPen, false) };
            }
        }

        for col in 0..bit_w as i32 {
            let bit = bit_off as usize + col as usize;
            let byte = f.data[row as usize * f.modulo + bit / 8];

            // The leftmost pixel of a byte is its top bit.
            if byte & (LEFTMOST_BIT_FIRST >> (bit % 8)) != 0 {
                unsafe { plot(bm, rp.cp_x as i32 + kern + col, top + row, rp.FgPen, false) };
            }
        }
    }

    advance as WORD
}

/// Draws `len` characters at the pen position (the pen sits on the baseline).
#[unsafe(no_mangle)]
pub extern "C" fn Text(rp: *mut RastPort, s: *const u8, len: LONG) -> LONG {
    let rp = unsafe { &mut *rp };

    if rp.BitMap.is_null() || rp.Font.is_null() || len <= 0 {
        return 0;
    }

    let font = unsafe { &*((*rp.Font).priv_ as *const FontData) };

    let bm = unsafe { &*rp.BitMap };

    let text = unsafe { std::slice::from_raw_parts(s, len as usize) };

    for &ch in text {
        rp.cp_x += match font {
            FontData::Builtin => draw_builtin_glyph(rp, bm, ch),
            FontData::Bitmap(f) => draw_bitmap_glyph(rp, bm, f, ch),
        };
    }

    0
}
