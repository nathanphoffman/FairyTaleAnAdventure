//! Ports and device requests.
//! The game allocates these to talk to the trackdisk, input and audio devices. We keep the
//! allocations (the game frees them) but no device ever does anything.

use crate::amiga::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

const PORT_BYTES: usize = 64;
const STD_IO_BYTES: usize = 128;
const AUDIO_DEVICE: &str = "audio.device";

pub fn create_port(_name: *const c_char, _pri: LONG) -> *mut c_void {
    unsafe { libc::calloc(1, PORT_BYTES) }
}

pub fn delete_port(p: *mut c_void) {
    unsafe { libc::free(p) }
}

pub fn create_ext_io(_port: *mut c_void, size: LONG) -> *mut c_void {
    unsafe { libc::calloc(1, size.max(1) as usize) }
}

pub fn delete_ext_io(io: *mut c_void) {
    unsafe { libc::free(io) }
}

pub fn create_std_io(_port: *mut c_void) -> *mut c_void {
    unsafe { libc::calloc(1, STD_IO_BYTES) }
}

pub fn delete_std_io(io: *mut c_void) {
    unsafe { libc::free(io) }
}

/// Returns 0 (success) for every device except audio, which the platform layer replaces.
pub fn open_device(name: *const c_char, _unit: ULONG, _io: *mut c_void, _flags: ULONG) -> LONG {
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy();

    if name == AUDIO_DEVICE { -1 } else { 0 }
}

pub fn close_device(_io: *mut c_void) {}

pub fn do_io(_io: *mut c_void) -> LONG {
    0
}

pub fn send_io(_io: *mut c_void) {}

pub fn begin_io(_io: *mut c_void) {}

pub fn wait_io(_io: *mut c_void) -> LONG {
    0
}

/// Always "finished".
pub fn check_io(_io: *mut c_void) -> LONG {
    1
}
