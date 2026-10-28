//! What choosing an entry of a menu does (`choose_menu_entry` from csrc/fmain.c): the Items and Magic
//! menus (in `items_menu` and `magic`), and here talking, buying, the game menu, using items,
//! keys, saving and giving.

mod game_menu;

mod give;

mod shop;

mod speeches;

mod talk;

mod use_item;

use crate::game::data::state::Game;
use self::game_menu::{file_option, game_option, save_exit_option};
use self::give::give_option;
use self::shop::buy_option;
use self::talk::talk_option;
use self::use_item::{keys_option, use_option};

use crate::game::data::state::the_ui;
use crate::game::ui::items_menu::items_option;
use crate::game::ui::magic::magic_option;
use crate::game::ui::menus::update_menu_availability;

// Menu numbers (`enum cmodes` in csrc/fmain.c).
const ITEMS: i32 = 0;
const MAGIC: i32 = 1;
const TALK: i32 = 2;
const BUY: i32 = 3;
const GAME: i32 = 4;
const SAVEX: i32 = 5;
const KEYS: i32 = 6;
const GIVE: i32 = 7;
const USE: i32 = 8;
const FILE: i32 = 9;

// Inventory slots.
const KEYBASE: usize = 16;
const ARROWS: usize = 8;
const BONE: usize = 29;

// Print queue entries.
const PRQ_STATS: i32 = 4;
const PRQ_VITALITY: i32 = 7;

/// Does what menu entry `chosen_entry` of the current menu `current_menu` says.
pub fn choose_menu_entry(game: &mut Game, chosen_entry: i16) {
    unsafe {
        let refresh = match the_ui.current_menu as i32 {
            ITEMS => {
                items_option(chosen_entry);

                true
            }

            MAGIC => magic_option(chosen_entry),
            TALK => talk_option(chosen_entry),
            BUY => buy_option(chosen_entry),
            GAME => game_option(chosen_entry),
            USE => use_option(game, chosen_entry),
            SAVEX => save_exit_option(chosen_entry),
            FILE => file_option(chosen_entry),
            KEYS => keys_option(chosen_entry),
            GIVE => give_option(chosen_entry),
            _ => true,
        };

        if refresh {
            update_menu_availability();
        }
    }
}
