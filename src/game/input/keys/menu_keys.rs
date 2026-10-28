//! Keys and clicks on the menu entries: choosing an entry with the mouse or with its letter.

use super::{
    CODE_MASK, DISPLAYED, FIRST_ACTION_MENU, FIRST_MENU_CLICK, LETTER_COUNT, PRQ_OPTIONS, RELEASED,
    SAVEX, SELECTED, TYPE_IMMEDIATE, TYPE_MASK, TYPE_RADIO, TYPE_TOGGLE,
};

use crate::game::data::state::the_ui;
use crate::game::data::tables::{menu_entry, menu_shortcuts, menus};
use crate::game::data::types::Letter;
use crate::game::input::handler::forget_last_menu;
use crate::game::ui::menus::{print_menu_entry, print_options};
use crate::game::ui::options::choose_menu_entry;
use crate::game::ui::text::queue_print;

/// A click on a menu slot (the mouse sends 'a' + slot for the button going down, and the same
/// with bit 7 set for it coming up).
pub(super) unsafe fn menu_click(key: i8, notpause: bool) {
    unsafe {
        // Masks with CODE_MASK, keeping only the bits that matter here.
        let slot = ((key & CODE_MASK) - FIRST_MENU_CLICK) as i16;
        the_ui.chosen_entry = the_ui.slot_entries[slot as usize];
        let chosen_entry = the_ui.chosen_entry as i16;
        let menu = the_ui.current_menu as usize;
        let valid = chosen_entry >= 0 && chosen_entry < menus[menu].num as i16;

        // Masks with RELEASED, keeping only the bits that matter here.
        if key & RELEASED != 0 {
            if valid {
                print_menu_entry(slot, 0);
            }

            return;
        }

        the_ui.menu_hit_enabled = 1;

        if !valid {
            return;
        }

        press_entry(
            slot,
            chosen_entry,
            menu_entry(menu, chosen_entry as usize),
            notpause,
        );
    }
}

/// Presses a valid menu entry according to its type: toggles, immediate actions and radio buttons
/// work even when paused; the rest (menu names) only when not.
unsafe fn press_entry(slot: i16, chosen_entry: i16, entry: &mut i8, notpause: bool) {
    unsafe {
        // Masks with TYPE_MASK, keeping only the bits that matter here.
        let kind = *entry as i32 & TYPE_MASK;

        if kind != TYPE_IMMEDIATE {
            forget_last_menu();
        }

        if kind == TYPE_TOGGLE {
            // Xor flips the selected bit: a toggle entry goes from on to off or back.
            *entry ^= SELECTED;
            // Masks with SELECTED, keeping only the bits that matter here.
            print_menu_entry(slot, (*entry & SELECTED) as i16);
            choose_menu_entry(chosen_entry);
        } else if kind == TYPE_IMMEDIATE {
            print_menu_entry(slot, 1);
            choose_menu_entry(chosen_entry);
        } else if kind == TYPE_RADIO {
            *entry |= SELECTED;
            print_menu_entry(slot, 1);
            choose_menu_entry(chosen_entry);
        } else if !notpause {
            // paused: only the menu types above work
        } else if kind == 0 && chosen_entry < FIRST_ACTION_MENU {
            the_ui.current_menu = chosen_entry;
            queue_print(PRQ_OPTIONS);
        } else {
            // Masks with SELECTED, keeping only the bits that matter here.
            print_menu_entry(slot, (*entry & SELECTED) as i16);
        }
    }
}

/// A keyboard shortcut for a menu entry. Letters only work for the Save/Exit menu while it is open.
pub(super) unsafe fn menu_letter(key: i8) {
    unsafe {
        for i in 0..LETTER_COUNT {
            let shortcut = &*(&raw const menu_shortcuts).cast::<Letter>().add(i);

            if shortcut.letter != key {
                continue;
            }

            if shortcut.menu as i16 == SAVEX && the_ui.current_menu != SAVEX {
                break;
            }

            the_ui.current_menu = shortcut.menu as i16;
            the_ui.chosen_entry = shortcut.choice;
            let entry =
                &raw mut menus[the_ui.current_menu as usize].enabled[the_ui.chosen_entry as usize];

            // Masks with DISPLAYED, keeping only the bits that matter here.
            the_ui.menu_hit_enabled = *entry & DISPLAYED;

            // Masks with TYPE_MASK, keeping only the bits that matter here.
            if *entry as i32 & TYPE_MASK == TYPE_TOGGLE {
                // Xor flips the selected bit: a toggle entry goes from on to off or back.
                *entry ^= SELECTED;
            }

            choose_menu_entry(the_ui.chosen_entry as i16);
            print_options();

            break;
        }
    }
}
