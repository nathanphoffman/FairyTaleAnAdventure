//! Fonts: the built-in 8x8 font and bitmap fonts read from an Amiga font file.

use super::bitmap::Bitmap;
use super::draw::{JAM2, Pen};

/// A bitmap font in the Amiga's layout: one wide strip of glyph bitmaps (`modulo` bytes per row)
/// with a table giving each glyph's bit offset and width, its advance and its kerning.
pub struct BitmapFont {
    height: i32,
    baseline: i32,
    lo: u8,
    hi: u8,
    modulo: usize,
    data: Vec<u8>,
    loc: Vec<(u16, u16)>,
    space: Vec<u16>,
    kern: Vec<i16>,
}

pub enum Font {
    /// Fixed 8x8 stand-in for the original's built-in font (Topaz 8).
    Builtin,
    Bitmap(BitmapFont),
}

/// Size of the built-in font's cells, and how far its baseline sits below the top.
const BUILTIN_SIZE: i32 = 8;
const BUILTIN_BASELINE: i32 = 6;
const ASCII_MASK: u8 = 0x7f;
const LEFTMOST_BIT_FIRST: u8 = 0x80;

/// Reads `N` bytes at offset `o`, or None when they run past the end of the data.
fn be_bytes<const N: usize>(d: &[u8], o: usize) -> Option<[u8; N]> {
    d.get(o..o + N)?.try_into().ok()
}

fn be16(d: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_be_bytes(be_bytes(d, o)?))
}

fn be32(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_be_bytes(be_bytes(d, o)?))
}

/// AmigaDOS hunk file markers.
const HUNK_HEADER: u32 = 0x3f3;
const HUNK_CODE: u32 = 0x3e9;

/// Bytes before the per-hunk size table in a HUNK_HEADER, and bytes of HUNK_CODE tag and size.
const HUNK_HEADER_FIXED: usize = 20;
const HUNK_CODE_PREFIX: usize = 8;
const LONG_BYTES: usize = 4;

/// Where the TextFont sits inside the DiskFontHeader, and where its fields sit inside that.
const TEXTFONT_OFFSET: usize = 58;
const TF_YSIZE: usize = 20;
const TF_BASELINE: usize = 26;
const TF_LOCHAR: usize = 32;
const TF_HICHAR: usize = 33;
const TF_CHARDATA: usize = 34;
const TF_MODULO: usize = 38;
const TF_CHARLOC: usize = 40;
const TF_CHARSPACE: usize = 44;
const TF_CHARKERN: usize = 48;

/// Cuts the glyph strip and the three per-glyph tables out of a font segment.
fn read_font_tables(seg: &[u8]) -> Option<BitmapFont> {
    let tf = TEXTFONT_OFFSET;
    let height = be16(seg, tf + TF_YSIZE)? as i32;
    let baseline = be16(seg, tf + TF_BASELINE)? as i32;
    let lo = *seg.get(tf + TF_LOCHAR)?;
    let hi = *seg.get(tf + TF_HICHAR)?;
    let chardata = be32(seg, tf + TF_CHARDATA)? as usize;
    let modulo = be16(seg, tf + TF_MODULO)? as usize;
    let charloc = be32(seg, tf + TF_CHARLOC)? as usize;
    let charspace = be32(seg, tf + TF_CHARSPACE)? as usize;
    let charkern = be32(seg, tf + TF_CHARKERN)? as usize;
    let count = hi as usize - lo as usize + 2; // plus the "default" glyph at the end
    let data = seg
        .get(chardata..chardata + modulo * height as usize)?
        .to_vec();
    let mut loc = Vec::new();
    let (mut space, mut kern) = (Vec::new(), Vec::new());

    for i in 0..count {
        loc.push((be16(seg, charloc + i * 4)?, be16(seg, charloc + i * 4 + 2)?));
        space.push(be16(seg, charspace + i * 2)?);
        kern.push(be16(seg, charkern + i * 2)? as i16);
    }

    Some(BitmapFont {
        height,
        baseline,
        lo,
        hi,
        modulo,
        data,
        loc,
        space,
        kern,
    })
}

impl Font {
    /// Reads an Amiga disk font (fonts/Amber/9): an executable file whose only hunk holds a
    /// DiskFontHeader. Offsets inside it are relative to the start of that hunk's data.
    pub fn load(path: &std::path::Path) -> Option<Font> {
        let file = std::fs::read(path).ok()?;

        Font::from_hunk_file(&file)
    }

    fn from_hunk_file(file: &[u8]) -> Option<Font> {
        if be32(file, 0)? != HUNK_HEADER {
            return None;
        }

        // HUNK_HEADER: 0, table size, first, last, one size per hunk, then HUNK_CODE, size, data
        let table = be32(file, 8)? as usize;
        let hunk = HUNK_HEADER_FIXED + LONG_BYTES * table;

        if be32(file, hunk)? != HUNK_CODE {
            return None;
        }

        read_font_tables(file.get(hunk + HUNK_CODE_PREFIX..)?).map(Font::Bitmap)
    }

    /// Draws character `ch` with its baseline at the pen and returns how far the pen moves.
    pub fn draw_glyph(&self, bm: &mut Bitmap, pen: &Pen, ch: u8, planes: usize) -> i32 {
        match self {
            Font::Builtin => draw_builtin_glyph(bm, pen, ch, planes),
            Font::Bitmap(font) => font.draw_glyph(bm, pen, ch, planes),
        }
    }
}

/// Draws one character of the built-in 8x8 font.
fn draw_builtin_glyph(bm: &mut Bitmap, pen: &Pen, ch: u8, planes: usize) -> i32 {
    let top = pen.y - BUILTIN_BASELINE;
    let glyph = font8x8::legacy::BASIC_LEGACY[(ch & ASCII_MASK) as usize];
    let jam2 = pen.mode & JAM2 != 0;

    for row in 0..BUILTIN_SIZE {
        for col in 0..BUILTIN_SIZE {
            let on = glyph[row as usize] >> col & 1 != 0; // bit 0 is leftmost
            let (x, y) = (pen.x + col, top + row);

            if on {
                bm.plot(x, y, pen.fg, false, planes);
            } else if jam2 {
                bm.plot(x, y, pen.bg, false, planes);
            }
        }
    }

    BUILTIN_SIZE
}

impl BitmapFont {
    fn draw_glyph(&self, bm: &mut Bitmap, pen: &Pen, ch: u8, planes: usize) -> i32 {
        let idx = if ch >= self.lo && ch <= self.hi {
            (ch - self.lo) as usize
        } else {
            (self.hi - self.lo) as usize + 1 // the "default" glyph sits after the last real one
        };

        let (bit_off, bit_w) = self.loc[idx];
        let advance = self.space[idx] as i32;
        let kern = self.kern[idx] as i32;
        let top = pen.y - self.baseline;
        let jam2 = pen.mode & JAM2 != 0;

        for row in 0..self.height {
            if jam2 {
                for x in 0..advance {
                    bm.plot(pen.x + x, top + row, pen.bg, false, planes);
                }
            }

            for col in 0..bit_w as i32 {
                let bit = bit_off as usize + col as usize;
                let byte = self.data[row as usize * self.modulo + bit / 8];

                // The leftmost pixel of a byte is its top bit.
                if byte & (LEFTMOST_BIT_FIRST >> (bit % 8)) != 0 {
                    bm.plot(pen.x + kern + col, top + row, pen.fg, false, planes);
                }
            }
        }

        advance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_big_endian_values() {
        let data = [0x12, 0x34, 0x56, 0x78];
        assert_eq!(be16(&data, 0), Some(0x1234));
        assert_eq!(be32(&data, 0), Some(0x1234_5678));
    }

    #[test]
    fn reading_past_the_end_gives_none() {
        let data = [1, 2, 3];
        assert_eq!(be16(&data, 2), None);
        assert_eq!(be32(&data, 0), None);
    }

    #[test]
    fn a_builtin_letter_is_drawn_and_advances_eight() {
        let mut bm = Bitmap::new(16, 16, 1);
        let mut pen = Pen::new();
        pen.x = 0;
        pen.y = 8;

        let advance = Font::Builtin.draw_glyph(&mut bm, &pen, b'I', 1);
        assert_eq!(advance, 8);
        assert!((0..16).any(|y| (0..8).any(|x| bm.pixel(x, y) == 1)));
    }
}
