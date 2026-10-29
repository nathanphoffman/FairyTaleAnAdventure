//! Showing the list of the things the hero carries, with their pictures.

use crate::game::data::state::Game;
use crate::game::output::Output;
use crate::game::render::screen_fx::show_still_screen;
use crate::game::ui::text::queue_print;

// Print queue entries and events.
const PRQ_LIST: i32 = 5;

/// `view_status` value while the item list is shown.
const VIEW_ITEMS: i8 = 4;
pub(super) const VIEW_GAME_OVER: i8 = 2;

/// Draws every item the hero has, stacked, on a black page and shows it.
pub(super) fn show_item_list(game: &mut Game, out: &mut dyn Output) {
    out.draw_item_list(&mut game.hero);
    show_still_screen(game, out);
    out.load_page_colors(&game.page_colors);

    game.ui.view_status = VIEW_ITEMS;
    queue_print(&mut game.ui, PRQ_LIST);
}
