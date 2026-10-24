//! IFF picture file helpers (from fsubs.asm via csrc/asm_port.c). The state they work on lives
//! in `iffsubs`.

use super::iffsubs::{blocklength, bytecount, compress, file_length, header, myfile, packdata};
use crate::amiga::BPTR;

unsafe extern "C" {
    fn Read(file: BPTR, buf: *mut u8, len: i32) -> i32;
}

/// Bytes in a chunk id and in a chunk length.
const CHUNK_FIELD_BYTES: i32 = 4;

/// Reads a 4 byte chunk id; compared against the id string in memory, so no byte swap.
#[unsafe(no_mangle)]
pub extern "C" fn ReadHeader() {
    unsafe {
        Read(myfile as BPTR, (&raw mut header).cast(), CHUNK_FIELD_BYTES);
        file_length -= CHUNK_FIELD_BYTES;
    }
}

/// Reads a big-endian chunk length.
#[unsafe(no_mangle)]
pub extern "C" fn ReadLength() {
    unsafe {
        Read(myfile as BPTR, (&raw mut blocklength).cast(), CHUNK_FIELD_BYTES);
        blocklength = blocklength.swap_bytes();
        file_length -= blocklength + CHUNK_FIELD_BYTES;
    }
}

/// Unpacks one ByteRun1 scan line from `src` into `dest`, stopping after `width` bytes.
/// Returns how many source bytes were used.
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
#[unsafe(no_mangle)]
pub extern "C" fn unpack_line(dest: *mut u8) {
    unsafe {
        if compress == 0 {
            std::ptr::copy_nonoverlapping(packdata.cast(), dest, bytecount as usize);
            packdata = packdata.add(bytecount as usize);
        } else {
            let used = unpack_byte_run(packdata, dest, bytecount);
            packdata = packdata.add(used);
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
