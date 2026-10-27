//! Opening and closing everything the game needs (from csrc/fmain.c): `open_all` sets up the screen,
//! memory, fonts, sound and input, and `close_all` gives it all back.

mod close;

mod open;

pub use self::close::close_all;
pub use self::open::open_all;

use crate::game::data::globals::open_flags;

// Memory the game allocates (csrc/fmain.c).
const PAGE_DEPTH: i32 = 5;
const TEXT_DEPTH: i32 = 4;
const PHANTA_WIDTH: i32 = 320; // two words added for horizontal scrolling
const PAGE_HEIGHT: i16 = 143;
const RAST_HEIGHT: i32 = 200;
const TEXT_HEIGHT: i32 = 57;
const MAXSHAPES: usize = 25;
const MEMF_CLEAR: u32 = 0x10000;
const SPRITES: u16 = 0x4000;
const VP_HIDE: u16 = 0x2000;
const S_WAVBUF: i32 = 128 * 8;
const S_VOLBUF: i32 = 10 * 256;
const VOICE_SZ: u32 = (S_WAVBUF + S_VOLBUF) as u32;
const SCORE_SZ: u32 = 5900;
const IPLAN_SZ: u32 = 16384; // one plane of 256 chars
const IMAGE_SZ: u32 = IPLAN_SZ * 5;
const SHAPE_SZ: u32 = 78000;
const SHADOW_SZ: u32 = 8192 + 4096; // background masks
const SECTOR_SZ: u32 = (128 * 256) + 4096; // 256 sectors, plus the region map
const SECTOR_OFF: usize = 128 * 256;
const SAMPLE_SZ: u32 = 5632;
const TERRA_SZ: u32 = 1024;
const SPRITE_DATA_SZ: i32 = 88;
const NORM_HIGH_SZ: i32 = 16 * 16;
const MODE_OLDFILE: i32 = 1005;
const BITMAP_COUNT: u32 = 5;
const LOWRES_WIDTH: i16 = 288;
const HIRES_WIDTH: i16 = 640;
const PAGE_VIEW_HEIGHT: i16 = 140;
const PAGE_VIEW_X: i16 = 16;
const CMD_RESET: u16 = 1;
const IOF_QUICK: u8 = 1;

/// A text row is 80 bytes (640 pixels, one bit per pixel).
const TEXT_ROW_BYTES: usize = 80;

/// Where the queue of shapes to draw and the first background save sit in the text bitmap's memory.
const BACKSAVE_OFFSET: usize = 962;

// What `open_all` has set up, so `close_all` knows what to undo (`openflags` bits).
const AL_BMAP: u16 = 0x0001;
const AL_GBASE: u16 = 0x0002;
const AL_HANDLE: u16 = 0x0004;
const AL_MUSIC: u16 = 0x0008;
const AL_IMAGE: u16 = 0x0010;
const AL_SECTOR: u16 = 0x0020;
const AL_SHAPE: u16 = 0x0080;
const AL_SHADOW: u16 = 0x0100;
const AL_FONT: u16 = 0x0200;
const AL_SAMPLE: u16 = 0x0400;
const AL_TERR: u16 = 0x4000;

// Screen modes (`view_status`).
/// How `dif_x` wraps: a 10 bit signed value.
unsafe fn set_flag(flag: u16) {
    unsafe { open_flags |= flag };
}

unsafe fn has_flag(flag: u16) -> bool {
    unsafe { open_flags & flag != 0 }
}
