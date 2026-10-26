//! Loading sprite files, songs and the intro pictures (from csrc/fmain2.c).
//!
//! Sprites: every kind of character (the three brothers, monsters, people, the raft, the turtle ...)
//! is stored on the disk image as a block of frames. `cfiles` lists them. `read_shapes` copies one
//! into `shape_mem`, a big scratch area, and `prep` builds a transparency mask for it so the
//! blitter knows which pixels to leave alone. Each loaded sprite sequence takes its frames (5
//! bitplanes) plus one mask plane, in memory order.
//!
//! Also here: reading the music scores (`read_score`) and the story book page-turn animation
//! (`copypage`, `flipscan`).

use super::asm_blit::make_mask;
use super::hdrive::{
    CheckLastDiskIO, InvalidDiskIO, IsReadLastDiskIO, WaitDiskIO, load_track_range, motor_off,
};
use super::iffsubs::unpackbrush;
use super::input::getkey;
use super::display::pagechange;
use super::types::{FPage, Menu, SeqInfo};
use crate::amiga::{BPTR, BitMap};
use std::ffi::c_char;

unsafe extern "C" {
    static brother: i16;
    static actor_file: i16;
    static set_file: i16;
    static region_num: u16;
    static mut new_region: u16;
    static shape_mem: *mut u8;
    static mut nextshape: *mut u8;
    static mut seq_list: [SeqInfo; 0];
    static mut pagea: BitMap;
    static mut pageb: BitMap;
    static fp_drawing: *mut FPage;
    static scoremem: *mut u8;
    static mut track: [*mut u8; 32];

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
    static sample_mem: *mut u8;
    static mut sample: [*mut u8; 6];
    static mut sample_size: [u32; 6];
    static menus: [Menu; 10];
    fn load_all();
    fn load_new_region();
}

// The sequence slots in `seq_list`.
const PHIL: u8 = 0;
const OBJECTS: u8 = 1;
const ENEMY: u8 = 2;
const RAFT: u8 = 3;
const SETFIG: u8 = 4;
const CARRIER: u8 = 5;
const DRAGON: u8 = 6;

const MEMF_CHIP: u32 = 2;
const MODE_OLDFILE: i32 = 1005;
const BLIT_COPY: u32 = 0xC0;
const ALL_PLANES: u32 = 0x1f;

/// Scratch memory for sprites: bytes available in `shape_mem`.
pub const SHAPE_SIZE: usize = 78000;
/// A sprite sequence is loaded with room for its mask and the shifted copies after it.
const SIZE_WITH_MASK: usize = 6;
const SIZE_WITHOUT_MASK: usize = 5;
/// Drive request used when loading sprite files.
const SPRITE_REQUEST: i16 = 8;

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

const fn cfile(width: u8, height: u8, count: u8, numblocks: u8, seq_num: u8, file_id: u16) -> CFileInfo {
    CFileInfo { width, height, count, numblocks, seq_num, file_id }
}

/// The sprite files on the disk image. Index 0-2 are the three brothers, 3 the objects, 4 the raft,
/// 5 the turtle, 6-9 and 12 monster sets, 10 the dragon, 11 the bird, 13-17 the people (wizard,
/// royal court, bartender, witch, ranger). `file_id` is the first disk block.
///
/// Entry 12 (snake and salamander) uses the same block as entry 0, which looks like a leftover
/// placeholder in the original; it is copied as it was.
#[unsafe(no_mangle)]
pub static cfiles: [CFileInfo; 18] = [
    cfile(1, 32, 67, 42, PHIL, 1376),    // julian
    cfile(1, 32, 67, 42, PHIL, 1418),    // phillip
    cfile(1, 32, 67, 42, PHIL, 1460),    // kevin
    cfile(1, 16, 116, 36, OBJECTS, 1312), // objects
    cfile(2, 32, 2, 3, RAFT, 1348),      // raft
    cfile(2, 32, 16, 20, CARRIER, 1351), // turtle
    cfile(1, 32, 64, 40, ENEMY, 960),    // ogre file
    cfile(1, 32, 64, 40, ENEMY, 1080),   // ghost file
    cfile(1, 32, 64, 40, ENEMY, 1000),   // dknight file (spiders)
    cfile(1, 32, 64, 40, ENEMY, 1040),   // necromancer (farmer / loraii)
    cfile(3, 40, 5, 12, DRAGON, 1160),   // dragon
    cfile(4, 64, 8, 40, CARRIER, 1120),  // bird
    cfile(1, 32, 64, 40, ENEMY, 1376),   // snake and salamander
    cfile(1, 32, 8, 5, SETFIG, 936),     // wizard/priest
    cfile(1, 32, 8, 5, SETFIG, 931),     // royal set
    cfile(1, 32, 8, 5, SETFIG, 941),     // bartender
    cfile(1, 32, 8, 5, SETFIG, 946),     // witch
    cfile(1, 32, 8, 5, SETFIG, 951),     // ranger/begger
];

// Indexes into `cfiles` that are always loaded together at the start.
const OBJECTS_FILE: i32 = 3;
const RAFT_FILE: i32 = 4;

/// Loads the sprite sets the game starts with: objects, the current brother, the raft, the
/// current monsters and set figures, then the region.
#[unsafe(no_mangle)]
pub extern "C" fn shape_read() {
    unsafe {
        nextshape = shape_mem;
        read_shapes(OBJECTS_FILE);
        prep(OBJECTS as i16);
        read_shapes(brother as i32 - 1);
        prep(PHIL as i16);
        read_shapes(RAFT_FILE);
        prep(RAFT as i16);
        sequence(ENEMY).location = nextshape;
        read_shapes(actor_file as i32);
        prep(cfiles[actor_file as usize].seq_num as i16);
        read_shapes(set_file as i32);
        prep(SETFIG as i16);
        new_region = region_num;
        load_all();
        motor_off();
    }
}

/// The `seq_list` entry for a sprite slot (PHIL, OBJECTS, ENEMY ...).
unsafe fn sequence(slot: u8) -> &'static mut SeqInfo {
    unsafe { &mut *(&raw mut seq_list).cast::<SeqInfo>().add(slot as usize) }
}

/// Starts loading sprite file `num` into the next free shape memory, if it fits.
///
/// Each sequence is stored as 5 planes of `size` bytes followed by a one-plane mask (`size` bytes).
/// The check makes sure that all of it, with room to spare, fits in the scratch area.
#[unsafe(no_mangle)]
pub extern "C" fn read_shapes(num: i32) {
    unsafe {
        let file = &cfiles[num as usize];
        let slot = sequence(file.seq_num);
        slot.bytes = file.height as i16 * file.width as i16 * 2;
        let size = slot.bytes as usize * file.count as usize;
        slot.location = nextshape;
        slot.width = file.width as i16;
        slot.height = file.height as i16;
        slot.count = file.count as i16;
        if nextshape.add(size * SIZE_WITH_MASK) <= shape_mem.add(SHAPE_SIZE) {
            load_track_range(file.file_id as i16, file.numblocks as i16, nextshape.cast(), SPRITE_REQUEST);
            nextshape = nextshape.add(size * SIZE_WITHOUT_MASK);
            slot.maskloc = nextshape;
            nextshape = nextshape.add(size);
        }
    }
}

/// Was the copy protection check (the original checked the disk's volume date); the protection is
/// removed, so it always passes.
#[unsafe(no_mangle)]
pub extern "C" fn cpytest() -> i32 {
    1
}

/// Was part of the copy protection; now just the (always passing) check.
#[unsafe(no_mangle)]
pub extern "C" fn seekn() {
    cpytest();
}

/// Waits for the sprite load to finish and builds the masks for slot `slot`.
#[unsafe(no_mangle)]
pub extern "C" fn prep(slot: i16) {
    WaitDiskIO(SPRITE_REQUEST as i32);
    InvalidDiskIO(SPRITE_REQUEST as i32);
    unsafe {
        let s = sequence(slot as u8);
        make_mask(s.location, s.maskloc, s.width as i32, s.height as i32, s.count as i32);
    }
}

/// Loads the next region once the previous disk read is done. On the Amiga the disk was slow, so
/// the game kept playing while a region loaded in the background and called this to check.
#[unsafe(no_mangle)]
pub extern "C" fn load_next() {
    if IsReadLastDiskIO() == 0 || CheckLastDiskIO() != 0 {
        unsafe { load_new_region() };
    }
}

// Songs: up to 28 tracks, packed into the score memory.
const TRACK_COUNT: usize = 4 * 7;
const SCORE_MEMORY: i32 = 5900;

/// Reads the song tracks from the file `songs` into `scoremem`.
///
/// The file is a list of tracks, each a 4 byte big-endian length (counted in 16-bit words) and
/// then that many words. There are up to 28 tracks (4 voices times 7 songs). Reading stops early
/// if they would not fit in the 5900 bytes of score memory.
#[unsafe(no_mangle)]
pub extern "C" fn read_score() {
    unsafe {
        let file = Open(c"songs".as_ptr(), MODE_OLDFILE) as i32;
        if file == 0 {
            return;
        }
        let (mut loaded, mut tracks) = (0, 0);
        for _ in 0..TRACK_COUNT {
            let mut length = 0i32;
            Read(file as BPTR, (&raw mut length).cast(), 4);
            length = length.swap_bytes(); // the length is stored big-endian
            if length * 2 + loaded > SCORE_MEMORY {
                break;
            }
            track[tracks] = scoremem.offset(loaded as isize);
            tracks += 1;
            Read(file as BPTR, scoremem.offset(loaded as isize), length * 2);
            loaded += length * 2;
        }
        Close(file as BPTR);
    }
}

/// Set once the player has pressed Space to skip the intro.
#[unsafe(no_mangle)]
pub static mut skipp: i8 = 0;

/// Whether the intro is being skipped. Once Space has been seen the skip sticks (the original
/// reset it on every poll without a key).
#[unsafe(no_mangle)]
pub extern "C" fn skipint() -> i32 {
    unsafe {
        if getkey() == b' ' as i32 {
            skipp = 1;
        }
        skipp as i32
    }
}

const SCREEN_WIDTH: i32 = 320;
const SCREEN_HEIGHT: i32 = 200;
/// Where the first story picture goes on the page.
const PICTURE_X: i16 = 4;
const PICTURE_Y: i16 = 24;
/// Ticks to show a page before turning it.
const PAGE_DELAY: i32 = 350;

/// Shows two pictures on the story page, then turns it over.
/// The new pictures are drawn onto `pageb` while `pagea` keeps the old page for the turn.
#[unsafe(no_mangle)]
pub extern "C" fn copypage(br1: *const c_char, br2: *const c_char, x: i16, y: i16) {
    unsafe {
        if skipp != 0 {
            return;
        }
        Delay(PAGE_DELAY);
        BltBitMap(&raw mut pageb, 0, 0, &raw mut pagea, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, BLIT_COPY, ALL_PLANES, std::ptr::null_mut());
        unpackbrush(br1, &raw mut pageb, PICTURE_X, PICTURE_Y);
        unpackbrush(br2, &raw mut pageb, x, y);
        if skipint() != 0 {
            return;
        }
        flipscan();
        skipint();
    }
}

// The page-turn animation, 22 frames: strip step, strip width and delay for each.
const FLIP_RATE: [i16; 22] = [8, 6, 5, 4, 3, 2, 3, 5, 13, 0, 0, 13, 5, 3, 2, 3, 4, 5, 6, 8, 0, 0];
const FLIP_WIDE: [i16; 22] = [7, 5, 4, 3, 2, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 2, 3, 4, 5, 7, 0, 0];
const FLIP_DELAY: [i32; 22] = [12, 9, 6, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 6, 9, 0, 0];
/// The first half of the frames turn the right page up, the rest lay the left one down.
const FIRST_HALF: usize = 11;
const PAGE_HALF_WIDTH: i32 = 135;
const PAGE_STRIP_END: i16 = 136;

/// Copies a rectangle between bitmaps (all five planes, plain copy).
unsafe fn copy_rect(src: *mut BitMap, sx: i32, sy: i32, dst: *mut BitMap, dx: i32, dy: i32, w: i32, h: i32) {
    unsafe { BltBitMap(src, sx, sy, dst, dx, dy, w, h, BLIT_COPY, ALL_PLANES, std::ptr::null_mut()) };
}

/// The page turn animation, 22 frames.
///
/// The first 11 frames lift the right page: the old page is drawn as thin vertical strips that
/// get narrower and shorter (using `page_det` for the curl), on top of the new page. The last 11
/// frames lay the curling page down on the left. `FLIP_RATE`, `FLIP_WIDE` and `FLIP_DELAY` give
/// the strip step, strip width and pause for each frame.
#[unsafe(no_mangle)]
pub extern "C" fn flipscan() {
    unsafe {
        let (a, b) = (&raw mut pagea, &raw mut pageb);
        for frame in 0..FLIP_RATE.len() {
            let temp = (*(*fp_drawing).ri_page).BitMap;
            let mut dcol: i32 = 0;
            let rate = FLIP_RATE[frame];
            let wide = FLIP_WIDE[frame] as i32;
            if frame < FIRST_HALF {
                let bcol = 161 - wide;
                copy_rect(b, 160, 0, temp, 160, 0, PAGE_HALF_WIDTH, SCREEN_HEIGHT);
                if rate != 0 {
                    let mut h = 0;
                    let mut scol = wide as i16;
                    while scol < PAGE_STRIP_END {
                        h = super::asm_map::page_det(scol as i32);
                        copy_rect(a, bcol + scol as i32, h, temp, 161 + dcol, h, wide, SCREEN_HEIGHT - h - h);
                        dcol += wide;
                        scol += rate;
                    }
                    copy_rect(a, 296, 7, temp, 161 + dcol, h, 1, 186);
                }
            } else {
                let bcol = 160;
                if rate == 0 {
                    copy_rect(b, 24, 0, temp, 24, 0, PAGE_HALF_WIDTH, SCREEN_HEIGHT);
                } else {
                    copy_rect(a, 24, 0, temp, 24, 0, PAGE_HALF_WIDTH, SCREEN_HEIGHT);
                    let mut scol = wide as i16;
                    while scol < PAGE_STRIP_END {
                        let h = super::asm_map::page_det(scol as i32);
                        dcol += wide;
                        copy_rect(b, bcol - scol as i32, h, temp, bcol - dcol, h, wide, SCREEN_HEIGHT - h - h);
                        scol += rate;
                    }
                    let h = 7;
                    copy_rect(b, 24, h, temp, 159 - dcol, h, 1, SCREEN_HEIGHT - h - h);
                }
            }
            pagechange();
            if FLIP_DELAY[frame] != 0 {
                Delay(FLIP_DELAY[frame]);
            }
        }
    }
}

/// Copies `oldpointer` into chip memory (memory the Amiga's graphics chips can see) if it is not there already.
#[unsafe(no_mangle)]
pub extern "C" fn into_chip(oldpointer: *mut u8, size: i32) -> *mut u8 {
    unsafe {
        if TypeOfMem(oldpointer) & MEMF_CHIP != 0 {
            return oldpointer;
        }
        let newpointer = AllocMem(size as u32, MEMF_CHIP);
        std::ptr::copy_nonoverlapping(oldpointer, newpointer, size as usize);
        newpointer
    }
}

/// Was the manual-lookup riddle (copy protection, typing a word from the manual); it is skipped.
#[unsafe(no_mangle)]
pub extern "C" fn copy_protect_junk() -> i32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_table_matches_the_c_struct() {
        assert_eq!(size_of::<CFileInfo>(), 8);
        assert_eq!(cfiles.len(), 18);
        assert_eq!(cfiles[16].file_id, 946); // witch
    }

    #[test]
    fn flip_tables_have_one_entry_per_frame() {
        assert_eq!(FLIP_RATE.len(), FLIP_WIDE.len());
        assert_eq!(FLIP_RATE.len(), FLIP_DELAY.len());
    }
}

/// Where the sound effects are on the disk (block) and how many blocks they take.
const SAMPLE_BLOCK: i16 = 920;
const SAMPLE_BLOCKS: i16 = 11;
/// The disk request number used for sound samples.
const SAMPLE_REQUEST: i16 = 8;
/// There are 6 sound effects.
const SAMPLE_COUNT: usize = 6;
/// The game menu entry that switches sound effects on and off.
const GAME_MENU: usize = 4;
const SOUND_ENTRY: usize = 7;

/// Reads the sound effects from the disk into `sample_mem`. Each one is a 4 byte big-endian
/// length followed by that many bytes of sound; `sample` and `sample_size` point at them.
#[unsafe(no_mangle)]
pub extern "C" fn read_sample() {
    unsafe {
        load_track_range(SAMPLE_BLOCK, SAMPLE_BLOCKS, sample_mem.cast(), SAMPLE_REQUEST);
        WaitDiskIO(SAMPLE_REQUEST as i32);
        InvalidDiskIO(SAMPLE_REQUEST as i32);

        let mut at = sample_mem;
        for i in 0..SAMPLE_COUNT {
            let length = u32::from_be_bytes([*at, *at.add(1), *at.add(2), *at.add(3)]);
            at = at.add(4);
            sample[i] = at;
            sample_size[i] = length as u32;
            at = at.add(length as usize);
        }
    }
}

/// Plays sound effect `num` at the pitch `speed`, unless sound effects are switched off.
#[unsafe(no_mangle)]
pub extern "C" fn effect(num: i16, speed: i64) {
    unsafe {
        if menus[GAME_MENU].enabled[SOUND_ENTRY] & 1 != 0 {
            let n = num as usize;
            playsample(sample[n], (sample_size[n] / 2) as i32, speed as i32);
        }
    }
}
