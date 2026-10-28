//! Showing the finished page (from csrc/fmain.c).
//!
//! The game draws on one of two pages while the other is on screen (`drawing`, `viewing`), then
//! swaps them.

use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::input::handler::reset_key_buffer;
use crate::game::statedump::dump_state;
use crate::platform::display;

/// Used when FTA_STATE_AFTER is not set.
const DEFAULT_AFTER_TICKS: u32 = 1;

/// Empties the key buffer (the hero is being revived, so old key presses are dropped).
pub fn reset_input() {
    reset_key_buffer();
}

/// FTA_STATE=file writes the game state as text once FTA_STATE_AFTER ticks have passed (then quits).
/// A test helper; the display only counts the ticks.
fn dump_state_if_due(game: &mut Game) {
    let Some(path) = std::env::var_os("FTA_STATE") else {
        return;
    };

    let after = std::env::var("FTA_STATE_AFTER")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_AFTER_TICKS);

    if display::ticks() < after {
        return;
    }

    dump_state(game, std::path::Path::new(&path));
    std::process::exit(0);
}

/// Waits one tick of 1/50 second, showing the screen as it is now.
fn wait_frame(game: &mut Game, video: &mut Video) {
    dump_state_if_due(game);
    display::wait_frame(&video.screen.scene());
}

/// Waits `ticks` ticks, showing the screen as it is now.
pub fn delay(game: &mut Game, video: &mut Video, ticks: i32) {
    for _ in 0..ticks.max(1) {
        wait_frame(game, video);
    }
}

/// Swaps the page being drawn with the one being shown, and shows the one just finished.
pub fn flip_pages(game: &mut Game, video: &mut Video) {
    let finished = video.screen.drawing;
    std::mem::swap(&mut video.screen.drawing, &mut video.screen.viewing);
    video.screen.vp_page.ras = Some(video.screen.pages[finished].ras);

    dump_state_if_due(game);
    display::present(&video.screen.scene());
    wait_frame(game, video);
}
