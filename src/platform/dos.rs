//! AmigaDOS file calls on top of normal files.
//!
//! The game opens files by bare name ("image", "songs", "fonts/Amber/9"). They are looked up in
//! the data directory (env `FTA_DATA`, default `faery-tale-amiga/game`), ignoring case. Names
//! like `df1:A.faery` (save disk) go to the `saves/` directory instead.

use crate::amiga::*;
use std::cell::RefCell;
use std::ffi::CStr;
use std::fs::{File, OpenOptions};
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

const MODE_NEWFILE: i32 = 1006;

thread_local! {
    static FILES: RefCell<Vec<Option<File>>> = const { RefCell::new(Vec::new()) };
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("FTA_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("faery-tale-amiga/game"))
}

fn save_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("FTA_SAVES").unwrap_or_else(|| "saves".into()))
}

/// Finds `name` under `base`, matching each path component case-insensitively.
fn find_ignore_case(base: &Path, name: &str) -> Option<PathBuf> {
    let mut cur = base.to_path_buf();
    for part in name.split('/').filter(|p| !p.is_empty()) {
        let exact = cur.join(part);
        if exact.exists() {
            cur = exact;
            continue;
        }
        let found = std::fs::read_dir(&cur).ok()?.flatten().find(|e| {
            e.file_name().to_string_lossy().eq_ignore_ascii_case(part)
        })?;
        cur = found.path();
    }
    Some(cur)
}

/// Maps an Amiga file name to a real path. `for_write` allows names that do not exist yet.
pub fn resolve(name: &str, for_write: bool) -> Option<PathBuf> {
    // device prefix such as "df1:" or "df0:"
    if let Some((dev, rest)) = name.split_once(':') {
        if dev.len() == 3 && dev[..2].eq_ignore_ascii_case("df") {
            if rest.is_empty() {
                return Some(save_dir());
            }
            let dir = save_dir();
            if for_write {
                let _ = std::fs::create_dir_all(&dir);
                return Some(dir.join(rest));
            }
            return find_ignore_case(&dir, rest);
        }
    }
    // game saves (A.faery ...) go to the saves directory, not the data directory
    if name.to_ascii_lowercase().ends_with(".faery") {
        let dir = save_dir();
        if for_write {
            let _ = std::fs::create_dir_all(&dir);
            return Some(dir.join(name));
        }
        return find_ignore_case(&dir, name);
    }
    find_ignore_case(&data_dir(), name).or_else(|| {
        if for_write {
            Some(data_dir().join(name))
        } else {
            None
        }
    })
}

unsafe fn cstr<'a>(p: *const c_char) -> &'a str {
    unsafe { CStr::from_ptr(p) }.to_str().unwrap_or("")
}

#[unsafe(no_mangle)]
pub extern "C" fn Open(name: *const c_char, mode: LONG) -> BPTR {
    let name = unsafe { cstr(name) };
    let write = mode == MODE_NEWFILE;
    let Some(path) = resolve(name, write) else { return 0 };
    let file = if write {
        OpenOptions::new().write(true).create(true).truncate(true).open(&path)
    } else {
        File::open(&path)
    };
    match file {
        Ok(f) => FILES.with(|files| {
            let mut files = files.borrow_mut();
            let slot = files.iter().position(|s| s.is_none()).unwrap_or_else(|| {
                files.push(None);
                files.len() - 1
            });
            files[slot] = Some(f);
            slot as BPTR + 1
        }),
        Err(_) => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Close(f: BPTR) {
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

#[unsafe(no_mangle)]
pub extern "C" fn Read(f: BPTR, buf: *mut u8, len: LONG) -> LONG {
    if len <= 0 {
        return 0;
    }
    let buf = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
    with_file(f, |file| {
        let mut total = 0;
        while total < buf.len() {
            match file.read(&mut buf[total..]) {
                Ok(0) => break,
                Ok(n) => total += n,
                Err(_) => return -1,
            }
        }
        total as LONG
    })
    .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn Write(f: BPTR, buf: *const u8, len: LONG) -> LONG {
    if len <= 0 {
        return 0;
    }
    let buf = unsafe { std::slice::from_raw_parts(buf, len as usize) };
    with_file(f, |file| file.write_all(buf).map(|_| len).unwrap_or(-1)).unwrap_or(-1)
}

/// mode: -1 from start, 0 from current position, 1 from end. Returns the old position.
#[unsafe(no_mangle)]
pub extern "C" fn Seek(f: BPTR, pos: LONG, mode: LONG) -> LONG {
    with_file(f, |file| {
        let old = file.stream_position().unwrap_or(0);
        let target = match mode {
            -1 => SeekFrom::Start(pos.max(0) as u64),
            0 => SeekFrom::Current(pos as i64),
            _ => SeekFrom::End(pos as i64),
        };
        match file.seek(target) {
            Ok(_) => old as LONG,
            Err(_) => -1,
        }
    })
    .unwrap_or(-1)
}

/// Returns a non-zero "lock" if the name exists. Locks are not tracked, so UnLock does nothing.
#[unsafe(no_mangle)]
pub extern "C" fn Lock(name: *const c_char, _mode: LONG) -> BPTR {
    let name = unsafe { cstr(name) };
    if resolve(name, false).is_some() { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn UnLock(_lock: BPTR) {}

#[unsafe(no_mangle)]
pub extern "C" fn IoErr() -> LONG {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn CurrentDir(_lock: BPTR) -> BPTR {
    0
}
