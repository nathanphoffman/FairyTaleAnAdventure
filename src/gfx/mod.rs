//! Safe graphics for the game: planar bitmaps, drawing into them, fonts, and turning them into the
//! pixels that are shown.
//!
//! The original drew with Amiga system calls on structs full of raw pointers. Here a `Bitmap` owns
//! its bit planes, the drawing functions take the bitmap they draw on, and the picture on screen is
//! described by a `Scene`.

pub mod bitmap;

pub mod draw;

pub mod font;

pub mod scene;

pub use self::bitmap::{Bitmap, PAD};
pub use self::font::Font;
