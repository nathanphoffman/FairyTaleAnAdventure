//! Amiga graphics library: planar bitmaps, rastports, drawing, text, palette, views.
//!
//! Bitmaps stay planar (one byte array per bitplane, 1 bit per pixel) exactly like the original,
//! because the game code reads and writes them directly. Only the final display step
//! (display.rs) converts them to RGB.

mod area;

mod bitmap;

mod blit;

mod draw;

mod text;

mod view;
