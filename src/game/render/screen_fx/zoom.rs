//! The intro's zoom: the picture grows from the middle of the screen.

use super::fading::fade_page;
use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::data::tables::intro_colors;
use crate::game::render::display::delay;
use crate::game::render::display::flip_pages;

// The intro zoom: the picture is centred on a 320 x 200 screen and its shape is 8:5.
const SCREEN_CENTER_X: i32 = 160;
const SCREEN_CENTER_Y: i32 = 100;
const ZOOM_ASPECT: (i32, i32) = (5, 8);
const ZOOM_TEXT_HEIGHT: i32 = 95;
const ZOOM_FADE_OFFSETS: (i32, i32, i32) = (40, 70, 100);

/// The size of the picture during the intro's zoom: `x` is half the width in pixels (0 to 160),
/// the height keeps the 8:5 shape. Moves the page and text view ports to match, fades the colours
/// with the size (black when small), and shows the result.
pub fn set_intro_zoom(game: &mut Game, video: &mut Video, x: i32) {
    let y = (x * ZOOM_ASPECT.0) / ZOOM_ASPECT.1;

    delay(game, video, 2);

    let left = (SCREEN_CENTER_X - x) as i16;
    let top = (SCREEN_CENTER_Y - y) as i16;

    for ras in &mut video.screen.ras[..2] {
        ras.x = left;
        ras.y = top;
    }

    video.screen.vp_page.x = left;
    video.screen.vp_page.width = (x + x) as i16;
    video.screen.vp_page.y = top;
    video.screen.vp_page.height = (y + y) as i16;

    video.screen.vp_text.x = 0;
    video.screen.vp_text.y = 0;
    video.screen.vp_text.height = (ZOOM_TEXT_HEIGHT - y) as i16;

    fade_page(
        game,
        video,
        (y * 2 - ZOOM_FADE_OFFSETS.0) as i16,
        (y * 2 - ZOOM_FADE_OFFSETS.1) as i16,
        (y * 2 - ZOOM_FADE_OFFSETS.2) as i16,
        0,
        &intro_colors,
    );

    flip_pages(game, video);
}
