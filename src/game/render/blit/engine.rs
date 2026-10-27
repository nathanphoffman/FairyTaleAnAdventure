//! The blitter in software: combines up to three source channels into a destination, with shifts and
//! a minterm, the way the Amiga chip did.

use super::{ALL_BITS, USE_A, USE_B, USE_C, USE_D};

/// A blitter job: pointers and strides for the channels, as the blitter registers held them.
pub(super) struct Blit {
    pub(super) a: *const u8,
    pub(super) b: *const u8,
    pub(super) c: *mut u8,
    pub(super) d: *mut u8,
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

/// BLTSIZE: height in the top 10 bits ...
pub(super) fn blit_height(size: i16) -> i32 {
    // The size word is height * 64 + width in words, so the height is everything above the low 6 bits.
    (size as i32 >> 6) & 0x3ff
}

/// ... and the width in words in the low 6 bits (0 means 64).
pub(super) fn blit_width(size: i16) -> i32 {
    match size as i32 & 0x3f {
        0 => 64,
        width => width,
    }
}

/// Reads a big-endian 16-bit word.
pub(super) unsafe fn read_word(p: *const u8) -> u32 {
    // Words are big-endian in memory: the first byte is the high half.
    unsafe { ((*p as u32) << 8) | *p.add(1) as u32 }
}

/// Writes a big-endian 16-bit word.
pub(super) unsafe fn write_word(p: *mut u8, v: u32) {
    unsafe {
        // Big-endian: the high byte is stored first.
        *p = (v >> 8) as u8;
        *p.add(1) = v as u8;
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

    for index in 0..8 {
        // Bit `index` of the minterm is the result for that combination of the A, B and C bits.
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
pub(super) unsafe fn blit(t: &mut Blit) {
    unsafe {
        for _ in 0..t.height {
            blit_row(t);
            skip_row_modulos(t);
        }
    }
}

/// Blits one row of `t.width` words, working through the A, B and C sources and the D destination
/// that are in use. A and B can be shifted, and carry bits over from the previous word.
pub(super) unsafe fn blit_row(t: &mut Blit) {
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

            // Shifting right across two words: the bits that fell off the previous word enter from the left.
            let shifted_a = ((a_prev << 16 | aw) >> t.ashift) & ALL_BITS;
            // Shifting right across two words: the bits that fell off the previous word enter from the left.
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
pub(super) unsafe fn skip_row_modulos(t: &mut Blit) {
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

impl Blit {
    pub(super) fn new(uses: u8, minterm: u32, blitsize: i16) -> Blit {
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

#[cfg(test)]
mod tests {
    use super::super::{D_EQUALS_A_AND_NOT_C, D_EQUALS_A_THEN_B_ELSE_C, D_EQUALS_B, USE_B, USE_D};
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
