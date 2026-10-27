//! Fonts and text drawing.

mod font;

mod render;

pub use self::font::{close_font, load_seg, open_font, set_font, unload_seg};
pub use self::render::text_out;
