//! Switching between the play view and the map page that shows messages.

use super::fading::fade_down;
use super::{COLOR_COUNT, HIRES, LoadRGB4, SPRITES, SetAPen, SetDrMd, SetRast, VP_HIDE};
use crate::game::data::object_tables::page_colors;
use crate::game::data::state::the_screen;
use crate::game::render::display::flip_pages;

const MAP_PEN: u32 = 24;
const JAM1: u32 = 0;
/// `view_status` values: showing a map message, and showing the normal view again.
const VIEW_MESSAGE: i8 = 2;
const VIEW_NORMAL_AGAIN: i8 = 3;

/// Shows the page without scrolling.
pub fn show_still_screen() {
    unsafe {
        let page = (*the_screen.drawing_page).ri_page;
        (*page).RxOffset = 0;
        (*page).RyOffset = 0;
        flip_pages();
    }
}

/// Switches to the map page for a message (the text panel is hidden) and fades in.
pub fn map_message() {
    unsafe {
        fade_down();
        the_screen.current_rastport = &raw mut the_screen.rp_map;
        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;
        SetDrMd(the_screen.current_rastport, JAM1);
        SetAPen(the_screen.current_rastport, MAP_PEN);
        SetRast(the_screen.current_rastport, 0);

        the_screen.vp_text.Modes = HIRES | SPRITES | VP_HIDE;
        show_still_screen();
        LoadRGB4(
            &raw mut the_screen.vp_page,
            (&raw const page_colors).cast(),
            COLOR_COUNT as i32,
        );

        the_screen.view_status = VIEW_MESSAGE;
    }
}

/// Goes back from the map message to the normal view.
pub fn message_off() {
    unsafe {
        fade_down();
        the_screen.current_rastport = &raw mut the_screen.rp_text;
        the_screen.vp_text.Modes = HIRES | SPRITES;
        flip_pages();
        the_screen.view_status = VIEW_NORMAL_AGAIN;
    }
}
