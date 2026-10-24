//! Locks and the current directory. Nothing is tracked: a lock only says "the name exists".

use super::files::cstr;
use super::paths::resolve;
use crate::amiga::*;
use std::os::raw::c_char;

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
