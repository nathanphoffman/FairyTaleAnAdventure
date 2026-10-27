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

use crate::platform::exec::{alloc_mem, type_of_mem};

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
        if type_of_mem(oldpointer.cast()) & MEMF_CHIP != 0 {
            return oldpointer;
        }

        let newpointer = alloc_mem(size as u32, MEMF_CHIP).cast::<u8>();
        std::ptr::copy_nonoverlapping(oldpointer, newpointer, size as usize);

        newpointer
    }
}

/// Was the manual-lookup riddle (copy protection, typing a word from the manual); it is skipped.
pub fn copy_protection_riddle() -> i32 {
    1
}
