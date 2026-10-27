//! The Game and Save/Exit menus and loading: music on and off, saving, quitting and loading.

use super::{FILE, GAME, SAVEX};
use crate::game::data::state::the_ui;
use crate::game::files::savegame::{is_saving, savegame};
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
pub(super) unsafe fn game_option(chosen_entry: i16) -> bool {
    unsafe {
        if chosen_entry == MUSIC {
            set_mood(1);
        }

        if chosen_entry == QUIT {
            go_to_menu(SAVEX as i16);
        }

        if chosen_entry == LOAD {
            is_saving = 0;
            go_to_menu(FILE as i16);
        }

        true
    }
}

/// Saving goes to the file menu; exiting ends the game.
pub(super) unsafe fn save_exit_option(chosen_entry: i16) -> bool {
    unsafe {
        if chosen_entry == EXIT {
            the_ui.quit_requested = 1;
        }

        if chosen_entry == SAVE {
            is_saving = 1;
            go_to_menu(FILE as i16);
        }

        true
    }
}

/// A save slot was chosen: save or load, then back to the game menu.
pub(super) fn file_option(chosen_entry: i16) -> bool {
    savegame(chosen_entry);
    go_to_menu(GAME as i16);

    true
}
