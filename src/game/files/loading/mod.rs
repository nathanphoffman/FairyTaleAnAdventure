//! Loading sprite files, songs and the intro pictures (from csrc/fmain2.c).
//!
//! Sprites: every kind of character (the three brothers, monsters, people, the raft, the turtle ...)
//! is stored on the disk image as a block of frames. `sprite_files` lists them. `load_sprite_file` copies one
//! into `shape_mem`, a big scratch area, and `finish_sprite_load` builds a transparency mask for it so the
//! blitter knows which pixels to leave alone. Each loaded sprite sequence takes its frames (5
//! bitplanes) plus one mask plane, in memory order.
//!
//! Also here: reading the music scores (`read_score`) and the story book page-turn animation
//! (`show_story_page`, `turn_page`).

mod sound_data;

mod sprites;

mod story_pages;

pub use self::sound_data::{play_effect, read_sample, read_score};
pub use self::sprites::{
    finish_sprite_load, load_pending_region, load_sprite_file, load_start_sprites, sprite_files,
};

pub use self::story_pages::{intro_skip_requested, intro_skipped, show_story_page};

use crate::amiga::{BPTR, BitMap};

use std::ffi::c_char;

unsafe extern "C" {
    fn Open(name: *const c_char, mode: i32) -> BPTR;

    fn Close(file: BPTR);

    fn Read(file: BPTR, buf: *mut u8, len: i32) -> i32;

    fn Delay(ticks: i32);

    fn AllocMem(size: u32, flags: u32) -> *mut u8;

    fn TypeOfMem(p: *const u8) -> u32;

    fn BltBitMap(
        src: *mut BitMap,
        sx: i32,
        sy: i32,
        dst: *mut BitMap,
        dx: i32,
        dy: i32,
        w: i32,
        h: i32,
        minterm: u32,
        mask: u32,
        temp: *mut u8,
    ) -> i32;

    fn playsample(data: *const u8, len: i32, period: i32);
}

const CARRIER: u8 = 5;
const DRAGON: u8 = 6;
const MEMF_CHIP: u32 = 2;

/// Mirror of `struct cfile_info`: one loadable sprite file (a character set).
/// `width` is in 16-pixel words, `height` in lines, `count` the number of frames.
#[repr(C)]
pub struct CFileInfo {
    pub width: u8,
    pub height: u8,
    pub count: u8,
    /// Disk blocks to read.
    pub numblocks: u8,
    /// Which `seq_list` slot to load into.
    pub seq_num: u8,
    pub file_id: u16,
}

const fn cfile(
    width: u8,
    height: u8,
    count: u8,
    numblocks: u8,
    seq_num: u8,
    file_id: u16,
) -> CFileInfo {
    CFileInfo {
        width,
        height,
        count,
        numblocks,
        seq_num,
        file_id,
    }
}

/// Was the copy protection check (the original checked the disk's volume date); the protection is
/// removed, so it always passes.
pub fn copy_protection_disk_check() -> i32 {
    1
}

/// Was part of the copy protection; now just the (always passing) check.
pub fn copy_protection_check() {
    copy_protection_disk_check();
}

/// Copies `oldpointer` into chip memory (memory the Amiga's graphics chips can see) if it is not there already.
pub fn into_chip(oldpointer: *mut u8, size: i32) -> *mut u8 {
    unsafe {
        // Masks with MEMF_CHIP, keeping only the bits that matter here.
        if TypeOfMem(oldpointer) & MEMF_CHIP != 0 {
            return oldpointer;
        }

        let newpointer = AllocMem(size as u32, MEMF_CHIP);
        std::ptr::copy_nonoverlapping(oldpointer, newpointer, size as usize);

        newpointer
    }
}

/// Was the manual-lookup riddle (copy protection, typing a word from the manual); it is skipped.
pub fn copy_protection_riddle() -> i32 {
    1
}
