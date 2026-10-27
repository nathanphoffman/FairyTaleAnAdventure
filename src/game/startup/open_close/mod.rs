//! Opening and closing everything the game needs (from csrc/fmain.c): `open_all` sets up the screen,
//! memory, fonts, sound and input, and `close_all` ends the program.

mod close;

mod open;

pub use self::close::close_all;
pub use self::open::open_all;

// Sizes of the memory areas the game loads its data into.
const S_WAVBUF: usize = 128 * 8;
const S_VOLBUF: usize = 10 * 256;
const VOICE_SZ: usize = S_WAVBUF + S_VOLBUF;
const SCORE_SZ: usize = 5900;
const IPLAN_SZ: usize = 16384; // one plane of 256 chars
const IMAGE_SZ: usize = IPLAN_SZ * 5;
const SHAPE_SZ: usize = 78000;
const SHADOW_SZ: usize = 8192 + 4096; // background masks
const SECTOR_SZ: usize = (128 * 256) + 4096; // 256 sectors, plus the region map
const SECTOR_OFF: usize = 128 * 256;
const SAMPLE_SZ: usize = 5632;
const TERRA_SZ: usize = 1024;
const MAXSHAPES: usize = 25;

/// Where the saved backgrounds start in the blitter's work memory (after the shape queue).
const BACKSAVE_OFFSET: usize = 962;
