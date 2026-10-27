//! Reading blocks: asking for a range of tracks, waiting for a read to finish and the motor.

use super::{
    CheckIO, DISKREQ1, DISKREQS, DoIO, FROM_FILE, IMAGE, IoExtTd, LASTREQ, Read, Seek, SendIO,
    WaitIO,
};
use std::ffi::c_void;

const OFFSET_BEGINNING: i32 = -1;
const CMD_INVALID: u16 = 0;
const CMD_READ: u16 = 2;
const TD_MOTOR: u16 = 9;
/// Bytes in a disk block.
const BLOCK_SIZE: i32 = 512;
const MOTOR_REQUEST: usize = 9;

/// True when blocks come from the `image` file instead of trackdisk.
fn from_file() -> bool {
    unsafe { FROM_FILE }
}

/// The I/O request slot `num`. Slots 0-8 are for loading, 9 is for the drive motor.
pub(super) fn request(num: usize) -> *mut IoExtTd {
    unsafe { &raw mut DISKREQS[num] }
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
