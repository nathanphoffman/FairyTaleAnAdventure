//! Switching between the play view and the map page that shows messages.

use super::fading::fade_down;
use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::output::Output;
use crate::game::render::display::flip_pages;
use crate::game::render::screen::{HIRES, PortId, SPRITES, VP_HIDE};
use crate::gfx::draw::JAM1;

const MAP_PEN: u8 = 24;

/// `view_status` values: showing a map message, and showing the normal view again.
const VIEW_MESSAGE: i8 = 2;
const VIEW_NORMAL_AGAIN: i8 = 3;

/// Shows the page without scrolling.
pub fn show_still_screen(game: &mut Game, video: &mut Video) {
    let ras = video.screen.drawing_page().ras;
    video.screen.ras[ras].x = 0;
    video.screen.ras[ras].y = 0;

    flip_pages(game, video);
}

/// Switches to the map page for a message (the text panel is hidden) and fades in.
pub fn map_message(game: &mut Game, video: &mut Video) {
    fade_down(game, video);

    video.screen.current_port = PortId::Map;
    video.screen.port_mut(PortId::Map).target = Some(video.screen.drawing_bitmap_id());
    video.screen.set_draw_mode(PortId::Map, JAM1);
    video.screen.set_a_pen(PortId::Map, MAP_PEN);
    video.screen.clear(PortId::Map, 0);

    video.screen.vp_text.modes = HIRES | SPRITES | VP_HIDE;
    show_still_screen(game, video);
    video.load_page_colors(&game.page_colors);

    game.ui.view_status = VIEW_MESSAGE;
}

/// Goes back from the map message to the normal view.
pub fn message_off(game: &mut Game, video: &mut Video) {
    fade_down(game, video);

    video.screen.current_port = PortId::Text;
    video.screen.vp_text.modes = HIRES | SPRITES;
    flip_pages(game, video);
    game.ui.view_status = VIEW_NORMAL_AGAIN;
}
