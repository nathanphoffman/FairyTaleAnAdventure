//! Switching between the play view and the map page that shows messages.

use super::fading::fade_down;
use crate::game::data::object_tables::page_colors;
use crate::game::data::state::Game;
use crate::game::render::display::flip_pages;
use crate::game::render::screen::{HIRES, PortId, SPRITES, VP_HIDE, ViewPortId};
use crate::gfx::draw::JAM1;

const MAP_PEN: u8 = 24;

/// `view_status` values: showing a map message, and showing the normal view again.
const VIEW_MESSAGE: i8 = 2;
const VIEW_NORMAL_AGAIN: i8 = 3;

/// Shows the page without scrolling.
pub fn show_still_screen(game: &mut Game) {
    let ras = game.screen.drawing_page().ras;
    game.screen.ras[ras].x = 0;
    game.screen.ras[ras].y = 0;

    flip_pages(game);
}

/// Switches to the map page for a message (the text panel is hidden) and fades in.
pub fn map_message(game: &mut Game) {
    fade_down(game);

    game.screen.current_port = PortId::Map;
    game.screen.port_mut(PortId::Map).target = Some(game.screen.drawing_bitmap_id());
    game.screen.set_draw_mode(PortId::Map, JAM1);
    game.screen.set_a_pen(PortId::Map, MAP_PEN);
    game.screen.clear(PortId::Map, 0);

    game.screen.vp_text.modes = HIRES | SPRITES | VP_HIDE;
    show_still_screen(game);
    game.screen
        .load_colors(ViewPortId::Page, unsafe { &*(&raw const page_colors) });

    game.screen.view_status = VIEW_MESSAGE;
}

/// Goes back from the map message to the normal view.
pub fn message_off(game: &mut Game) {
    fade_down(game);

    game.screen.current_port = PortId::Text;
    game.screen.vp_text.modes = HIRES | SPRITES;
    flip_pages(game);
    game.screen.view_status = VIEW_NORMAL_AGAIN;
}
