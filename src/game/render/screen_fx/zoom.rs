//! The intro's zoom: the picture grows from the middle of the screen.

use super::fading::fade_page;
use crate::platform::display::delay;
use crate::platform::gfx::make_vport;

use crate::game::data::globals::main_view;
use crate::game::data::state::the_screen;
use crate::game::data::tables::intro_colors;
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
pub fn set_intro_zoom(x: i32) {
    unsafe {
        let y = (x * ZOOM_ASPECT.0) / ZOOM_ASPECT.1;

        delay(2);

        let left = (SCREEN_CENTER_X - x) as i16;
        the_screen.ri_page2.RxOffset = left;
        the_screen.ri_page1.RxOffset = left;
        the_screen.vp_page.DxOffset = left;
        the_screen.vp_page.DWidth = (x + x) as i16;

        let top = (SCREEN_CENTER_Y - y) as i16;
        the_screen.ri_page2.RyOffset = top;
        the_screen.ri_page1.RyOffset = top;
        the_screen.vp_page.DyOffset = top;
        the_screen.vp_page.DHeight = (y + y) as i16;

        the_screen.vp_text.DxOffset = 0;
        the_screen.vp_text.DyOffset = 0;
        the_screen.vp_text.DHeight = (ZOOM_TEXT_HEIGHT - y) as i16;

        let colors = (&raw const intro_colors).cast::<u16>();
        fade_page(
            (y * 2 - ZOOM_FADE_OFFSETS.0) as i16,
            (y * 2 - ZOOM_FADE_OFFSETS.1) as i16,
            (y * 2 - ZOOM_FADE_OFFSETS.2) as i16,
            0,
            colors,
        );

        make_vport(&raw mut main_view, &raw mut the_screen.vp_text);
        flip_pages();
    }
}
