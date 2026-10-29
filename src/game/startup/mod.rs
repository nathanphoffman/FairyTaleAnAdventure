//! Starting and stopping the game (from csrc/fmain.c): `open_all` sets up the screen, memory, fonts,
//! sound and input; `close_all` gives it all back; and `run_game` (the original `main`) shows the
//! title and intro, starts the game and runs the main loop, one pass per frame.

pub mod game_loop;

pub mod intro;

pub mod open_close;

pub mod story_pages;

use crate::game::data::state::{Game, Video};
use game_loop::main_loop;
use intro::start_up;
use open_close::{close_all, open_all};

use crate::game::ui::text::show_error;

use crate::platform::audio::stopscore;

/// The game: the original `main`. Called once from `src/main.rs`.
pub fn run_game() {
    let mut game = Game::new();
    let mut video = Video::new();
    game.world.light_timer = 0;
    let err = open_all(&mut game, &mut video);

    if err != 0 {
        show_error(&mut video, err);
        quit_all();

        return;
    }

    if start_up(&mut game, &mut video) {
        main_loop(&mut game, &mut video);
        stopscore();
    }

    quit_all();
}

/// The end of the program (the `quit_all` label).
fn quit_all() {
    close_all();
}
