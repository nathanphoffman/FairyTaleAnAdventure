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

use crate::amiga::{BitMap, RastPort, TextFont, ViewPort};
use crate::game::data::state::the_hero;
use crate::game::data::tables::item_list;
use crate::game::data::types::InvItem;
use crate::game::ui::menus::go_to_menu;
use std::ffi::c_char;

unsafe extern "C" {
    fn SetRast(current_rastport: *mut RastPort, pen: u32);

    fn InitBitMap(bm: *mut BitMap, depth: i32, width: i32, height: i32);

    fn SetRGB4(current_viewport: *mut ViewPort, n: i32, r: u32, g: u32, b: u32);

    fn LoadRGB4(current_viewport: *mut ViewPort, colors: *const u16, count: i32);

    fn SetFont(current_rastport: *mut RastPort, font: *mut TextFont);

    fn BltBitMap(
        src: *mut BitMap,
        sx: i32,
        sy: i32,
        dst: *mut BitMap,
        dx: i32,
        dy: i32,
        w: i32,
        h: i32,
        minterm: u32,
        mask: u32,
        temp: *mut u8,
    ) -> i32;
}

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
pub(in crate::game::ui) unsafe fn item_name(i: usize) -> *const c_char {
    unsafe { (*(&raw const item_list).cast::<InvItem>().add(i)).name }
}

unsafe fn item(i: usize) -> &'static InvItem {
    unsafe { &*(&raw const item_list).cast::<InvItem>().add(i) }
}

/// Adds `amount` to the hero's count of inventory slot `slot`.
pub(in crate::game::ui) unsafe fn add_stuff(slot: usize, amount: u8) {
    unsafe { *the_hero.stuff.add(slot) = (*the_hero.stuff.add(slot)).wrapping_add(amount) };
}

/// Handles an entry of the Items menu.
pub unsafe fn items_option(chosen_entry: i16) {
    unsafe {
        match chosen_entry {
            USE_ENTRY => go_to_menu(USE),
            LIST => show_item_list(),
            TAKE => take_nearest(),
            LOOK => look_around(),
            GIVE_ENTRY => go_to_menu(GIVE),
            _ => {}
        }
    }
}
