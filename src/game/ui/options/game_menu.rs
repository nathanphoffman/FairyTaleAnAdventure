//! The Game and Save/Exit menus and loading: music on and off, saving, quitting and loading.

use super::{FILE, GAME, SAVEX};
use crate::game::data::state::Game;
use crate::game::files::savegame::savegame;
use crate::game::ui::menus::go_to_menu;
use crate::game::ui::music::set_mood;

// Game menu entries.
const MUSIC: i16 = 6;
const QUIT: i16 = 8;
const LOAD: i16 = 9;

// Save/exit menu entries.
const SAVE: i16 = 5;
const EXIT: i16 = 6;

/// Music on/off, and going to the save/exit and load menus.
pub(super) fn game_option(game: &mut Game, chosen_entry: i16) -> bool {
    if chosen_entry == MUSIC {
        set_mood(game, 1);
    }

    if chosen_entry == QUIT {
        go_to_menu(game, SAVEX as i16);
    }

    if chosen_entry == LOAD {
        game.ui.saving = false;
        go_to_menu(game, FILE as i16);
    }

    true
}

/// Saving goes to the file menu; exiting ends the game.
pub(super) fn save_exit_option(game: &mut Game, chosen_entry: i16) -> bool {
    if chosen_entry == EXIT {
        game.ui.quit_requested = 1;
    }

    if chosen_entry == SAVE {
        game.ui.saving = true;
        go_to_menu(game, FILE as i16);
    }

    true
}

/// A save slot was chosen: save or load, then back to the game menu.
pub(super) fn file_option(game: &mut Game, chosen_entry: i16) -> bool {
    savegame(game, chosen_entry, game.ui.saving);
    go_to_menu(game, GAME as i16);

    true
}
