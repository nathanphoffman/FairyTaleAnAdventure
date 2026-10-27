//! Sprite drawing routines that drove the Amiga blitter in fsubs.asm (from csrc/asm_blit.c).
//!
//! The blitter combines up to three source channels (A, B, C) into a destination D, 16 bits at
//! a time, with an optional barrel shift of A and B, first/last word masks on A, and a
//! "minterm" saying how the bits combine. `blit` does the same in software for the modes the
//! game uses, so each routine here sets up the same values as its assembly original and the
//! drawing comes out the same. Words are big-endian in memory (leftmost pixels in the first byte).
//!
//! How a sprite gets onto the screen (see the main loop in csrc/fmain.c):
//! 1. `save_background` copies the background under the sprite's rectangle to a save area.
//! 2. `add_scenery_mask` copies, for every scenery tile that should hide the sprite (trees, walls ...), the
//!    tile's mask into a screen-sized scenery mask.
//! 3. `combine_masks` combines the sprite's own mask with that: the sprite is visible only where it is
//!    opaque and nothing is in front of it.
//! 4. `draw_shape` draws the sprite's pixels through that mask into the 5 bitplanes of the screen.
//! 5. Next frame `restore_background` puts the saved background back, which erases the sprite.
//!
//! `make_sprite_masks` builds each frame's own mask once, when the sprite sheet is loaded.

use crate::game::data::globals::{
    current_sshape, last_word_mask, shadow_mask_offset, sprite_mask_modulo, sprite_mask_offset,
    sprite_x_shift,
};
use crate::game::data::state::{the_memory, the_screen};
/// Mirror of `struct sshape` in csrc/ftale.h: how one sprite is drawn this frame and where its
/// background is saved. `coff` is the byte offset of the sprite in a bitplane and `cmod` the bytes
/// to skip to the next line; `blitsize` is the blitter size word (height * 64 + width in words).
#[repr(C)]
pub struct SShape {
    pub backsave: *mut u8,
    pub savesize: i16,
    pub blitsize: i16,
    pub coff: i16,
    pub cmod: i16,
}

// Which channels a blit uses.
const USE_A: u8 = 1;
const USE_B: u8 = 2;
const USE_C: u8 = 4;
const USE_D: u8 = 8;

// The minterms the game uses (result for each combination of the A, B and C bits).
const D_EQUALS_B: u32 = 0xcc;
const D_EQUALS_A_AND_NOT_C: u32 = 0x50;
const D_EQUALS_A_THEN_B_ELSE_C: u32 = 0xca;

/// Bitplanes in the play field.
const PLANE_COUNT: i32 = 5;
const SHIFT_MASK: i16 = 15;
const ALL_BITS: u32 = 0xffff;
/// Bytes in the mask for one scenery tile (32 lines of one word).
const TILE_MASK_BYTES: usize = 64;
const TILE_LINES: usize = 32;
/// The scenery mask starts this many bytes into `bmask_mem`.
const MASK_START: usize = 2;

/// A blitter job: pointers and strides for the channels, as the blitter registers held them.
struct Blit {
    a: *const u8,
    b: *const u8,
    c: *mut u8,
    d: *mut u8,
    /// Bytes to skip at the end of each line.
    amod: i32,
    bmod: i32,
    cmod: i32,
    dmod: i32,
    /// Masks for the first and last word of a line of A.
    afwm: u32,
    alwm: u32,
    ashift: u32,
    bshift: u32,
    uses: u8,
    minterm: u32,
    /// In words and lines.
    width: i32,
    height: i32,
}

impl Blit {
    fn new(uses: u8, minterm: u32, blitsize: i16) -> Blit {
        Blit {
            a: std::ptr::null(),
            b: std::ptr::null(),
            c: std::ptr::null_mut(),
            d: std::ptr::null_mut(),
            amod: 0,
            bmod: 0,
            cmod: 0,
            dmod: 0,
            afwm: ALL_BITS,
            alwm: ALL_BITS,
            ashift: 0,
            bshift: 0,
            uses,
            minterm,
            width: blit_width(blitsize),
            height: blit_height(blitsize),
        }
    }
}

/// BLTSIZE: height in the top 10 bits ...
fn blit_height(size: i16) -> i32 {
    (size as i32 >> 6) & 0x3ff
}

/// ... and the width in words in the low 6 bits (0 means 64).
fn blit_width(size: i16) -> i32 {
    match size as i32 & 0x3f {
        0 => 64,
        width => width,
    }
}

/// Reads a big-endian 16-bit word.
unsafe fn read_word(p: *const u8) -> u32 {
    unsafe { ((*p as u32) << 8) | *p.add(1) as u32 }
}

/// Writes a big-endian 16-bit word.
unsafe fn write_word(p: *mut u8, v: u32) {
    unsafe {
        *p = (v >> 8) as u8;
        *p.add(1) = v as u8;
    }
}

/// One 16 bit word of the minterm: for every bit, the result is picked by the A, B and C bits.
///
/// The minterm is a table of 8 result bits, one for each combination of the A, B and C bits
/// (bit number = A*4 + B*2 + C). For example 0xCA means "if A then B else C", which is how a mask
/// (A) selects between sprite pixels (B) and the screen behind (C).
fn minterm_word(minterm: u32, a: u32, b: u32, c: u32) -> u32 {
    let pick = |word: u32, set: bool| if set { word } else { !word };
    let mut result = 0;
    for index in 0..8 {
        if minterm >> index & 1 != 0 {
            result |= pick(a, index & 4 != 0) & pick(b, index & 2 != 0) & pick(c, index & 1 != 0);
        }
    }
    result & ALL_BITS
}

/// Ascending blit (all the game's blits go left to right, top to bottom).
///
/// For each line it reads a word from each channel in use, shifts A and B (the shifted-out bits
/// of the previous word carry into the next, which lets a sprite sit at any pixel position, not
/// just on 16-pixel boundaries), combines them with the minterm and writes the result to D.
unsafe fn blit(t: &mut Blit) {
    unsafe {
        for _ in 0..t.height {
            blit_row(t);
            skip_row_modulos(t);
        }
    }
}

/// Blits one row of `t.width` words, working through the A, B and C sources and the D destination
/// that are in use. A and B can be shifted, and carry bits over from the previous word.
unsafe fn blit_row(t: &mut Blit) {
    unsafe {
        let (mut a_prev, mut b_prev) = (0u32, 0u32);
        for x in 0..t.width {
            let (mut aw, mut bw, mut cw) = (ALL_BITS, 0, 0);
            if t.uses & USE_A != 0 {
                aw = read_word(t.a);
                t.a = t.a.add(2);
                if x == 0 {
                    aw &= t.afwm;
                }
                if x == t.width - 1 {
                    aw &= t.alwm;
                }
            }
            if t.uses & USE_B != 0 {
                bw = read_word(t.b);
                t.b = t.b.add(2);
            }
            if t.uses & USE_C != 0 {
                cw = read_word(t.c);
            }
            let shifted_a = ((a_prev << 16 | aw) >> t.ashift) & ALL_BITS;
            let shifted_b = ((b_prev << 16 | bw) >> t.bshift) & ALL_BITS;
            a_prev = aw;
            b_prev = bw;

            let dw = minterm_word(t.minterm, shifted_a, shifted_b, cw);
            if t.uses & USE_D != 0 {
                write_word(t.d, dw);
                t.d = t.d.add(2);
            }
            if t.uses & USE_C != 0 {
                t.c = t.c.add(2);
            }
        }
    }
}

/// After a row, moves each pointer in use on by its modulo (bytes to skip to reach the next row).
unsafe fn skip_row_modulos(t: &mut Blit) {
    unsafe {
        if t.uses & USE_A != 0 {
            t.a = t.a.offset(t.amod as isize);
        }
        if t.uses & USE_B != 0 {
            t.b = t.b.offset(t.bmod as isize);
        }
        if t.uses & USE_C != 0 {
            t.c = t.c.offset(t.cmod as isize);
        }
        if t.uses & USE_D != 0 {
            t.d = t.d.offset(t.dmod as isize);
        }
    }
}

/// Where plane `p` of the play field starts for the current sprite.
unsafe fn plane_at_sprite(p: i32) -> *mut u8 {
    unsafe { (*the_screen.planes.offset(p as isize)).offset((*current_sshape).coff as isize) }
}

/// Zeroes a rectangle of memory, sized like a BLTSIZE value.
pub fn clear_memory(mem: *mut u8, size: i32) {
    let bytes = blit_height(size as i16) * blit_width(size as i16) * 2;
    unsafe { std::ptr::write_bytes(mem, 0, bytes as usize) };
}

/// Copies the background under the sprite (all planes) to `savemem`.
pub fn save_background(savemem: *mut u8) {
    unsafe {
        for p in (0..PLANE_COUNT).rev() {
            let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, (*current_sshape).blitsize);
            t.b = plane_at_sprite(p);
            t.bmod = (*current_sshape).cmod as i32;
            t.d = savemem.offset((*current_sshape).savesize as isize * p as isize);
            blit(&mut t);
        }
    }
}

/// Puts the saved background back (this erases the sprite).
pub fn restore_background(savemem: *mut u8) {
    unsafe {
        for p in (0..PLANE_COUNT).rev() {
            let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, (*current_sshape).blitsize);
            t.b = savemem.offset((*current_sshape).savesize as isize * p as isize);
            t.d = plane_at_sprite(p);
            t.dmod = (*current_sshape).cmod as i32;
            blit(&mut t);
        }
    }
}

/// Combines the sprite's own mask with the scenery mask built by `add_scenery_mask`. The result goes in
/// `bmask_mem`: the sprite is opaque here and nothing is in front of it (D = A and not C).
pub fn combine_masks() {
    unsafe {
        let mut t = Blit::new(
            USE_A | USE_C | USE_D,
            D_EQUALS_A_AND_NOT_C,
            (*current_sshape).blitsize,
        );
        t.a = the_memory.mask_buffer.offset(sprite_mask_offset as isize);
        t.amod = sprite_mask_modulo as i32;
        t.ashift = (sprite_x_shift & SHIFT_MASK) as u32;
        t.c = the_memory
            .bmask_mem
            .add(MASK_START)
            .offset(shadow_mask_offset as isize);
        t.d = t.c;
        blit(&mut t);
    }
}

/// Draws the sprite into the planes through the mask: D = mask ? shape : screen.
/// The first and last word masks stop the sprite's shifted edges leaking into neighbouring pixels.
pub fn draw_shape() {
    unsafe {
        let bit_shift = (sprite_x_shift & SHIFT_MASK) as u32;
        let first = ALL_BITS >> bit_shift;
        let mut last = !first & ALL_BITS;
        if last == 0 {
            last = ALL_BITS;
        }
        for p in (0..PLANE_COUNT).rev() {
            let mut t = Blit::new(
                USE_A | USE_B | USE_C | USE_D,
                D_EQUALS_A_THEN_B_ELSE_C,
                (*current_sshape).blitsize,
            );
            t.bshift = bit_shift;
            t.afwm = first & last_word_mask as u16 as u32;
            t.alwm = last;
            t.a = the_memory
                .bmask_mem
                .add(MASK_START)
                .offset(shadow_mask_offset as isize);
            t.b = the_memory
                .shapedata
                .offset(sprite_mask_offset as isize + the_screen.planesize as isize * p as isize);
            t.bmod = sprite_mask_modulo as i32;
            t.c = plane_at_sprite(p);
            t.d = t.c;
            t.cmod = (*current_sshape).cmod as i32;
            t.dmod = t.cmod;
            blit(&mut t);
        }
    }
}

/// Copies the 32 line mask of scenery tile `tile` (from `shadow_mem`) into the scenery mask at
/// word column `x`, tile row `y`; `mod_words` is the mask width in words.
pub fn add_scenery_mask(x: i32, y: i32, mod_words: i32, tile: i32) {
    let row_bytes = mod_words as isize * 2;
    unsafe {
        let mut dst = the_memory
            .bmask_mem
            .add(MASK_START)
            .offset(y as isize * row_bytes * TILE_LINES as isize + x as isize * 2);
        let mut src = the_memory
            .shadow_mem
            .offset(tile as isize * TILE_MASK_BYTES as isize);
        for _ in 0..TILE_LINES {
            *dst = *src;
            *dst.add(1) = *src.add(1);
            src = src.add(2);
            dst = dst.offset(row_bytes);
        }
    }
}

/// For every sprite frame builds a one plane mask: all 5 planes ANDed then inverted, so colour
/// 31 (all bits set) is transparent. The masks are stored right after the frames.
pub fn make_sprite_masks(src: *const u8, dest: *mut u8, words: i32, scans: i32, chars: i32) {
    let plane_bytes = (words * scans) as u16 as usize * 2;
    unsafe {
        let (mut src, mut dest) = (src, dest);
        for _ in 0..chars {
            let plane = |n: usize| src.add(n * plane_bytes);
            for i in 0..plane_bytes {
                let all = (0..PLANE_COUNT as usize).fold(0xff, |acc, n| acc & *plane(n).add(i));
                *dest = !all;
                dest = dest.add(1);
            }
            src = plane(PLANE_COUNT as usize);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bit-at-a-time definition of a minterm, as the C original computed it.
    fn minterm_by_bits(mt: u32, a: u32, b: u32, c: u32) -> u32 {
        (0..16).fold(0, |r, bit| {
            let index = ((a >> bit & 1) << 2) | ((b >> bit & 1) << 1) | (c >> bit & 1);
            r | ((mt >> index & 1) << bit)
        })
    }

    #[test]
    fn minterm_words_match_the_bit_by_bit_version() {
        let samples = [0x0000, 0xffff, 0x1234, 0xa5c3, 0x0ff0, 0x8001];
        for mt in [D_EQUALS_B, D_EQUALS_A_AND_NOT_C, D_EQUALS_A_THEN_B_ELSE_C] {
            for &a in &samples {
                for &b in &samples {
                    for &c in &samples {
                        assert_eq!(minterm_word(mt, a, b, c), minterm_by_bits(mt, a, b, c));
                    }
                }
            }
        }
    }

    #[test]
    fn blit_size_fields() {
        assert_eq!(blit_height(0x0802), 32);
        assert_eq!(blit_width(0x0802), 2);
        assert_eq!(blit_width(0x0800), 64);
    }

    #[test]
    fn copy_blit_copies_words() {
        let src = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let mut dst = [0u8; 8];
        let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, 2 << 6 | 2);
        t.b = src.as_ptr();
        t.d = dst.as_mut_ptr();
        unsafe { blit(&mut t) };
        assert_eq!(dst, src);
    }

    #[test]
    fn shifted_copy_moves_bits_right() {
        let src = [0xffu8, 0x00];
        let mut dst = [0u8; 2];
        let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, 1 << 6 | 1);
        t.b = src.as_ptr();
        t.d = dst.as_mut_ptr();
        t.bshift = 4;
        unsafe { blit(&mut t) };
        assert_eq!(dst, [0x0f, 0xf0]);
    }
}
