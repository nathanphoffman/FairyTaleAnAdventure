//! Starting and stopping the game (from csrc/fmain.c): `open_all` sets up the screen, memory, fonts,
//! sound and input; `close_all` gives it all back; and `run_game` (the original `main`) shows the
//! title and intro, starts the game and runs the main loop, one pass per frame.

pub mod amiga_leftovers;

pub mod game_loop;

pub mod intro;

pub mod open_close;

use game_loop::main_loop;
use intro::start_up;
use open_close::{close_all, open_all};

use crate::amiga::RastPort;

use crate::game::data::state::{the_screen, the_world};
use crate::game::ui::text::show_error;

unsafe extern "C" {

    // AmigaOS calls the platform layer provides.
    fn SetRast(current_rastport: *mut RastPort, pen: u32);

    // The music player (in the platform layer).
    fn stopscore();
}

/// The game: the original `main`. Called once from `src/main.rs`.
pub fn run_game() {
    unsafe {
        the_world.light_timer = 0;
        let err = open_all();

        if err != 0 {
            show_error(err);
            quit_all();

            return;
        }

        if start_up() {
            main_loop();
            stopscore();
        }

        quit_all();
    }
}

/// The end of the program (the `quit_all` label): back to the plain text screen, then `close_all`.
unsafe fn quit_all() {
    unsafe {
        the_screen.rp_text.BitMap = the_screen.wb_bmap;
        SetRast(&raw mut the_screen.rp_text, 0);
        close_all();
    }
}
