//! Disk access for the game's data (from csrc/hdrive.c). When the file `image` exists the blocks
//! are read from it; otherwise the original trackdisk requests are used.
//!
//! Background: nearly all the game's graphics, maps and sounds are in one big file, `image`, a dump
//! of the original floppy disk (1760 blocks of 512 bytes). The game asks for "N blocks starting at
//! block B" with `load_track_range`. On the Amiga the original read the real disk through the
//! trackdisk device and the reads happened in the background; here every read finishes at once from
//! the file, so the "wait" and "check" functions below just report that everything is done.
//!
//! The trackdisk path is kept (it is the original behaviour) but is never used on Linux.

mod reads;

pub use self::reads::{
    is_last_read_pending, is_read_pending, last_read_finished, load_track_range,
    mark_last_read_unused, mark_read_unused, motor_off, read_finished, wait_for_last_read,
    wait_for_read,
};

use crate::amiga::BPTR;
use crate::game::data::globals::open_flags;
use std::ffi::c_void;

/// Mirror of `struct IOExtTD` in csrc/amiga.h: an Amiga trackdisk I/O request (a standard I/O
/// request plus two trackdisk fields). Only the trackdisk path uses it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IoExtTd {
    message_reply_port: *mut c_void,
    device: *mut c_void,
    unit: *mut c_void,
    command: u16,
    flags: u8,
    error: i8,
    actual: u32,
    length: u32,
    data: *mut c_void,
    offset: u32,
    count: u32,
    sec_label: u32,
}

const ZERO_REQUEST: IoExtTd = IoExtTd {
    message_reply_port: std::ptr::null_mut(),
    device: std::ptr::null_mut(),
    unit: std::ptr::null_mut(),
    command: 0,
    flags: 0,
    error: 0,
    actual: 0,
    length: 0,
    data: std::ptr::null_mut(),
    offset: 0,
    count: 0,
    sec_label: 0,
};

use crate::platform::dos::{close_file, lock_path, open_file, unlock_path};
use crate::platform::exec::{
    close_device, create_ext_io, create_port, delete_ext_io, delete_port, open_device,
};

// Bits in `openflags` recording what has been set up, so it can be undone on exit.
const AL_PORT: u16 = 0x0800;
const AL_IOREQ: u16 = 0x1000;
const AL_TDISK: u16 = 0x2000;
const ACCESS_READ: i32 = -2;
const MODE_OLDFILE: i32 = 1005;
const TD_NAME: &[u8] = b"trackdisk.device\0";
const IMAGE_FILE: &[u8] = b"image\0";

/// Requests 0-8 are for loading (5 landscape, 2 terrain, 1 map/sector/monster); 9 is the motor.
const LOADING_REQUESTS: usize = 9;

// Error codes returned by `open_disk`.
const ERR_NO_FILE: i32 = 30;
const ERR_NO_REQUEST: i32 = 31;
const ERR_NO_DEVICE: i32 = 32;

static mut FROM_FILE: bool = false;
static mut IMAGE: BPTR = 0;

pub static mut DISKPORT: *mut c_void = std::ptr::null_mut();
pub static mut DISKREQ1: *mut IoExtTd = std::ptr::null_mut();
pub static mut DISKREQS: [IoExtTd; 10] = [ZERO_REQUEST; 10];
pub static mut LASTREQ: *mut IoExtTd = std::ptr::null_mut();

/// Records in `openflags` (a variable in csrc/fmain.c) that a resource was set up, so it can be freed on exit.
fn set_flag(flag: u16) {
    unsafe { open_flags |= flag };
}

/// Whether a resource was set up (see `set_flag`).
fn has_flag(flag: u16) -> bool {
    unsafe { open_flags & flag != 0 }
}

/// Opens the `image` file, or else sets up the trackdisk device. Returns 0 or an error code
/// (30 no file or port, 31 no request, 32 no device).
pub fn open_disk() -> i32 {
    unsafe {
        let lock = lock_path(IMAGE_FILE.as_ptr().cast(), ACCESS_READ);

        if lock != 0 {
            FROM_FILE = true;
            unlock_path(lock);
            IMAGE = open_file(IMAGE_FILE.as_ptr().cast(), MODE_OLDFILE);

            return if IMAGE == 0 { ERR_NO_FILE } else { 0 };
        }

        DISKPORT = create_port(std::ptr::null(), 0);

        if DISKPORT.is_null() {
            return ERR_NO_FILE;
        }

        set_flag(AL_PORT);
        DISKREQ1 = create_ext_io(DISKPORT, size_of::<IoExtTd>() as i32).cast();

        if DISKREQ1.is_null() {
            return ERR_NO_REQUEST;
        }

        set_flag(AL_IOREQ);

        if open_device(TD_NAME.as_ptr().cast(), 0, DISKREQ1.cast(), 0) != 0 {
            return ERR_NO_DEVICE;
        }

        set_flag(AL_TDISK);

        for request in (*(&raw mut DISKREQS)).iter_mut().take(LOADING_REQUESTS) {
            *request = *DISKREQ1;
        }

        0
    }
}

/// Closes the image file or the trackdisk device, whichever was opened.
pub fn close_disk() {
    unsafe {
        if IMAGE != 0 {
            close_file(IMAGE);
        }

        if has_flag(AL_TDISK) {
            close_device(DISKREQ1.cast());
        }

        if has_flag(AL_IOREQ) {
            delete_ext_io(DISKREQ1.cast());
        }

        if has_flag(AL_PORT) {
            delete_port(DISKPORT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_is_the_size_of_the_c_struct() {
        assert_eq!(size_of::<IoExtTd>(), 64);
    }
}
