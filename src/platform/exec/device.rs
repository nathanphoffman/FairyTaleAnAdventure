//! Ports and device requests.
//! The game allocates these to talk to the trackdisk, input and audio devices. We keep the
//! allocations (the game frees them) but no device ever does anything.

use crate::amiga::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

const PORT_BYTES: usize = 64;
const STD_IO_BYTES: usize = 128;
const AUDIO_DEVICE: &str = "audio.device";

#[unsafe(no_mangle)]
pub extern "C" fn CreatePort(_name: *const c_char, _pri: LONG) -> *mut c_void {
    unsafe { libc::calloc(1, PORT_BYTES) }
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
    unsafe { libc::calloc(1, STD_IO_BYTES) }
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
    if name == AUDIO_DEVICE { -1 } else { 0 }
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
