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

use super::globals::openflags;
use crate::amiga::BPTR;
use std::ffi::{c_char, c_void};

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

unsafe extern "C" {
    fn Lock(name: *const c_char, mode: i32) -> BPTR;
    fn UnLock(lock: BPTR);
    fn Open(name: *const c_char, mode: i32) -> BPTR;
    fn Close(file: BPTR);
    fn Read(file: BPTR, buf: *mut u8, len: i32) -> i32;
    fn Seek(file: BPTR, pos: i32, mode: i32) -> i32;
    fn CreatePort(name: *const c_char, pri: i32) -> *mut c_void;
    fn DeletePort(port: *mut c_void);
    fn CreateExtIO(port: *mut c_void, size: i32) -> *mut c_void;
    fn DeleteExtIO(io: *mut c_void);
    fn OpenDevice(name: *const c_char, unit: u32, io: *mut c_void, flags: u32) -> i32;
    fn CloseDevice(io: *mut c_void);
    fn DoIO(io: *mut c_void) -> i32;
    fn SendIO(io: *mut c_void);
    fn WaitIO(io: *mut c_void) -> i32;
    fn CheckIO(io: *mut c_void) -> i32;
}

// Bits in `openflags` recording what has been set up, so it can be undone on exit.
const AL_PORT: u16 = 0x0800;
const AL_IOREQ: u16 = 0x1000;
const AL_TDISK: u16 = 0x2000;

const ACCESS_READ: i32 = -2;
const MODE_OLDFILE: i32 = 1005;
const OFFSET_BEGINNING: i32 = -1;

const CMD_INVALID: u16 = 0;
const CMD_READ: u16 = 2;
const TD_MOTOR: u16 = 9;
const TD_NAME: &[u8] = b"trackdisk.device\0";
const IMAGE_FILE: &[u8] = b"image\0";

/// Bytes in a disk block.
const BLOCK_SIZE: i32 = 512;
/// Requests 0-8 are for loading (5 landscape, 2 terrain, 1 map/sector/monster); 9 is the motor.
const LOADING_REQUESTS: usize = 9;
const MOTOR_REQUEST: usize = 9;

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

/// True when blocks come from the `image` file instead of trackdisk.
fn from_file() -> bool {
    unsafe { FROM_FILE }
}

/// The I/O request slot `num`. Slots 0-8 are for loading, 9 is for the drive motor.
fn request(num: usize) -> *mut IoExtTd {
    unsafe { &raw mut DISKREQS[num] }
}

/// Records in `openflags` (a variable in csrc/fmain.c) that a resource was set up, so it can be freed on exit.
fn set_flag(flag: u16) {
    unsafe { openflags |= flag };
}

/// Whether a resource was set up (see `set_flag`).
fn has_flag(flag: u16) -> bool {
    unsafe { openflags & flag != 0 }
}

/// Opens the `image` file, or else sets up the trackdisk device. Returns 0 or an error code
/// (30 no file or port, 31 no request, 32 no device).
pub fn open_disk() -> i32 {
    unsafe {
        let lock = Lock(IMAGE_FILE.as_ptr().cast(), ACCESS_READ);
        if lock != 0 {
            FROM_FILE = true;
            UnLock(lock);
            IMAGE = Open(IMAGE_FILE.as_ptr().cast(), MODE_OLDFILE);
            return if IMAGE == 0 { ERR_NO_FILE } else { 0 };
        }
        DISKPORT = CreatePort(std::ptr::null(), 0);
        if DISKPORT.is_null() {
            return ERR_NO_FILE;
        }
        set_flag(AL_PORT);
        DISKREQ1 = CreateExtIO(DISKPORT, size_of::<IoExtTd>() as i32).cast();
        if DISKREQ1.is_null() {
            return ERR_NO_REQUEST;
        }
        set_flag(AL_IOREQ);
        if OpenDevice(TD_NAME.as_ptr().cast(), 0, DISKREQ1.cast(), 0) != 0 {
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
            Close(IMAGE);
        }
        if has_flag(AL_TDISK) {
            CloseDevice(DISKREQ1.cast());
        }
        if has_flag(AL_IOREQ) {
            DeleteExtIO(DISKREQ1.cast());
        }
        if has_flag(AL_PORT) {
            DeletePort(DISKPORT);
        }
    }
}

/// Waits for read request `num` to finish. Nothing to wait for when reading from the file.
pub fn wait_for_read(num: i32) {
    if !from_file() {
        unsafe { WaitIO(request(num as usize).cast()) };
    }
}

/// Marks read request `num` as unused.
pub fn mark_read_unused(num: i32) {
    if !from_file() {
        unsafe { (*request(num as usize)).command = CMD_INVALID };
    }
}

/// Whether request `num` has finished (always true for the file).
pub fn read_finished(num: i32) -> i32 {
    if from_file() {
        return 1;
    }
    unsafe { CheckIO(request(num as usize).cast()) }
}

/// Whether request `num` is a read in progress (never, for the file).
pub fn is_read_pending(num: i32) -> i32 {
    (!from_file() && unsafe { (*request(num as usize)).command } == CMD_READ) as i32
}

/// Waits for the most recently started read.
pub fn wait_for_last_read() {
    if !from_file() {
        unsafe { WaitIO(LASTREQ.cast()) };
    }
}

/// Marks the most recently started read as unused.
pub fn mark_last_read_unused() {
    if !from_file() {
        unsafe { (*LASTREQ).command = CMD_INVALID };
    }
}

/// Whether the most recently started read has finished (always true for the file).
pub fn last_read_finished() -> i32 {
    if from_file() {
        return 1;
    }
    unsafe { CheckIO(LASTREQ.cast()) }
}

/// Whether the most recent request is a read in progress (never, for the file).
pub fn is_last_read_pending() -> i32 {
    (!from_file() && unsafe { (*LASTREQ).command } == CMD_READ) as i32
}

/// Starts reading `b_count` blocks from block `f_block` into `buffer`, using request `dr` when
/// the trackdisk is in use. Reads from the `image` file immediately otherwise.
///
/// The file offset is `block * 512`. The game uses a different request number `dr` for each
/// kind of data (0 sector data, 1-2 terrain, 3-7 landscape images, 8 sprites) so several reads
/// could be in flight at once on a real disk.
pub fn load_track_range(f_block: i16, b_count: i16, buffer: *mut c_void, dr: i16) {
    unsafe {
        if from_file() {
            Seek(IMAGE, f_block as i32 * BLOCK_SIZE, OFFSET_BEGINNING);
            Read(IMAGE, buffer.cast(), b_count as i32 * BLOCK_SIZE);
            return;
        }
        LASTREQ = request(dr as usize);
        if (*LASTREQ).command == CMD_READ {
            WaitIO(LASTREQ.cast());
        }
        *LASTREQ = *DISKREQ1;
        (*LASTREQ).length = (b_count as i32 * BLOCK_SIZE) as u32;
        (*LASTREQ).data = buffer;
        (*LASTREQ).command = CMD_READ;
        (*LASTREQ).offset = (f_block as i32 * BLOCK_SIZE) as u32;
        SendIO(LASTREQ.cast());
    }
}

/// Turns the floppy drive motor off after loading (trackdisk only; nothing to do for the file).
pub fn motor_off() {
    if from_file() {
        return;
    }
    unsafe {
        let req = request(MOTOR_REQUEST);
        *req = *DISKREQ1;
        (*req).length = 0;
        (*req).command = TD_MOTOR;
        DoIO(req.cast());
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
