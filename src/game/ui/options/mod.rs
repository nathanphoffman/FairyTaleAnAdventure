//! What choosing an entry of a menu does (`choose_menu_entry` from csrc/fmain.c): the Items and Magic
//! menus (in `items_menu` and `magic`), and here talking, buying, the game menu, using items,
//! keys, saving and giving.

mod game_menu;
mod give;
mod shop;
mod speeches;
mod talk;
mod use_item;

use self::game_menu::*;
use self::give::*;
use self::shop::*;
use self::speeches::*;
use self::talk::*;
use self::use_item::*;

use crate::game::actors::interaction::{eat_food, find_nearest_figure, summon_turtle};
use crate::game::data::object_tables::{jtrans, ob_list8, ob_listg};
use crate::game::data::random::rand64;
use crate::game::data::state::{the_actors, the_hero, the_ui, the_witch, the_world};
use crate::game::data::tables::{item_list, setfig_table};
use crate::game::data::types::{InvItem, Shape};
use crate::game::files::savegame::{is_saving, savegame};
use crate::game::ui::items_menu::items_option;
use crate::game::ui::magic::magic_option;
use crate::game::ui::menus::go_to_menu;
use crate::game::ui::menus::update_menu_availability;
use crate::game::ui::messages::{print, print_cont, print_wrapped};
use crate::game::ui::music::set_mood;
use crate::game::ui::text::{queue_print, show_event, speak};
use crate::game::world::key_doors::{bumped_locked_door, open_door_near};
use crate::game::world::map::{step_x, step_y};
use crate::game::world::objects::{SetFigEntry, leave_item};

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
pub fn choose_menu_entry(chosen_entry: i16) {
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
            USE => use_option(chosen_entry),
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

unsafe fn add_stuff(slot: usize, amount: u8) {
    unsafe { *the_hero.stuff.add(slot) = (*the_hero.stuff.add(slot)).wrapping_add(amount) };
}

unsafe fn item_name(i: usize) -> *const std::ffi::c_char {
    unsafe { (*(&raw const item_list).cast::<InvItem>().add(i)).name }
}
