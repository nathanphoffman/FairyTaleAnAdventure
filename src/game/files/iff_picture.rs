//! Loads IFF ILBM pictures ("brushes") into a bitmap (from csrc/iffsubs.c).
//!
//! An IFF file is a `FORM` containing chunks. Each chunk is a 4-letter id, a big-endian length and
//! the data. This loader understands `BMHD` (picture size and compression), skips `CMAP`, `GRAB`,
//! `CAMG` and `CRNG` (colours and display modes, which the game does not use) and unpacks `BODY`
//! (the pixels). The pixels are stored one bitplane row at a time, which is also how the Amiga's
//! screen memory is laid out, so each row is unpacked straight into the matching plane of the
//! destination bitmap.

use crate::gfx::Bitmap;
use crate::platform::paths::resolve;

/// Planes the brush loader fills.
const BRUSH_PLANES: usize = 5;

const FORM: &[u8; 4] = b"FORM";
const ILBM: &[u8; 4] = b"ILBM";
const BMHD: &[u8; 4] = b"BMHD";
const BODY: &[u8; 4] = b"BODY";
const SKIPPED_CHUNKS: [&[u8; 4]; 4] = [b"CAMG", b"CRNG", b"CMAP", b"GRAB"];

/// Bytes in a chunk id and in a chunk length.
const CHUNK_FIELD_BYTES: usize = 4;

/// A picture row is a whole number of 16 pixel words, as bytes: round up, divide by 8, make even.
const WORD_ROUNDING: usize = 15;
const PIXELS_PER_BYTE: usize = 8;

/// What the loader needs from the ILBM bitmap header chunk (the file is big-endian).
#[derive(Default)]
struct BitMapHeader {
    width: usize,
    height: usize,
    compressed: bool,
}

impl BitMapHeader {
    /// Reads the sizes at the start of the chunk's data: the width and height, then (after the
    /// position, plane count and masking fields) the compression type.
    fn read(data: &[u8]) -> BitMapHeader {
        let be16 = |at: usize| {
            data.get(at..at + 2)
                .map_or(0, |b| u16::from_be_bytes([b[0], b[1]]))
        };

        BitMapHeader {
            width: be16(0) as usize,
            height: be16(2) as usize,
            compressed: data.get(10).copied().unwrap_or(0) != 0,
        }
    }

    /// Bytes in one unpacked scan line.
    fn row_bytes(&self) -> usize {
        ((self.width + WORD_ROUNDING) / PIXELS_PER_BYTE) & !1
    }
}

/// The compressed or plain pixel data of a BODY chunk, read row by row.
struct Body<'a> {
    data: &'a [u8],
    at: usize,
    compressed: bool,
}

impl Body<'_> {
    /// Unpacks the next scan line (`dest.len()` bytes).
    fn unpack_line(&mut self, dest: &mut [u8]) {
        if self.compressed {
            self.unpack_byte_run(dest);
        } else {
            let end = (self.at + dest.len()).min(self.data.len());
            let line = &self.data[self.at.min(end)..end];
            dest[..line.len()].copy_from_slice(line);
            self.at = end;
        }
    }

    /// Unpacks one ByteRun1 scan line, stopping after `dest.len()` bytes.
    ///
    /// ByteRun1: a control byte `n` from 0 to 127 means the next n+1 bytes are copied as they are;
    /// from -127 to -1 means the next byte is repeated 1-n times.
    fn unpack_byte_run(&mut self, dest: &mut [u8]) {
        let mut written = 0;

        while written < dest.len() {
            let Some(&control) = self.data.get(self.at) else {
                return;
            };

            self.at += 1;
            let control = control as i8;

            if control >= 0 {
                for _ in 0..=control {
                    let byte = self.data.get(self.at).copied().unwrap_or(0);
                    self.at += 1;
                    put(dest, written, byte);
                    written += 1;
                }
            } else {
                let byte = self.data.get(self.at).copied().unwrap_or(0);
                self.at += 1;

                for _ in 0..=(-(control as i32)) {
                    put(dest, written, byte);
                    written += 1;
                }
            }
        }
    }
}

/// Stores a byte unless the run has gone past the end of the line.
fn put(dest: &mut [u8], at: usize, byte: u8) {
    if let Some(slot) = dest.get_mut(at) {
        *slot = byte;
    }
}

/// Reads chunk ids and lengths from the file.
struct Chunks<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Chunks<'a> {
    fn id(&mut self) -> Option<[u8; 4]> {
        let id = self
            .data
            .get(self.at..self.at + CHUNK_FIELD_BYTES)?
            .try_into()
            .ok()?;
        self.at += CHUNK_FIELD_BYTES;

        Some(id)
    }

    fn length(&mut self) -> Option<usize> {
        let bytes = self.data.get(self.at..self.at + CHUNK_FIELD_BYTES)?;
        self.at += CHUNK_FIELD_BYTES;

        Some(u32::from_be_bytes(bytes.try_into().ok()?) as usize)
    }

    /// The next `length` bytes, which are skipped over.
    fn take(&mut self, length: usize) -> &'a [u8] {
        let end = (self.at + length).min(self.data.len());
        let bytes = &self.data[self.at.min(end)..end];
        self.at = end;

        bytes
    }
}

/// Loads the picture `name` into `bitmap` at byte column `x`, row `y`. Returns false if the file is
/// missing or not a picture the game can read. Used for the title pages, the story pictures, the
/// text panel background and the winning picture.
pub fn load_picture(name: &str, bitmap: &mut Bitmap, x: usize, y: usize) -> bool {
    let Some(file) = resolve(name, false).and_then(|path| std::fs::read(path).ok()) else {
        return false;
    };

    read_form(&file, bitmap, x + bitmap.bytes_per_row() * y).is_some()
}

/// Reads the chunks of an IFF file: the header, then the picture body into `bitmap` at byte
/// `offset`. Returns None if this is not a picture file the game can read.
fn read_form(file: &[u8], bitmap: &mut Bitmap, offset: usize) -> Option<()> {
    let mut chunks = Chunks { data: file, at: 0 };

    if &chunks.id()? != FORM {
        return None;
    }

    let mut left = chunks.length()? as i64;
    let mut header = BitMapHeader::default();

    while left != 0 {
        let id = chunks.id()?;
        left -= CHUNK_FIELD_BYTES as i64;

        if &id == ILBM {
            // just the form type, no data
        } else if &id == BMHD {
            let length = chunks.length()?;
            left -= (length + CHUNK_FIELD_BYTES) as i64;
            header = BitMapHeader::read(chunks.take(length));
        } else if SKIPPED_CHUNKS.contains(&&id) {
            let length = chunks.length()?;
            left -= (length + CHUNK_FIELD_BYTES) as i64;
            chunks.take(length);
        } else if &id == BODY {
            let length = chunks.length()?;
            let mut body = Body {
                data: chunks.take(length),
                at: 0,
                compressed: header.compressed,
            };

            unpack_body(&header, &mut body, bitmap, offset);

            break;
        } else {
            return None;
        }
    }

    Some(())
}

/// Unpacks the BODY into the bitmap, starting `offset` bytes into each plane: each picture row is
/// unpacked once per bitplane (up to 5, fewer if the bitmap is shallower).
fn unpack_body(header: &BitMapHeader, body: &mut Body, bitmap: &mut Bitmap, offset: usize) {
    let row_bytes = header.row_bytes();
    let bytes_per_row = bitmap.bytes_per_row();
    let mut line = vec![0u8; row_bytes];

    for row in 0..header.height {
        let start = offset + row * bytes_per_row;

        for p in 0..BRUSH_PLANES {
            if bitmap.depth() > p {
                body.unpack_line(&mut line);

                if let Some(dest) = bitmap.plane_mut(p).get_mut(start..start + row_bytes) {
                    dest.copy_from_slice(&line);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(data: &[u8], compressed: bool) -> Body<'_> {
        Body {
            data,
            at: 0,
            compressed,
        }
    }

    #[test]
    fn literal_runs_are_copied() {
        let mut out = [0u8; 3];
        let mut b = body(&[2, 10, 20, 30], true); // 3 literal bytes
        b.unpack_line(&mut out);

        assert_eq!(out, [10, 20, 30]);
        assert_eq!(b.at, 4);
    }

    #[test]
    fn repeat_runs_are_expanded() {
        let mut out = [0u8; 4];
        let mut b = body(&[(-3i8) as u8, 7], true); // 7 repeated 4 times
        b.unpack_line(&mut out);

        assert_eq!(out, [7, 7, 7, 7]);
        assert_eq!(b.at, 2);
    }

    #[test]
    fn an_uncompressed_row_is_copied() {
        let mut out = [0u8; 2];
        let mut b = body(&[5, 6, 7, 8], false);
        b.unpack_line(&mut out);
        assert_eq!(out, [5, 6]);
        b.unpack_line(&mut out);
        assert_eq!(out, [7, 8]);
    }

    #[test]
    fn rows_are_whole_words() {
        let header = BitMapHeader {
            width: 320,
            ..Default::default()
        };
        assert_eq!(header.row_bytes(), 40);
    }

    /// A tiny 16 x 2 picture with 2 planes, uncompressed.
    fn tiny_picture() -> Vec<u8> {
        let mut bmhd = vec![0, 16, 0, 2, 0, 0, 0, 0, 2, 0, 0, 0];
        bmhd.resize(20, 0);
        let pixels = [0xff, 0x00, 0x00, 0xff, 0x0f, 0xf0, 0xf0, 0x0f];
        let mut form = b"ILBM".to_vec();
        form.extend(b"BMHD");
        form.extend((bmhd.len() as u32).to_be_bytes());
        form.extend(&bmhd);
        form.extend(b"BODY");
        form.extend((pixels.len() as u32).to_be_bytes());
        form.extend(pixels);
        let mut file = b"FORM".to_vec();
        file.extend((form.len() as u32).to_be_bytes());
        file.extend(form);

        file
    }

    #[test]
    fn a_picture_file_is_unpacked_into_the_planes() {
        let mut bm = Bitmap::new(16, 2, 2);

        assert!(read_form(&tiny_picture(), &mut bm, 0).is_some());
        assert_eq!(bm.plane(0)[..2], [0xff, 0x00]);
        assert_eq!(bm.plane(1)[..2], [0x00, 0xff]);
        assert_eq!(bm.plane(0)[2..4], [0x0f, 0xf0]);
    }

    #[test]
    fn something_else_is_refused() {
        let mut bm = Bitmap::new(16, 2, 2);

        assert!(read_form(b"JUNKJUNKJUNK", &mut bm, 0).is_none());
    }
}
