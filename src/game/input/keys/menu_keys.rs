//! Keys and clicks on the menu entries: choosing an entry with the mouse or with its letter.

use super::{
    CODE_MASK, DISPLAYED, FIRST_ACTION_MENU, FIRST_MENU_CLICK, LETTER_COUNT, PRQ_OPTIONS, RELEASED,
    SAVEX, SELECTED, TYPE_IMMEDIATE, TYPE_MASK, TYPE_RADIO, TYPE_TOGGLE,
};
use crate::game::data::state::Game;

use crate::game::data::tables::menu_shortcuts;
use crate::game::input::handler::forget_last_menu;
use crate::game::ui::menus::{print_menu_entry, print_options};
use crate::game::ui::options::choose_menu_entry;
use crate::game::ui::text::queue_print;

/// A click on a menu slot (the mouse sends 'a' + slot for the button going down, and the same
/// with bit 7 set for it coming up).
pub(super) fn menu_click(game: &mut Game, key: i8, notpause: bool) {
    let slot = ((key & CODE_MASK) - FIRST_MENU_CLICK) as i16;
    game.ui.chosen_entry = game.ui.slot_entries[slot as usize];
    let chosen_entry = game.ui.chosen_entry as i16;
    let menu = game.ui.current_menu as usize;
    let valid = chosen_entry >= 0 && chosen_entry < game.menus[menu].num as i16;

    if key & RELEASED != 0 {
        if valid {
            print_menu_entry(game, slot, 0);
        }

        return;
    }

    game.ui.menu_hit_enabled = 1;

    if !valid {
        return;
    }

    press_entry(game, slot, chosen_entry, notpause);
}

/// Presses a valid menu entry according to its type: toggles, immediate actions and radio buttons
/// work even when paused; the rest (menu names) only when not.
fn press_entry(game: &mut Game, slot: i16, chosen_entry: i16, notpause: bool) {
    let menu = game.ui.current_menu as usize;
    let entry = chosen_entry as usize;
    let kind = game.menus[menu].enabled[entry] as i32 & TYPE_MASK;

    if kind != TYPE_IMMEDIATE {
        forget_last_menu();
    }

    if kind == TYPE_TOGGLE {
        game.menus[menu].enabled[entry] ^= SELECTED;
        print_menu_entry(
            game,
            slot,
            (game.menus[menu].enabled[entry] & SELECTED) as i16,
        );
        choose_menu_entry(game, chosen_entry);
    } else if kind == TYPE_IMMEDIATE {
        print_menu_entry(game, slot, 1);
        choose_menu_entry(game, chosen_entry);
    } else if kind == TYPE_RADIO {
        game.menus[menu].enabled[entry] |= SELECTED;
        print_menu_entry(game, slot, 1);
        choose_menu_entry(game, chosen_entry);
    } else if !notpause {
        // paused: only the menu types above work
    } else if kind == 0 && chosen_entry < FIRST_ACTION_MENU {
        game.ui.current_menu = chosen_entry;
        queue_print(&mut game.ui, PRQ_OPTIONS);
    } else {
        print_menu_entry(
            game,
            slot,
            (game.menus[menu].enabled[entry] & SELECTED) as i16,
        );
    }
}

/// A keyboard shortcut for a menu entry. Letters only work for the Save/Exit menu while it is open.
pub(super) fn menu_letter(game: &mut Game, key: i8) {
    for shortcut in menu_shortcuts.iter().take(LETTER_COUNT) {
        if shortcut.letter != key {
            continue;
        }

        if shortcut.menu as i16 == SAVEX && game.ui.current_menu != SAVEX {
            break;
        }

        game.ui.current_menu = shortcut.menu as i16;
        game.ui.chosen_entry = shortcut.choice;
        let entry =
            &mut game.menus[game.ui.current_menu as usize].enabled[game.ui.chosen_entry as usize];

        game.ui.menu_hit_enabled = *entry & DISPLAYED;

        if *entry as i32 & TYPE_MASK == TYPE_TOGGLE {
            *entry ^= SELECTED;
        }

        choose_menu_entry(game, game.ui.chosen_entry as i16);
        print_options(game);

        break;
    }
}
