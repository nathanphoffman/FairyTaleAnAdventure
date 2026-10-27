//! Opening, reading, writing and seeking files.

use super::paths::resolve;
use crate::amiga::*;
use std::cell::RefCell;
use std::ffi::CStr;
use std::fs::{File, OpenOptions};
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::os::raw::c_char;

const MODE_NEWFILE: i32 = 1006;

/// Seek modes.
const OFFSET_BEGINNING: LONG = -1;
const OFFSET_CURRENT: LONG = 0;
const ERROR: LONG = -1;

thread_local! {
    static FILES: RefCell<Vec<Option<File>>> = const { RefCell::new(Vec::new()) };
}

pub(super) unsafe fn cstr<'a>(p: *const c_char) -> &'a str {
    unsafe { CStr::from_ptr(p) }.to_str().unwrap_or("")
}

/// Stores an open file in the first free slot. The handle is the slot number plus one, so 0 can
/// mean failure.
fn store_file(f: File) -> BPTR {
    FILES.with(|files| {
        let mut files = files.borrow_mut();
        let slot = files.iter().position(|s| s.is_none()).unwrap_or_else(|| {
            files.push(None);

            files.len() - 1
        });

        files[slot] = Some(f);

        slot as BPTR + 1
    })
}

/// The Amiga `Open`: opens an existing file, or creates a new one in write mode. Returns 0 on failure.
pub fn open_file(name: *const c_char, mode: LONG) -> BPTR {
    let name = unsafe { cstr(name) };
    let write = mode == MODE_NEWFILE;
    let Some(path) = resolve(name, write) else {
        return 0;
    };

    let file = if write {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
    } else {
        File::open(&path)
    };

    match file {
        Ok(f) => store_file(f),
        Err(e) => {
            eprintln!("Could not open {}: {e}", path.display());

            0
        }
    }
}

pub fn close_file(f: BPTR) {
    FILES.with(|files| {
        if let Some(slot) = files.borrow_mut().get_mut((f - 1) as usize) {
            *slot = None;
        }
    });
}

fn with_file<R>(f: BPTR, op: impl FnOnce(&mut File) -> R) -> Option<R> {
    FILES.with(|files| {
        let mut files = files.borrow_mut();

        files.get_mut((f - 1) as usize)?.as_mut().map(op)
    })
}

/// Reads until `buf` is full or the file ends. Returns the byte count, or ERROR.
fn read_fully(file: &mut File, buf: &mut [u8]) -> LONG {
    let mut total = 0;

    while total < buf.len() {
        match file.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) => {
                eprintln!("Read failed: {e}");

                return ERROR;
            }
        }
    }

    total as LONG
}

pub fn read_file(f: BPTR, buf: *mut u8, len: LONG) -> LONG {
    if len <= 0 {
        return 0;
    }

    let buf = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };

    with_file(f, |file| read_fully(file, buf)).unwrap_or(ERROR)
}

/// The Amiga `Write`: writes `len` bytes to an open file. Returns the number written, or -1.
pub fn write_file(f: BPTR, buf: *const u8, len: LONG) -> LONG {
    if len <= 0 {
        return 0;
    }

    let buf = unsafe { std::slice::from_raw_parts(buf, len as usize) };

    with_file(f, |file| match file.write_all(buf) {
        Ok(()) => len,
        Err(e) => {
            eprintln!("Write failed: {e}");

            ERROR
        }
    })
    .unwrap_or(ERROR)
}

/// mode: -1 from start, 0 from current position, 1 from end. Returns the old position.
pub fn seek_file(f: BPTR, pos: LONG, mode: LONG) -> LONG {
    with_file(f, |file| {
        let old = file.stream_position().unwrap_or(0);
        let target = match mode {
            OFFSET_BEGINNING => SeekFrom::Start(pos.max(0) as u64),
            OFFSET_CURRENT => SeekFrom::Current(pos as i64),
            _ => SeekFrom::End(pos as i64),
        };

        match file.seek(target) {
            Ok(_) => old as LONG,
            Err(e) => {
                eprintln!("Seek failed: {e}");

                ERROR
            }
        }
    })
    .unwrap_or(ERROR)
}
