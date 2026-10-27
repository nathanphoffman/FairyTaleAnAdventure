//! Replacements for the Amiga system libraries the original code called.
//! Each function is exported with its Amiga name so the C game code links against it directly.

pub mod audio;

pub mod display;

pub mod dos;

pub mod exec;

pub mod gfx;

pub mod input;
