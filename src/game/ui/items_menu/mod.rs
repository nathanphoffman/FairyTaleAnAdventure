//! The Items menu actions (from `choose_menu_entry` in csrc/fmain.c): showing the item list, taking what
//! is nearby (objects, containers, dead bodies) and looking around.

mod containers;

mod item_list;

mod look;

mod search_body;

mod take;

use self::item_list::show_item_list;
use self::look::look_around;
use self::take::take_nearest;
use crate::game::data::state::Game;

use crate::game::data::tables::item_list;
use crate::game::data::types::InvItem;
use crate::game::ui::menus::go_to_menu;

// Menu numbers and the entries of the Items menu.
const USE: i16 = 8;
const GIVE: i16 = 7;
const LIST: i16 = 5;
const TAKE: i16 = 6;
const LOOK: i16 = 7;
const USE_ENTRY: i16 = 8;
const GIVE_ENTRY: i16 = 9;

// Actor kinds (the sequence numbers in csrc/ftale.h).
const OBJECTS: i8 = 1;

// Inventory slots.
const GOLDBASE: usize = 31;
const ARROWBASE: usize = 35;
const KEYBASE: i32 = 16;
const ARROWS: usize = 8;
const FRUIT_SLOT: usize = 24;
const TALISMAN: usize = 22;
const URN: u32 = 0x0e;
const CHEST: u32 = 0x0f;
const SACKS: u32 = 0x10;
const PRQ_VITALITY: i32 = 7;

/// The name of inventory item `i`.
pub(in crate::game::ui) fn item_name(i: usize) -> &'static str {
    item(i).name
}

fn item(i: usize) -> &'static InvItem {
    &item_list[i]
}

/// Adds `amount` to the hero's count of inventory slot `slot`.
pub(in crate::game::ui) fn add_stuff(game: &mut Game, slot: usize, amount: u8) {
    game.hero.stuff_mut()[slot] = (game.hero.stuff()[slot]).wrapping_add(amount);
}

/// Handles an entry of the Items menu.
pub fn items_option(game: &mut Game, chosen_entry: i16) {
    match chosen_entry {
        USE_ENTRY => go_to_menu(game, USE),
        LIST => show_item_list(game),
        TAKE => take_nearest(game),
        LOOK => look_around(game),
        GIVE_ENTRY => go_to_menu(game, GIVE),
        _ => {}
    }
}
