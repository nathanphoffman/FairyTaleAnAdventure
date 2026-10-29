//! Switching between the play view and the map page that shows messages.

use super::fading::fade_down;
use crate::game::data::state::Game;
use crate::game::output::Output;
use crate::game::render::display::flip_pages;
use crate::game::render::screen::PortId;

const MAP_PEN: u8 = 24;

/// `view_status` values: showing a map message, and showing the normal view again.
const VIEW_MESSAGE: i8 = 2;
const VIEW_NORMAL_AGAIN: i8 = 3;

/// Shows the page without scrolling.
pub fn show_still_screen(game: &mut Game, out: &mut dyn Output) {
    out.reset_page_scroll();
    flip_pages(game, out);
}

/// Switches to the map page for a message (the text panel is hidden) and fades in.
pub fn map_message(game: &mut Game, out: &mut dyn Output) {
    fade_down(game, out);

    out.prepare_message_page(MAP_PEN);
    out.set_panel_visible(false);
    show_still_screen(game, out);
    out.load_page_colors(&game.page_colors);

    game.ui.view_status = VIEW_MESSAGE;
}

/// Goes back from the map message to the normal view.
pub fn message_off(game: &mut Game, out: &mut dyn Output) {
    fade_down(game, out);

    out.select_port(PortId::Text);
    out.set_panel_visible(true);
    flip_pages(game, out);
    game.ui.view_status = VIEW_NORMAL_AGAIN;
}
