//! Memory, libraries, tasks and device I/O. Mostly stand-ins: there is no multitasking OS to
//! talk to, so these hand back dummy objects that the game can hold and release.

use crate::amiga::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr::{addr_of_mut, null_mut};

// ---- memory ----

#[unsafe(no_mangle)]
pub extern "C" fn AllocMem(size: ULONG, _flags: ULONG) -> *mut c_void {
    // Always zeroed. Chip vs fast RAM does not exist here.
    // a little slack at the end: the original's blitter code reads a word past some buffers
    unsafe { libc::calloc(1, size as usize + 128) }
}

#[unsafe(no_mangle)]
pub extern "C" fn FreeMem(mem: *mut c_void, _size: ULONG) {
    unsafe { libc::free(mem) }
}

#[unsafe(no_mangle)]
pub extern "C" fn AvailMem(_flags: ULONG) -> ULONG {
    1_000_000
}

/// Everything counts as chip memory, so the game's `into_chip` never needs to copy.
#[unsafe(no_mangle)]
pub extern "C" fn TypeOfMem(_p: *mut c_void) -> ULONG {
    MEMF_CHIP
}

// ---- libraries and tasks ----

static mut GFX_BASE: GfxBase = GfxBase {
    ActiView: null_mut(),
    DisplayFlags: 0,
};
static mut DUMMY_LIB: [u8; 64] = [0; 64];
static mut THIS_TASK: Process = Process {
    pr_Task: Task {
        tc_Node_pri: 0,
        tc_TrapCode: null_mut(),
    },
    pr_WindowPtr: null_mut(),
};

#[unsafe(no_mangle)]
pub extern "C" fn OpenLibrary(name: *const c_char, _version: ULONG) -> *mut c_void {
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    {
        if name == "graphics.library" {
            addr_of_mut!(GFX_BASE) as *mut c_void
        } else {
            addr_of_mut!(DUMMY_LIB) as *mut c_void
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn CloseLibrary(_lib: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn FindTask(_name: *const c_char) -> *mut Process {
    addr_of_mut!(THIS_TASK)
}

// ---- ports and device requests ----
// The game allocates these to talk to the trackdisk, input and audio devices. We keep the
// allocations (the game frees them) but no device ever does anything.

#[unsafe(no_mangle)]
pub extern "C" fn CreatePort(_name: *const c_char, _pri: LONG) -> *mut c_void {
    unsafe { libc::calloc(1, 64) }
}

#[unsafe(no_mangle)]
pub extern "C" fn DeletePort(p: *mut c_void) {
    unsafe { libc::free(p) }
}

#[unsafe(no_mangle)]
pub extern "C" fn CreateExtIO(_port: *mut c_void, size: LONG) -> *mut c_void {
    unsafe { libc::calloc(1, size.max(1) as usize) }
}

#[unsafe(no_mangle)]
pub extern "C" fn DeleteExtIO(io: *mut c_void) {
    unsafe { libc::free(io) }
}

#[unsafe(no_mangle)]
pub extern "C" fn CreateStdIO(_port: *mut c_void) -> *mut c_void {
    unsafe { libc::calloc(1, 128) }
}

#[unsafe(no_mangle)]
pub extern "C" fn DeleteStdIO(io: *mut c_void) {
    unsafe { libc::free(io) }
}

/// Returns 0 (success) for every device except audio, which the platform layer replaces.
#[unsafe(no_mangle)]
pub extern "C" fn OpenDevice(
    name: *const c_char,
    _unit: ULONG,
    _io: *mut c_void,
    _flags: ULONG,
) -> LONG {
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    if name == "audio.device" { -1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn CloseDevice(_io: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn DoIO(_io: *mut c_void) -> LONG {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn SendIO(_io: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn BeginIO(_io: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn WaitIO(_io: *mut c_void) -> LONG {
    0
}

/// Always "finished".
#[unsafe(no_mangle)]
pub extern "C" fn CheckIO(_io: *mut c_void) -> LONG {
    1
}
