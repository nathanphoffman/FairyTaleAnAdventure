//! The intro's zoom: the picture grows from the middle of the screen.

use super::fading::fade_page;
use crate::game::data::state::Game;
use crate::game::data::tables::intro_colors;
use crate::game::output::Output;
use crate::game::render::display::delay;
use crate::game::render::display::flip_pages;

// The intro zoom: the picture's shape is 8:5.
const ZOOM_ASPECT: (i32, i32) = (5, 8);
const ZOOM_FADE_OFFSETS: (i32, i32, i32) = (40, 70, 100);

/// The size of the picture during the intro's zoom: `x` is half the width in pixels (0 to 160),
/// the height keeps the 8:5 shape. Moves the page and text view ports to match, fades the colours
/// with the size (black when small), and shows the result.
pub fn set_intro_zoom(game: &mut Game, out: &mut dyn Output, x: i32) {
    let y = (x * ZOOM_ASPECT.0) / ZOOM_ASPECT.1;

    delay(game, out, 2);

    out.set_zoom_size(x, y);

    fade_page(
        game,
        out,
        (
            (y * 2 - ZOOM_FADE_OFFSETS.0) as i16,
            (y * 2 - ZOOM_FADE_OFFSETS.1) as i16,
            (y * 2 - ZOOM_FADE_OFFSETS.2) as i16,
        ),
        0,
        &intro_colors,
    );

    flip_pages(game, out);
}
