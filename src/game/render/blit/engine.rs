//! The blitter in software: combines up to three source channels into a destination, with shifts and
//! a minterm, the way the Amiga chip did.

use super::{ALL_BITS, Area, Loc, USE_A, USE_B, USE_C, USE_D};
use crate::gfx::{Bitmap, PAD};

/// A blitter job: places and strides for the channels, as the blitter registers held them.
pub(super) struct Blit {
    pub(super) a: Loc,
    pub(super) b: Loc,
    pub(super) c: Loc,
    pub(super) d: Loc,
    /// Bytes to skip at the end of each line.
    pub(super) amod: i32,
    pub(super) bmod: i32,
    pub(super) cmod: i32,
    pub(super) dmod: i32,
    /// Masks for the first and last word of a line of A.
    pub(super) afwm: u32,
    pub(super) alwm: u32,
    pub(super) ashift: u32,
    pub(super) bshift: u32,
    pub(super) uses: u8,
    pub(super) minterm: u32,
    /// In words and lines.
    pub(super) width: i32,
    pub(super) height: i32,
}

/// The memory a blit can reach: the bit planes being drawn on, the two work regions and the sprite
/// sheets. A read outside an area gives zeros, and a write outside it is dropped.
pub(super) struct BlitMem<'a> {
    pub(super) bitmap: &'a mut Bitmap,
    pub(super) queue: &'a mut [u8],
    pub(super) mask: &'a mut [u8],
    pub(super) shapes: &'a mut [u8],
}

impl BlitMem<'_> {
    /// The bytes of an area, and how far in the position 0 of the area is.
    fn area(&self, area: Area) -> (&[u8], isize) {
        match area {
            Area::Plane(p) if (p as usize) < self.bitmap.depth() => {
                (self.bitmap.padded_plane(p as usize), PAD as isize)
            }
            Area::Plane(_) => (&[], 0),
            Area::Queue => (self.queue, 0),
            Area::Mask => (self.mask, 0),
            Area::Shapes => (self.shapes, 0),
        }
    }

    fn area_mut(&mut self, area: Area) -> (&mut [u8], isize) {
        match area {
            Area::Plane(p) if (p as usize) < self.bitmap.depth() => {
                (self.bitmap.padded_plane_mut(p as usize), PAD as isize)
            }
            Area::Plane(_) => (&mut [], 0),
            Area::Queue => (self.queue, 0),
            Area::Mask => (self.mask, 0),
            Area::Shapes => (self.shapes, 0),
        }
    }

    pub(super) fn byte(&self, at: Loc) -> u8 {
        let (bytes, base) = self.area(at.area);

        usize::try_from(base + at.pos)
            .ok()
            .and_then(|i| bytes.get(i))
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn set_byte(&mut self, at: Loc, value: u8) {
        let (bytes, base) = self.area_mut(at.area);

        if let Some(byte) = usize::try_from(base + at.pos)
            .ok()
            .and_then(|i| bytes.get_mut(i))
        {
            *byte = value;
        }
    }

    /// Reads a big-endian 16-bit word.
    fn read_word(&self, at: Loc) -> u32 {
        // Words are big-endian in memory: the first byte is the high half.
        ((self.byte(at) as u32) << BYTE_BITS) | self.byte(at.add(1)) as u32
    }

    /// Writes a big-endian 16-bit word.
    fn write_word(&mut self, at: Loc, v: u32) {
        // Big-endian: the high byte is stored first.
        self.set_byte(at, (v >> BYTE_BITS) as u8);
        self.set_byte(at.add(1), v as u8);
    }
}

/// BLTSIZE: the width in words is the low 6 bits, the height the 10 bits above them.
const WIDTH_BITS: u32 = 6;
const WIDTH_MASK: i32 = 0x3f;
const HEIGHT_MASK: i32 = 0x3ff;

/// A width of 0 in the size word means 64 words.
const MAX_WIDTH: i32 = 64;

/// A word is 16 bits: two words are joined to shift across them. The minterm has 8 result bits,
/// picked by the A, B and C bits (4, 2, 1).
const WORD_BITS: u32 = 16;
const BYTE_BITS: u32 = 8;
const MINTERM_BITS: u32 = 8;
const MINTERM_A: u32 = 4;
const MINTERM_B: u32 = 2;
const MINTERM_C: u32 = 1;

/// BLTSIZE: height in the top 10 bits ...
pub(super) fn blit_height(size: i16) -> i32 {
    // The size word is height * 64 + width in words, so the height is everything above the low 6 bits.
    (size as i32 >> WIDTH_BITS) & HEIGHT_MASK
}

/// ... and the width in words in the low 6 bits (0 means 64).
pub(super) fn blit_width(size: i16) -> i32 {
    match size as i32 & WIDTH_MASK {
        0 => MAX_WIDTH,
        width => width,
    }
}

/// One 16 bit word of the minterm: for every bit, the result is picked by the A, B and C bits.
///
/// The minterm is a table of 8 result bits, one for each combination of the A, B and C bits
/// (bit number = A*4 + B*2 + C). For example 0xCA means "if A then B else C", which is how a mask
/// (A) selects between sprite pixels (B) and the screen behind (C).
pub(super) fn minterm_word(minterm: u32, a: u32, b: u32, c: u32) -> u32 {
    let pick = |word: u32, set: bool| if set { word } else { !word };
    let mut result = 0;

    for index in 0..MINTERM_BITS {
        // Bit `index` of the minterm is the result for that combination of the A, B and C bits.
        if minterm >> index & 1 != 0 {
            result |= pick(a, index & MINTERM_A != 0)
                & pick(b, index & MINTERM_B != 0)
                & pick(c, index & MINTERM_C != 0);
        }
    }

    result & ALL_BITS
}

/// Ascending blit (all the game's blits go left to right, top to bottom).
///
/// For each line it reads a word from each channel in use, shifts A and B (the shifted-out bits
/// of the previous word carry into the next, which lets a sprite sit at any pixel position, not
/// just on 16-pixel boundaries), combines them with the minterm and writes the result to D.
pub(super) fn blit(t: &mut Blit, mem: &mut BlitMem) {
    for _ in 0..t.height {
        blit_row(t, mem);
        skip_row_modulos(t);
    }
}

/// The A source word for column `x` of a row (all ones when A is not in use): the first and last
/// words of the row are masked.
fn next_a_word(t: &mut Blit, mem: &BlitMem, x: i32) -> u32 {
    if t.uses & USE_A == 0 {
        return ALL_BITS;
    }

    let mut aw = mem.read_word(t.a);
    t.a = t.a.add(2);

    if x == 0 {
        aw &= t.afwm;
    }

    if x == t.width - 1 {
        aw &= t.alwm;
    }

    aw
}

/// Blits one row of `t.width` words, working through the A, B and C sources and the D destination
/// that are in use. A and B can be shifted, and carry bits over from the previous word.
pub(super) fn blit_row(t: &mut Blit, mem: &mut BlitMem) {
    let (mut a_prev, mut b_prev) = (0u32, 0u32);

    for x in 0..t.width {
        let (mut bw, mut cw) = (0, 0);
        let aw = next_a_word(t, mem, x);

        if t.uses & USE_B != 0 {
            bw = mem.read_word(t.b);
            t.b = t.b.add(2);
        }

        if t.uses & USE_C != 0 {
            cw = mem.read_word(t.c);
        }

        // Shifting right across two words: the bits that fell off the previous word enter from the left.
        let shifted_a = ((a_prev << WORD_BITS | aw) >> t.ashift) & ALL_BITS;

        // Shifting right across two words: the bits that fell off the previous word enter from the left.
        let shifted_b = ((b_prev << WORD_BITS | bw) >> t.bshift) & ALL_BITS;
        a_prev = aw;
        b_prev = bw;

        let dw = minterm_word(t.minterm, shifted_a, shifted_b, cw);

        if t.uses & USE_D != 0 {
            mem.write_word(t.d, dw);
            t.d = t.d.add(2);
        }

        if t.uses & USE_C != 0 {
            t.c = t.c.add(2);
        }
    }
}

/// After a row, moves each place in use on by its modulo (bytes to skip to reach the next row).
pub(super) fn skip_row_modulos(t: &mut Blit) {
    if t.uses & USE_A != 0 {
        t.a = t.a.add(t.amod as isize);
    }

    if t.uses & USE_B != 0 {
        t.b = t.b.add(t.bmod as isize);
    }

    if t.uses & USE_C != 0 {
        t.c = t.c.add(t.cmod as isize);
    }

    if t.uses & USE_D != 0 {
        t.d = t.d.add(t.dmod as isize);
    }
}

impl Blit {
    pub(super) fn new(uses: u8, minterm: u32, blitsize: i16) -> Blit {
        Blit {
            a: Loc::NONE,
            b: Loc::NONE,
            c: Loc::NONE,
            d: Loc::NONE,
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

#[cfg(test)]
mod tests {
    use super::super::{D_EQUALS_A_AND_NOT_C, D_EQUALS_A_THEN_B_ELSE_C, D_EQUALS_B, USE_B, USE_D};
    use super::*;

    /// The bit-at-a-time definition of a minterm, as the C original computed it.
    fn minterm_by_bits(mt: u32, a: u32, b: u32, c: u32) -> u32 {
        (0..16).fold(0, |r, bit| {
            // The A, B and C bits of this position make the minterm's bit number (A * 4 + B * 2 + C).
            let index = ((a >> bit & 1) << 2) | ((b >> bit & 1) << 1) | (c >> bit & 1);

            // The minterm's bit for that combination becomes this bit of the result.
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

    /// Runs `job` on work memory holding `shapes` and two empty regions of 8 bytes.
    fn run(shapes: &[u8], job: impl FnOnce(&mut BlitMem)) -> [u8; 8] {
        let mut bitmap = Bitmap::empty();
        let mut queue = [0u8; 8];
        let mut mask = [0u8; 8];
        let mut shapes = shapes.to_vec();
        job(&mut BlitMem {
            bitmap: &mut bitmap,
            queue: &mut queue,
            mask: &mut mask,
            shapes: &mut shapes,
        });

        queue
    }

    fn at(area: Area) -> Loc {
        Loc { area, pos: 0 }
    }

    #[test]
    fn copy_blit_copies_words() {
        let src = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let dst = run(&src, |mem| {
            let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, 2 << 6 | 2); // size word: height 2 lines, width 2 words
            t.b = at(Area::Shapes);
            t.d = at(Area::Queue);
            blit(&mut t, mem);
        });

        assert_eq!(dst, src);
    }

    #[test]
    fn shifted_copy_moves_bits_right() {
        let dst = run(&[0xff, 0x00], |mem| {
            let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, 1 << 6 | 1); // size word: height 1 line, width 1 word
            t.b = at(Area::Shapes);
            t.d = at(Area::Queue);
            t.bshift = 4;
            blit(&mut t, mem);
        });

        assert_eq!(dst[..2], [0x0f, 0xf0]);
    }

    #[test]
    fn reads_outside_an_area_are_zero_and_writes_are_dropped() {
        let dst = run(&[9, 9], |mem| {
            let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, 1 << 6 | 1);
            t.b = Loc {
                area: Area::Shapes,
                pos: -1,
            };
            t.d = Loc {
                area: Area::Queue,
                pos: 7,
            };
            blit(&mut t, mem);
        });

        assert_eq!(dst[7], 0);
    }
}
