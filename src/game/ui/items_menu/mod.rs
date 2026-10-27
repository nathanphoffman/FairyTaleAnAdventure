//! The Items menu actions (from `choose_menu_entry` in csrc/fmain.c): showing the item list, taking what
//! is nearby (objects, containers, dead bodies) and looking around.

mod containers;
mod item_list;
mod look;
mod search_body;
mod take;

use self::containers::*;
use self::item_list::*;
use self::look::*;
use self::search_body::*;
use self::take::*;

use crate::amiga::{BitMap, RastPort, TextFont, ViewPort};
use crate::game::actors::encounters::Encounter;
use crate::game::actors::interaction::{actor_distance, eat_food, find_nearest_figure};
use crate::game::data::globals::story_font;
use crate::game::data::object_tables::{itrans, ob_listg, page_colors, treasure_probs};
use crate::game::data::random::{rand4, rand8};
use crate::game::data::state::{the_actors, the_hero, the_memory, the_screen, the_ui, the_world};
use crate::game::data::tables::{encounter_chart, item_list};
use crate::game::data::types::{InvItem, SeqInfo, Shape};
use crate::game::render::screen_fx::{map_message, show_still_screen, win_colors};
use crate::game::ui::menus::go_to_menu;
use crate::game::ui::messages::{
    announce_container, announce_treasure, print, print_cont, print_wrapped,
};
use crate::game::ui::text::{print_number, queue_print, show_event};
use crate::game::world::objects::change_object;
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
unsafe fn item_name(i: usize) -> *const c_char {
    unsafe { (*(&raw const item_list).cast::<InvItem>().add(i)).name }
}

unsafe fn item(i: usize) -> &'static InvItem {
    unsafe { &*(&raw const item_list).cast::<InvItem>().add(i) }
}

unsafe fn add_stuff(slot: usize, amount: u8) {
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
