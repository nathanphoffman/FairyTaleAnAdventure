//! Showing the finished page (from csrc/fmain.c).
//!
//! The game draws on one of two pages while the other is on screen (`drawing`, `viewing`), then
//! swaps them.

use crate::game::data::state::the_screen;
use crate::game::input::handler::reset_key_buffer;
use crate::platform::display;

/// Empties the key buffer (the hero is being revived, so old key presses are dropped).
pub fn reset_input() {
    reset_key_buffer();
}

/// Waits `ticks` ticks, showing the screen as it is now.
pub fn delay(ticks: i32) {
    unsafe { display::delay(&(*(&raw const the_screen)).scene(), ticks) };
}

/// Swaps the page being drawn with the one being shown, and shows the one just finished.
pub fn flip_pages() {
    unsafe {
        let screen = &mut *(&raw mut the_screen);
        let finished = screen.drawing;
        std::mem::swap(&mut screen.drawing, &mut screen.viewing);
        screen.vp_page.ras = Some(screen.pages[finished].ras);

        display::present(&screen.scene());
        display::wait_frame(&screen.scene());
    }
}
