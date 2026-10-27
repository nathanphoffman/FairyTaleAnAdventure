//! Sprite drawing routines that drove the Amiga blitter in fsubs.asm (from csrc/asm_blit.c).
//!
//! The blitter combines up to three source channels (A, B, C) into a destination D, 16 bits at
//! a time, with an optional barrel shift of A and B, first/last word masks on A, and a
//! "minterm" saying how the bits combine. `blit` does the same in software for the modes the
//! game uses, so each routine here sets up the same values as its assembly original and the
//! drawing comes out the same. Words are big-endian in memory (leftmost pixels in the first byte).
//!
//! How a sprite gets onto the screen (see the main loop in csrc/fmain.c):
//! 1. `save_background` copies the background under the sprite's rectangle to a save area.
//! 2. `add_scenery_mask` copies, for every scenery tile that should hide the sprite (trees, walls ...), the
//!    tile's mask into a screen-sized scenery mask.
//! 3. `combine_masks` combines the sprite's own mask with that: the sprite is visible only where it is
//!    opaque and nothing is in front of it.
//! 4. `draw_shape` draws the sprite's pixels through that mask into the 5 bitplanes of the screen.
//! 5. Next frame `restore_background` puts the saved background back, which erases the sprite.
//!
//! `make_sprite_masks` builds each frame's own mask once, when the sprite sheet is loaded.

mod engine;

mod sprites;

pub use self::sprites::{
    add_scenery_mask, clear_memory, combine_masks, draw_shape, make_sprite_masks,
    restore_background, save_background,
};

/// Mirror of `struct sshape` in csrc/ftale.h: how one sprite is drawn this frame and where its
/// background is saved. `coff` is the byte offset of the sprite in a bitplane and `cmod` the bytes
/// to skip to the next line; `blitsize` is the blitter size word (height * 64 + width in words).
#[repr(C)]
pub struct SShape {
    pub backsave: *mut u8,
    pub savesize: i16,
    pub blitsize: i16,
    pub coff: i16,
    pub cmod: i16,
}

// Which channels a blit uses.
const USE_A: u8 = 1;
const USE_B: u8 = 2;
const USE_C: u8 = 4;
const USE_D: u8 = 8;

// The minterms the game uses (result for each combination of the A, B and C bits).
const D_EQUALS_B: u32 = 0xcc;
const D_EQUALS_A_AND_NOT_C: u32 = 0x50;
const D_EQUALS_A_THEN_B_ELSE_C: u32 = 0xca;

/// Bitplanes in the play field.
const PLANE_COUNT: i32 = 5;
const SHIFT_MASK: i16 = 15;
const ALL_BITS: u32 = 0xffff;

/// Bytes in the mask for one scenery tile (32 lines of one word).
const TILE_MASK_BYTES: usize = 64;
const TILE_LINES: usize = 32;

/// The scenery mask starts this many bytes into `bmask_mem`.
const MASK_START: usize = 2;
