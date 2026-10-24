//! Disk fonts: reading the Amiga font format and handing out `TextFont`s.

use crate::amiga::*;
use std::ffi::CStr;
use std::os::raw::c_char;

/// A bitmap font in the Amiga's layout: one wide strip of glyph bitmaps (`modulo` bytes per row)
/// with a table giving each glyph's bit offset and width, its advance and its kerning.
pub(super) struct BitmapFont {
    pub(super) height: i32,
    pub(super) baseline: i32,
    pub(super) lo: u8,
    pub(super) hi: u8,
    pub(super) modulo: usize,
    pub(super) data: Vec<u8>,
    pub(super) loc: Vec<(u16, u16)>,
    pub(super) space: Vec<u16>,
    pub(super) kern: Vec<i16>,
}

pub(super) enum FontData {
    /// Fixed 8x8 stand-in for the Amiga's ROM font Topaz 8.
    Builtin,
    Bitmap(BitmapFont),
}

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

/// Size of the built-in stand-in fonts: (width, height, baseline).
const TOPAZ: (i32, i32, i32) = (8, 8, 6);
const STORY_FALLBACK: (i32, i32, i32) = (8, 9, 7);
const STORY_FONT_WIDTH: i32 = 10;

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

/// Reads an Amiga disk font (fonts/Amber/9): an executable file whose only hunk holds a
/// DiskFontHeader. Offsets inside it are relative to the start of that hunk's data.
fn load_disk_font(path: &std::path::Path) -> Option<BitmapFont> {
    let file = std::fs::read(path).ok()?;
    if be32(&file, 0)? != HUNK_HEADER {
        return None;
    }
    // HUNK_HEADER: 0, table size, first, last, one size per hunk, then HUNK_CODE, size, data
    let table = be32(&file, 8)? as usize;
    let hunk = HUNK_HEADER_FIXED + LONG_BYTES * table;
    if be32(&file, hunk)? != HUNK_CODE {
        return None;
    }
    read_font_tables(file.get(hunk + HUNK_CODE_PREFIX..)?)
}

fn new_font(data: FontData, width: i32, height: i32, baseline: i32) -> *mut TextFont {
    let data = Box::into_raw(Box::new(data));
    Box::into_raw(Box::new(TextFont {
        tf_YSize: height as UWORD,
        tf_XSize: width as UWORD,
        tf_Baseline: baseline as UWORD,
        tf_Flags: 0,
        priv_: data as APTR,
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn OpenFont(_attr: *mut TextAttr) -> *mut TextFont {
    new_font(FontData::Builtin, TOPAZ.0, TOPAZ.1, TOPAZ.2)
}

#[unsafe(no_mangle)]
pub extern "C" fn OpenDiskFont(_attr: *mut TextAttr) -> *mut TextFont {
    new_font(FontData::Builtin, TOPAZ.0, TOPAZ.1, TOPAZ.2)
}

#[unsafe(no_mangle)]
pub extern "C" fn CloseFont(_f: *mut TextFont) {}

#[unsafe(no_mangle)]
pub extern "C" fn SetFont(rp: *mut RastPort, f: *mut TextFont) {
    unsafe { (*rp).Font = f }
}

/// The game adds this many bytes to `seg << 2` to reach the DiskFontHeader.
const SEG_HEADER_BYTES: isize = 8;

/// The game finds its story font with `font = (seg << 2) + 8` and `afont = &font->dfh_TF`, an
/// Amiga trick for turning a loaded segment into a pointer. We hand back a segment value that
/// makes that arithmetic land on a DiskFontHeader we allocated.
#[unsafe(no_mangle)]
pub extern "C" fn LoadSeg(name: *const c_char) -> BPTR {
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    let Some(path) = crate::platform::dos::resolve(&name, false) else {
        return 0;
    };
    let tf = match load_disk_font(&path) {
        Some(font) => {
            let (h, b) = (font.height, font.baseline);
            new_font(FontData::Bitmap(font), STORY_FONT_WIDTH, h, b)
        }
        None => new_font(
            FontData::Builtin,
            STORY_FALLBACK.0,
            STORY_FALLBACK.1,
            STORY_FALLBACK.2,
        ),
    };
    let header = Box::into_raw(Box::new(DiskFontHeader {
        dfh_TF: unsafe { std::ptr::read(tf) },
    }));
    ((header as isize) - SEG_HEADER_BYTES) >> 2
}

#[unsafe(no_mangle)]
pub extern "C" fn UnLoadSeg(_seg: BPTR) {}

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
}
