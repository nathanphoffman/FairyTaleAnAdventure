//! Switching between the play view and the map page that shows messages.

use super::fading::fade_down;
use crate::game::data::object_tables::page_colors;
use crate::game::render::display::flip_pages;
use crate::game::render::screen::{HIRES, PortId, SPRITES, VP_HIDE, ViewPortId, screen};
use crate::gfx::draw::JAM1;

const MAP_PEN: u8 = 24;

/// `view_status` values: showing a map message, and showing the normal view again.
const VIEW_MESSAGE: i8 = 2;
const VIEW_NORMAL_AGAIN: i8 = 3;

/// Shows the page without scrolling.
pub fn show_still_screen() {
    let screen = screen();
    let ras = screen.drawing_page().ras;
    screen.ras[ras].x = 0;
    screen.ras[ras].y = 0;

    flip_pages();
}

/// Switches to the map page for a message (the text panel is hidden) and fades in.
pub fn map_message() {
    fade_down();

    let screen = screen();
    screen.current_port = PortId::Map;
    screen.port_mut(PortId::Map).target = Some(screen.drawing_bitmap_id());
    screen.set_draw_mode(PortId::Map, JAM1);
    screen.set_a_pen(PortId::Map, MAP_PEN);
    screen.clear(PortId::Map, 0);

    screen.vp_text.modes = HIRES | SPRITES | VP_HIDE;
    show_still_screen();
    screen.load_colors(ViewPortId::Page, unsafe { &*(&raw const page_colors) });

    screen.view_status = VIEW_MESSAGE;
}

/// Goes back from the map message to the normal view.
pub fn message_off() {
    fade_down();

    let screen = screen();
    screen.current_port = PortId::Text;
    screen.vp_text.modes = HIRES | SPRITES;
    flip_pages();
    screen.view_status = VIEW_NORMAL_AGAIN;
}
