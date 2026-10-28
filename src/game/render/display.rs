//! Showing the finished page (from csrc/fmain.c).
//!
//! The game draws on one of two pages while the other is on screen (`drawing`, `viewing`), then
//! swaps them.

use crate::game::data::state::Game;
use crate::game::input::handler::reset_key_buffer;
use crate::platform::display;

/// Empties the key buffer (the hero is being revived, so old key presses are dropped).
pub fn reset_input() {
    reset_key_buffer();
}

/// Waits `ticks` ticks, showing the screen as it is now.
pub fn delay(game: &mut Game, ticks: i32) {
    display::delay(game, ticks);
}

/// Swaps the page being drawn with the one being shown, and shows the one just finished.
pub fn flip_pages(game: &mut Game) {
    let finished = game.screen.drawing;
    std::mem::swap(&mut game.screen.drawing, &mut game.screen.viewing);
    game.screen.vp_page.ras = Some(game.screen.pages[finished].ras);

    display::present(game);
    display::wait_frame(game);
}
