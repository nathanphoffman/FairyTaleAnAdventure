//! Reading the game's data from the disk image (from csrc/hdrive.c).
//!
//! Nearly all the game's graphics, maps and sounds are in one big file, `image`, a dump of the
//! original floppy disk (1760 blocks of 512 bytes). The game asks for "N blocks starting at block B"
//! with `load_track_range`. On the Amiga those reads ran in the background through the trackdisk
//! device; here every read is finished when the call returns, so the functions that wait for or
//! check a read just report that it is done.

use crate::platform::paths::resolve;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Mutex;

/// The disk image file.
const IMAGE_FILE: &str = "image";

/// Error code returned by `open_disk` when there is no image.
const ERR_NO_FILE: i32 = 30;

/// Bytes in a disk block.
const BLOCK_SIZE: u64 = 512;

static IMAGE: Mutex<Option<File>> = Mutex::new(None);

fn image() -> std::sync::MutexGuard<'static, Option<File>> {
    IMAGE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Opens the `image` file. Returns 0, or an error code.
pub fn open_disk() -> i32 {
    match resolve(IMAGE_FILE, false).and_then(|path| File::open(path).ok()) {
        Some(file) => {
            *image() = Some(file);

            0
        }

        None => ERR_NO_FILE,
    }
}

/// Closes the image file.
pub fn close_disk() {
    *image() = None;
}

/// Reads `count` blocks from block `first`. A read past the end of the image is padded with zeros.
pub fn read_blocks(first: i16, count: i16) -> Vec<u8> {
    let mut data = vec![0u8; count.max(0) as usize * BLOCK_SIZE as usize];
    let mut guard = image();

    if let Some(file) = guard.as_mut() {
        read_into(file, first, &mut data);
    }

    data
}

/// Reads the bytes at block `first` of the image into `buffer`, as far as the image goes.
fn read_into(file: &mut File, first: i16, buffer: &mut [u8]) {
    if file
        .seek(SeekFrom::Start(first as u64 * BLOCK_SIZE))
        .is_err()
    {
        eprintln!("Could not find block {first} of the disk image");

        return;
    }

    let mut done = 0;

    while done < buffer.len() {
        match file.read(&mut buffer[done..]) {
            Ok(0) => break,
            Ok(n) => done += n,
            Err(e) => {
                eprintln!("Could not read the disk image: {e}");

                break;
            }
        }
    }
}

/// Reads `count` blocks from block `first` into `buffer`. The game used a different request number
/// for each kind of data so that several reads could be under way at once on a real disk.
pub fn load_track_range(first: i16, count: i16, buffer: &mut [u8], _request: i16) {
    let mut guard = image();
    let Some(file) = guard.as_mut() else {
        return;
    };

    let bytes = (count.max(0) as usize * BLOCK_SIZE as usize).min(buffer.len());

    read_into(file, first, &mut buffer[..bytes]);
}

/// Waits for read request `num` to finish (it already has).
pub fn wait_for_read(_num: i32) {}

/// Marks read request `num` as unused.
pub fn mark_read_unused(_num: i32) {}

/// Whether request `num` has finished (always).
pub fn read_finished(_num: i32) -> i32 {
    1
}

/// Whether request `num` is a read in progress (never).
pub fn is_read_pending(_num: i32) -> i32 {
    0
}

pub fn last_read_finished() -> i32 {
    1
}

pub fn is_last_read_pending() -> i32 {
    0
}

/// Turns the floppy drive motor off after loading (nothing to do for a file).
pub fn motor_off() {}
