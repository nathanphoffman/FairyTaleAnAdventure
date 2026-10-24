//! Disk access for the game's data (from csrc/hdrive.c). When the file `image` exists the blocks
//! are read from it; otherwise the original trackdisk requests are used.

use crate::amiga::BPTR;
use std::ffi::{c_char, c_void};

/// Mirror of `struct IOExtTD` in csrc/amiga.h (an `IOStdReq` plus two trackdisk fields).
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
    static mut openflags: u16;
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

// Error codes returned by `AllocDiskIO`.
const ERR_NO_FILE: i32 = 30;
const ERR_NO_REQUEST: i32 = 31;
const ERR_NO_DEVICE: i32 = 32;

static mut FROM_FILE: bool = false;
static mut IMAGE: BPTR = 0;

#[unsafe(no_mangle)]
pub static mut diskport: *mut c_void = std::ptr::null_mut();
#[unsafe(no_mangle)]
pub static mut diskreq1: *mut IoExtTd = std::ptr::null_mut();
#[unsafe(no_mangle)]
pub static mut diskreqs: [IoExtTd; 10] = [ZERO_REQUEST; 10];
#[unsafe(no_mangle)]
pub static mut lastreq: *mut IoExtTd = std::ptr::null_mut();

/// True when blocks come from the `image` file instead of trackdisk.
fn from_file() -> bool {
    unsafe { FROM_FILE }
}

fn request(num: usize) -> *mut IoExtTd {
    unsafe { &raw mut diskreqs[num] }
}

fn set_flag(flag: u16) {
    unsafe { openflags |= flag };
}

fn has_flag(flag: u16) -> bool {
    unsafe { openflags & flag != 0 }
}

/// Opens the `image` file, or else sets up the trackdisk device. Returns 0 or an error code.
#[unsafe(no_mangle)]
pub extern "C" fn AllocDiskIO() -> i32 {
    unsafe {
        let lock = Lock(IMAGE_FILE.as_ptr().cast(), ACCESS_READ);
        if lock != 0 {
            FROM_FILE = true;
            UnLock(lock);
            IMAGE = Open(IMAGE_FILE.as_ptr().cast(), MODE_OLDFILE);
            return if IMAGE == 0 { ERR_NO_FILE } else { 0 };
        }
        diskport = CreatePort(std::ptr::null(), 0);
        if diskport.is_null() {
            return ERR_NO_FILE;
        }
        set_flag(AL_PORT);
        diskreq1 = CreateExtIO(diskport, size_of::<IoExtTd>() as i32).cast();
        if diskreq1.is_null() {
            return ERR_NO_REQUEST;
        }
        set_flag(AL_IOREQ);
        if OpenDevice(TD_NAME.as_ptr().cast(), 0, diskreq1.cast(), 0) != 0 {
            return ERR_NO_DEVICE;
        }
        set_flag(AL_TDISK);
        for num in 0..LOADING_REQUESTS {
            diskreqs[num] = *diskreq1;
        }
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn FreeDiskIO() {
    unsafe {
        if IMAGE != 0 {
            Close(IMAGE);
        }
        if has_flag(AL_TDISK) {
            CloseDevice(diskreq1.cast());
        }
        if has_flag(AL_IOREQ) {
            DeleteExtIO(diskreq1.cast());
        }
        if has_flag(AL_PORT) {
            DeletePort(diskport);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn WaitDiskIO(num: i32) {
    if !from_file() {
        unsafe { WaitIO(request(num as usize).cast()) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn InvalidDiskIO(num: i32) {
    if !from_file() {
        unsafe { (*request(num as usize)).command = CMD_INVALID };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn CheckDiskIO(num: i32) -> i32 {
    if from_file() {
        return 1;
    }
    unsafe { CheckIO(request(num as usize).cast()) }
}

#[unsafe(no_mangle)]
pub extern "C" fn IsReadDiskIO(num: i32) -> i32 {
    (!from_file() && unsafe { (*request(num as usize)).command } == CMD_READ) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn WaitLastDiskIO() {
    if !from_file() {
        unsafe { WaitIO(lastreq.cast()) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn InvalidLastDiskIO() {
    if !from_file() {
        unsafe { (*lastreq).command = CMD_INVALID };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn CheckLastDiskIO() -> i32 {
    if from_file() {
        return 1;
    }
    unsafe { CheckIO(lastreq.cast()) }
}

#[unsafe(no_mangle)]
pub extern "C" fn IsReadLastDiskIO() -> i32 {
    (!from_file() && unsafe { (*lastreq).command } == CMD_READ) as i32
}

/// Starts reading `b_count` blocks from block `f_block` into `buffer`, using request `dr` when
/// the trackdisk is in use. Reads from the `image` file immediately otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn load_track_range(f_block: i16, b_count: i16, buffer: *mut c_void, dr: i16) {
    unsafe {
        if from_file() {
            Seek(IMAGE, f_block as i32 * BLOCK_SIZE, OFFSET_BEGINNING);
            Read(IMAGE, buffer.cast(), b_count as i32 * BLOCK_SIZE);
            return;
        }
        lastreq = request(dr as usize);
        if (*lastreq).command == CMD_READ {
            WaitIO(lastreq.cast());
        }
        *lastreq = *diskreq1;
        (*lastreq).length = (b_count as i32 * BLOCK_SIZE) as u32;
        (*lastreq).data = buffer;
        (*lastreq).command = CMD_READ;
        (*lastreq).offset = (f_block as i32 * BLOCK_SIZE) as u32;
        SendIO(lastreq.cast());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_off() {
    if from_file() {
        return;
    }
    unsafe {
        let req = request(MOTOR_REQUEST);
        *req = *diskreq1;
        (*req).length = 0;
        (*req).command = TD_MOTOR;
        DoIO(req.cast());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn IsHardDrive() -> i16 {
    from_file() as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_is_the_size_of_the_c_struct() {
        assert_eq!(size_of::<IoExtTd>(), 64);
    }
}
