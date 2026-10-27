//! IFF picture file helpers (from fsubs.asm via csrc/asm_port.c). The state they work on lives
//! in `iff_picture`.
//!
//! `read_chunk_id` and `read_chunk_length` read the two fields at the start of every IFF chunk;
//! `unpack_line` unpacks one row of picture data, which is compressed with "ByteRun1" (the usual
//! run-length scheme on the Amiga).

use crate::amiga::BPTR;
use crate::game::files::iff_picture::{
    bytes_left_in_file, chunk_length, compression_type, current_chunk_id, pack_cursor,
    picture_file, row_byte_count,
};

unsafe extern "C" {
    fn Read(file: BPTR, buf: *mut u8, len: i32) -> i32;
}

/// Bytes in a chunk id and in a chunk length.
const CHUNK_FIELD_BYTES: i32 = 4;

/// Reads a 4 byte chunk id; compared against the id string in memory, so no byte swap.
pub fn read_chunk_id() {
    unsafe {
        Read(
            picture_file as BPTR,
            (&raw mut current_chunk_id).cast(),
            CHUNK_FIELD_BYTES,
        );
        bytes_left_in_file -= CHUNK_FIELD_BYTES;
    }
}

/// Reads a big-endian chunk length and updates how many bytes of the file are left.
pub fn read_chunk_length() {
    unsafe {
        Read(
            picture_file as BPTR,
            (&raw mut chunk_length).cast(),
            CHUNK_FIELD_BYTES,
        );
        chunk_length = chunk_length.swap_bytes();
        bytes_left_in_file -= chunk_length + CHUNK_FIELD_BYTES;
    }
}

/// Unpacks one ByteRun1 scan line from `src` into `dest`, stopping after `width` bytes.
/// Returns how many source bytes were used.
///
/// ByteRun1: a control byte `n` from 0 to 127 means the next n+1 bytes are copied as they are;
/// from -127 to -1 means the next byte is repeated 1-n times.
unsafe fn unpack_byte_run(mut src: *const i8, mut dest: *mut u8, width: i32) -> usize {
    let start = src;
    let mut written = 0;
    while written < width {
        let mut count = unsafe { *src } as i32;
        src = unsafe { src.add(1) };
        if count >= 0 {
            // a literal run of count + 1 bytes
            while count >= 0 {
                unsafe { *dest = *src as u8 };
                dest = unsafe { dest.add(1) };
                src = unsafe { src.add(1) };
                written += 1;
                count -= 1;
            }
        } else {
            // one byte repeated -count + 1 times
            count = -count;
            let byte = unsafe { *src } as u8;
            src = unsafe { src.add(1) };
            while count >= 0 {
                unsafe { *dest = byte };
                dest = unsafe { dest.add(1) };
                written += 1;
                count -= 1;
            }
        }
    }
    unsafe { src.offset_from(start) as usize }
}

/// Unpacks one scan line of an IFF body (ByteRun1, or plain copy) into `dest`.
pub fn unpack_line(dest: *mut u8) {
    unsafe {
        if compression_type == 0 {
            std::ptr::copy_nonoverlapping(pack_cursor.cast(), dest, row_byte_count as usize);
            pack_cursor = pack_cursor.add(row_byte_count as usize);
        } else {
            let used = unpack_byte_run(pack_cursor, dest, row_byte_count);
            pack_cursor = pack_cursor.add(used);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_runs_are_copied() {
        let src: [i8; 4] = [2, 10, 20, 30]; // 3 literal bytes
        let mut out = [0u8; 3];
        let used = unsafe { unpack_byte_run(src.as_ptr(), out.as_mut_ptr(), 3) };
        assert_eq!(out, [10, 20, 30]);
        assert_eq!(used, 4);
    }

    #[test]
    fn repeat_runs_are_expanded() {
        let src: [i8; 2] = [-3, 7]; // 7 repeated 4 times
        let mut out = [0u8; 4];
        let used = unsafe { unpack_byte_run(src.as_ptr(), out.as_mut_ptr(), 4) };
        assert_eq!(out, [7, 7, 7, 7]);
        assert_eq!(used, 2);
    }
}
