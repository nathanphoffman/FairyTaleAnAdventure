//! Memory allocation.

use crate::amiga::*;
use std::os::raw::c_void;

/// A little slack at the end: the original's blitter code reads a word past some buffers.
const ALLOC_SLACK: usize = 128;

/// What `AvailMem` claims is free.
const FAKE_FREE_BYTES: ULONG = 1_000_000;

// ---- memory ----

pub fn alloc_mem(size: ULONG, _flags: ULONG) -> *mut c_void {
    // Always zeroed. Chip vs fast RAM does not exist here.
    unsafe { libc::calloc(1, size as usize + ALLOC_SLACK) }
}

pub fn free_mem(mem: *mut c_void, _size: ULONG) {
    unsafe { libc::free(mem) }
}

pub fn avail_mem(_flags: ULONG) -> ULONG {
    FAKE_FREE_BYTES
}

/// Everything counts as chip memory, so the game's `into_chip` never needs to copy.
pub fn type_of_mem(_p: *mut c_void) -> ULONG {
    MEMF_CHIP
}
