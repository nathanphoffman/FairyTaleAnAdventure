//! Replacements for the Amiga system libraries the original code called. The game calls them as
//! ordinary Rust functions (`gfx::set_a_pen`, `dos::open_file`, ...), named after the Amiga call
//! they replace; they work on the Amiga-shaped structs in `crate::amiga`.

pub mod audio;

pub mod display;

pub mod dos;

pub mod input;
