//! Loads IFF ILBM pictures ("brushes") into a bitmap (from csrc/iffsubs.c). It also holds the
//! state shared with the chunk and line helpers in `iff_chunks`.
//!
//! An IFF file is a `FORM` containing chunks. Each chunk is a 4-letter id, a big-endian length and
//! the data. This loader understands `BMHD` (picture size and compression), skips `CMAP`, `GRAB`,
//! `CAMG` and `CRNG` (colours and display modes, which the game does not use) and unpacks `BODY`
//! (the pixels). The pixels are stored one bitplane row at a time, which is also how the Amiga's
//! screen memory is laid out, so each row is unpacked straight into the matching plane of the
//! destination bitmap.

use crate::amiga::{BPTR, BitMap};
use std::ffi::c_char;

use crate::game::data::state::the_memory;
use crate::game::files::iff_chunks::{read_chunk_id, read_chunk_length, unpack_line};

unsafe extern "C" {
    fn Open(name: *const c_char, mode: i32) -> BPTR;
    fn Close(file: BPTR);
    fn Read(file: BPTR, buf: *mut u8, len: i32) -> i32;
    fn Seek(file: BPTR, pos: i32, mode: i32) -> i32;
}

// State shared with iff_helpers; the names are the C ones (the C code and iff_helpers use them).
/// Bytes left in the FORM being read.
pub static mut bytes_left_in_file: i32 = 0;
pub static mut picture_file: i32 = 0;
/// The id of the chunk just read.
pub static mut current_chunk_id: i32 = 0;
pub static mut chunk_length: i32 = 0;
/// Bytes in one unpacked scan line.
pub static mut row_byte_count: i32 = 0;
/// Read position in the compressed body.
pub static mut pack_cursor: *const i8 = std::ptr::null();
/// Compression type of the body: 0 = none, 1 = ByteRun1.
pub static mut compression_type: i8 = 0;

/// AmigaDOS open mode for an existing file.
const MODE_OLDFILE: i32 = 1005;
/// Seek relative to the current position.
const OFFSET_CURRENT: i32 = 0;
const TRUE: i32 = 1;
const FALSE: i32 = 0;

/// Planes the brush loader fills.
const BRUSH_PLANES: usize = 5;

/// A chunk id as the game compares it: the four characters in memory order.
const fn chunk_id(name: &[u8; 4]) -> i32 {
    i32::from_ne_bytes(*name)
}
const FORM: i32 = chunk_id(b"FORM");
const ILBM: i32 = chunk_id(b"ILBM");
const BMHD: i32 = chunk_id(b"BMHD");
const BODY: i32 = chunk_id(b"BODY");
const SKIPPED_CHUNKS: [i32; 4] = [
    chunk_id(b"CAMG"),
    chunk_id(b"CRNG"),
    chunk_id(b"CMAP"),
    chunk_id(b"GRAB"),
];

/// The ILBM bitmap header chunk; the file is big-endian. Only `width`, `height` and
/// `compression` matter. (The C struct is 22 bytes although the IFF standard says 20; the
/// difference is harmless because the file's own length is used when reading it.)
#[repr(C)]
#[derive(Default)]
struct BitMapHeader {
    width: i16,
    height: i16,
    xpic: i16,
    ypic: i16,
    n_planes: u8,
    masking: u8,
    compression: u8,
    pad1: u8,
    transcolor: i16,
    x_aspect: i16,
    y_aspect: i16,
    page_width: i16,
    page_height: i16,
}

/// Reads the BMHD chunk whose length was just read, and converts the sizes from big-endian.
unsafe fn read_bitmap_header() -> BitMapHeader {
    let mut bmhd = BitMapHeader::default();
    unsafe {
        let len = chunk_length.clamp(0, size_of::<BitMapHeader>() as i32);
        Read(picture_file as BPTR, (&raw mut bmhd).cast(), len);
    }
    bmhd.width = bmhd.width.swap_bytes();
    bmhd.height = bmhd.height.swap_bytes();
    bmhd
}

/// Unpacks the BODY chunk into the bitmap, starting `offset` bytes into each plane.
///
/// The compressed body is first read into `shape_mem` (a scratch area), then each picture row is
/// unpacked once per bitplane (up to 5, fewer if the bitmap is shallower).
unsafe fn read_body(bmhd: &BitMapHeader, bitmap: &BitMap, offset: usize) {
    unsafe {
        pack_cursor = the_memory.shape_mem.cast();
        row_byte_count = ((bmhd.width as i32 + 15) / 8) & 0xfffe;
        read_chunk_length();
        Read(
            picture_file as BPTR,
            the_memory.shape_mem.cast(),
            chunk_length,
        );
        compression_type = bmhd.compression as i8;
        let mut planes: [*mut u8; BRUSH_PLANES] =
            std::array::from_fn(|p| bitmap.Planes[p].wrapping_add(offset));
        for _ in 0..bmhd.height {
            for (p, plane) in planes.iter_mut().enumerate() {
                if bitmap.Depth as usize > p {
                    unpack_line(*plane);
                }
                *plane = plane.wrapping_add(bitmap.BytesPerRow as usize);
            }
        }
    }
}

/// Loads the picture `filename` into `bitmap` at byte column `x`, row `y`. Returns TRUE/FALSE.
/// Used for the title pages, the story pictures, the text panel background and the winning picture.
pub fn load_picture(filename: *const c_char, bitmap: *mut BitMap, x: i16, y: i16) -> i32 {
    unsafe {
        let bitmap = &*bitmap;
        let offset = (x as i32 + bitmap.BytesPerRow as i32 * y as i32) as usize;

        picture_file = Open(filename, MODE_OLDFILE) as i32;
        if picture_file == 0 {
            return FALSE;
        }
        read_chunk_id();
        if current_chunk_id != FORM {
            Close(picture_file as BPTR);
            return FALSE;
        }
        read_chunk_length();
        bytes_left_in_file = chunk_length;

        let mut bmhd = BitMapHeader::default();
        while bytes_left_in_file != 0 {
            read_chunk_id();
            if current_chunk_id == ILBM {
                // just the form type, no data
            } else if current_chunk_id == BMHD {
                read_chunk_length();
                bmhd = read_bitmap_header();
            } else if SKIPPED_CHUNKS.contains(&{ current_chunk_id }) {
                read_chunk_length();
                Seek(picture_file as BPTR, chunk_length, OFFSET_CURRENT);
            } else if current_chunk_id == BODY {
                read_body(&bmhd, bitmap, offset);
                break;
            } else {
                Close(picture_file as BPTR);
                return FALSE;
            }
        }
        Close(picture_file as BPTR);
        TRUE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_ids_match_the_file_bytes() {
        assert_eq!(FORM.to_ne_bytes(), *b"FORM");
        assert_eq!(BODY.to_ne_bytes(), *b"BODY");
    }

    #[test]
    fn bitmap_header_matches_the_c_struct() {
        assert_eq!(size_of::<BitMapHeader>(), 22);
    }
}
