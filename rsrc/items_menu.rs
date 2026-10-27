//! The Items menu actions (from `choose_menu_entry` in csrc/fmain.c): showing the item list, taking what
//! is nearby (objects, containers, dead bodies) and looking around.

use super::actors::{actor_distance, eat_food, find_nearest_figure};
use super::asm_text::{print_number, queue_print, show_event};
use super::encounters::Encounter;
use super::game_tables::{encounter_chart, item_list};
use super::globals::{
    current_rastport, drawing_page, julstuff, pagea, philstuff, quit_requested, rp_map, seq_list,
    story_font, view_status, vp_page,
};
use super::menus::go_to_menu;
use super::messages::{announce_container, announce_treasure, print, print_cont, print_wrapped};
use super::objects::change_object;
use super::random::{rand4, rand8};
use super::screen_fx::{map_message, show_still_screen, win_colors};
use super::state::{the_actors, the_hero, the_world};
use super::tables::{itrans, ob_listg, page_colors, treasure_probs};
use super::types::{InvItem, SeqInfo, Shape};
use crate::amiga::{BitMap, RastPort, TextFont, ViewPort};
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
/// The 100 gold pieces entry of `item_list`.
const HUNDRED_GOLD: i32 = GOLDBASE as i32 + 3;

// Object ids of things that are not plain pickups.
const GOLD_PILE: u32 = 0x0d;
const URN: u32 = 0x0e;
const CHEST: u32 = 0x0f;
const SACKS: u32 = 0x10;
const SCRAP_OF_PAPER: u32 = 0x14;
const EMPTY_CHEST: u32 = 0x1d;
const BROTHER_BONES: u32 = 28;
const FOOTSTOOL: u32 = 31;
const TURTLE_EGGS: u32 = 102;
const FRUIT: u32 = 148;
/// Object list slots (in `ob_listg`) of the two brothers' bones.
const BONES: (usize, usize) = (3, 4);

/// Hunger below this and the fruit is kept for later instead of being eaten.
const NOT_HUNGRY: i16 = 15;
const FRUIT_HUNGER: i32 = 30;
const GOLD_FOUND: i16 = 50;
const CONTAINER_GOLD: i16 = 100;
/// Search range for the nearest thing to take.
const TAKE_RANGE: i16 = 30;
/// `Look` finds hidden objects closer than this.
const LOOK_RANGE: i32 = 40;

// Print queue entries and events.
const PRQ_LIST: i32 = 5;
const PRQ_VITALITY: i32 = 7;
const PRQ_NOTHING_NEAR: i32 = 10;
const EVENT_PAPER: i32 = 17;
const EVENT_PAPER_OUTDOORS: i32 = 18;
const EVENT_PAPER_INDOORS: i32 = 19;
const EVENT_NOTHING_HIDDEN: i32 = 20;
const EVENT_FOUND_HIDDEN: i32 = 38;
const EVENT_ALIVE: i32 = 35;
const EVENT_FRUIT_KEPT: i32 = 36;
/// Regions numbered above this are indoors.
const FIRST_INDOOR_REGION: u16 = 7;

// Drawing the item list screen.
const ITEM_IMAGE_WIDTH: i32 = 16;
const ITEM_IMAGE_BYTES: usize = 32;
const ITEM_IMAGE_PLANES: i32 = 5;
const ITEM_IMAGE_ROWS: i32 = 8000;
const ITEM_ROW_HEIGHT: i32 = 80;
const ITEM_X_MARGIN: i32 = 20;
const BLIT_COPY: u32 = 0xC0;
const ALL_PLANES: u32 = 0xff;
const BACKGROUND_PEN: u32 = 31;
/// `view_status` value while the item list is shown.
const VIEW_ITEMS: i8 = 4;
const VIEW_GAME_OVER: i8 = 2;
const COLOURS_SHOWN: i32 = 31;

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

/// Draws every item the hero has, stacked, on a black page and shows it.
unsafe fn show_item_list() {
    unsafe {
        let bm = (*(*drawing_page).ri_page).BitMap;
        rp_map.BitMap = bm;
        SetRast(&raw mut rp_map, 0);

        InitBitMap(
            &raw mut pagea,
            ITEM_IMAGE_PLANES,
            ITEM_IMAGE_WIDTH,
            ITEM_IMAGE_ROWS,
        );
        let data = (*(&raw mut seq_list).cast::<SeqInfo>().add(OBJECTS as usize)).location;
        for plane in 0..ITEM_IMAGE_PLANES as usize {
            pagea.Planes[plane] = data.add(plane * ITEM_IMAGE_BYTES);
        }

        draw_item_icons(bm);

        SetRGB4(&raw mut vp_page, BACKGROUND_PEN as i32, 0, 0, 0);
        show_still_screen();
        LoadRGB4(
            &raw mut vp_page,
            (&raw const page_colors).cast(),
            COLOURS_SHOWN,
        );
        view_status = VIEW_ITEMS;
        queue_print(PRQ_LIST);
    }
}

/// Draws one small picture for each item the hero holds (up to the most the screen shows),
/// stacked down the item's column. The pictures are read from `pagea`.
unsafe fn draw_item_icons(bm: *mut BitMap) {
    unsafe {
        for slot in 0..GOLDBASE {
            let it = item(slot);
            let shown = (*the_hero.stuff.add(slot)).min(it.maxshown);
            let x = it.xoff as i32 + ITEM_X_MARGIN;
            let mut y = it.yoff as u16;
            let source_row = it.image_number as i32 * ITEM_ROW_HEIGHT + it.img_off as i32;
            for _ in 0..shown {
                BltBitMap(
                    &raw mut pagea,
                    0,
                    source_row,
                    bm,
                    x,
                    y as i32,
                    ITEM_IMAGE_WIDTH,
                    it.img_height as i32,
                    BLIT_COPY,
                    ALL_PLANES,
                    std::ptr::null_mut(),
                );
                y = y.wrapping_add(it.ydelta as u16);
            }
        }
    }
}

/// Takes the nearest object, or searches the nearest dead body.
unsafe fn take_nearest() {
    unsafe {
        queue_print(PRQ_VITALITY);
        find_nearest_figure(0, TAKE_RANGE);
        *the_hero.stuff.add(ARROWBASE) = 0;
        if the_actors.nearest_object == 0 {
            queue_print(PRQ_NOTHING_NEAR);
            return;
        }
        let an = &raw mut the_actors.list[the_actors.nearest_object as usize];
        if (*an).kind == OBJECTS {
            if take_object() {
                finish_pickup();
            }
        } else if (*an).weapon < 0 {
            // already searched
        } else if (*an).vitality == 0 || the_world.freeze_timer != 0 {
            search_body();
        } else {
            show_event(EVENT_ALIVE);
        }
    }
}

/// Removes the picked-up object from the world, turns any arrows picked up into the arrow
/// count, and ends the game if the talisman was found.
unsafe fn finish_pickup() {
    unsafe {
        change_object(the_actors.nearest_object as i32, 2);
        add_stuff(ARROWS, (*the_hero.stuff.add(ARROWBASE) as u32 * 10) as u8);
        if *the_hero.stuff.add(TALISMAN) != 0 {
            quit_requested = 1;
            view_status = VIEW_GAME_OVER;
            map_message();
            SetFont(current_rastport, story_font);
            win_colors();
        }
    }
}

/// Handles taking the object `nearest_object`. Returns true if it was taken (so it must be removed).
unsafe fn take_object() -> bool {
    unsafe {
        let an = &raw mut the_actors.list[the_actors.nearest_object as usize];
        let id = (*an).index as u8 as u32;
        let owner = (*an).vitality as u32 & 0x7f;
        match id {
            GOLD_PILE => {
                announce_treasure(c"50 gold pieces".as_ptr());
                the_hero.wealth += GOLD_FOUND;
                true
            }
            SCRAP_OF_PAPER => {
                show_event(EVENT_PAPER);
                show_event(if the_world.region > FIRST_INDOOR_REGION {
                    EVENT_PAPER_INDOORS
                } else {
                    EVENT_PAPER_OUTDOORS
                });
                true
            }
            FRUIT => {
                if the_hero.hunger < NOT_HUNGRY {
                    add_stuff(FRUIT_SLOT, 1);
                    show_event(EVENT_FRUIT_KEPT);
                } else {
                    eat_food(FRUIT_HUNGER);
                }
                true
            }
            TURTLE_EGGS => false, // not taken
            BROTHER_BONES => {
                announce_treasure(c"his brother's bones.".as_ptr());
                ob_listg[BONES.0].ob_stat = 0;
                ob_listg[BONES.1].ob_stat = 0;
                for slot in 0..GOLDBASE {
                    let other = if owner == 1 {
                        julstuff[slot]
                    } else {
                        philstuff[slot]
                    };
                    add_stuff(slot, other);
                }
                true
            }
            _ => take_container(id),
        }
    }
}

/// Opens a chest, urn or sacks (or picks up a plain item) and says what was inside.
unsafe fn take_container(id: u32) -> bool {
    unsafe {
        match id {
            CHEST => announce_container(c"a chest".as_ptr()),
            URN => announce_container(c"a brass urn".as_ptr()),
            SACKS => announce_container(c"some sacks".as_ptr()),
            EMPTY_CHEST | FOOTSTOOL => return false,
            _ => return take_plain_item(id),
        }
        container_contents()
    }
}

/// A single item lying about: the slot is found in the `itrans` table. Unknown objects are left.
unsafe fn take_plain_item(id: u32) -> bool {
    unsafe {
        let mut k = 0;
        while itrans[k] != 0 {
            if id == itrans[k] as u32 {
                let slot = itrans[k + 1] as usize;
                add_stuff(slot, 1);
                announce_treasure(c"a ".as_ptr());
                print_cont(item_name(slot));
                print_cont(c".".as_ptr());
                return true;
            }
            k += 2;
        }
        false
    }
}

/// What was inside a container: nothing, an item, two things or three of something.
/// Returns true (the container is always used up).
unsafe fn container_contents() -> bool {
    unsafe {
        match rand4() {
            0 => print(c"nothing.".as_ptr()),
            1 => {
                let mut i = rand8() as usize + 8;
                if i == 8 {
                    i = ARROWBASE;
                }
                print(c"a ".as_ptr());
                print_cont(item_name(i));
                print_cont(c".".as_ptr());
                add_stuff(i, 1);
            }
            2 => two_things(),
            _ => three_things(),
        }
        true
    }
}

/// "an X and a Y" (the first may be 100 gold pieces instead).
unsafe fn two_things() {
    unsafe {
        let mut i = rand8() + 8;
        if i == 8 {
            i = HUNDRED_GOLD;
            the_hero.wealth += CONTAINER_GOLD;
        } else {
            print_cont(c" a".as_ptr());
        }
        print(item_name(i as usize));
        print_cont(c" and a ".as_ptr());
        let mut k;
        loop {
            k = rand8() + 8;
            if k != i {
                break;
            }
        }
        if k == 8 {
            k = ARROWBASE as i32;
        }
        print_cont(item_name(k as usize));
        if i < GOLDBASE as i32 {
            add_stuff(i as usize, 1);
        }
        add_stuff(k as usize, 1);
    }
}

/// "3 keys" or three of one item.
unsafe fn three_things() {
    unsafe {
        let i = rand8() + 8;
        if i == 8 {
            print(c"3 keys.".as_ptr());
            for _ in 0..3 {
                let mut key = rand8() + KEYBASE;
                if key == 22 {
                    key = 16;
                }
                if key == 23 {
                    key = 20;
                }
                add_stuff(key as usize, 1);
            }
            return;
        }
        print(c"3 ".as_ptr());
        print_cont(item_name(i as usize));
        print_cont(c"s.".as_ptr());
        add_stuff(i as usize, 3);
    }
}

/// Searches the dead body `nearest_object`: takes its weapon and any treasure.
unsafe fn search_body() {
    unsafe {
        const ARCHER_WEAPON: u32 = 4;
        const MAX_WEAPON: u32 = 5;
        const EXTRA_ARROWS_MIN: i32 = 2;
        let an = &raw mut the_actors.list[the_actors.nearest_object as usize];
        print_wrapped(c"% searched the body and found".as_ptr());
        print(c"".as_ptr());
        let mut weapon = (*an).weapon as u32;
        if weapon > MAX_WEAPON {
            weapon = 0;
        }
        if weapon != 0 {
            print_cont(c"a ".as_ptr());
            print_cont(item_name(weapon as usize - 1));
            add_stuff(weapon as usize - 1, 1);
            if weapon as i8 > the_actors.list[0].weapon {
                the_actors.list[0].weapon = weapon as i8;
            }
            if weapon == ARCHER_WEAPON {
                print_cont(c" and ".as_ptr());
                let arrows = rand8() + EXTRA_ARROWS_MIN;
                print_number(arrows, 1);
                print_cont(c" Arrows.".as_ptr());
                add_stuff(ARROWS, arrows as u8);
                (*an).weapon = -1;
                return;
            }
        }
        (*an).weapon = -1;
        let found = roll_body_treasure(an);
        if found != 0 {
            if weapon != 0 {
                print_cont(c" and ".as_ptr());
            }
            if found < GOLDBASE as u32 {
                print_cont(c"a ".as_ptr());
            }
            print_cont(item_name(found as usize));
            if found >= GOLDBASE as u32 {
                the_hero.wealth += item(found as usize).maxshown as i16;
            } else {
                add_stuff(found as usize, 1);
            }
        } else if weapon == 0 {
            print_cont(c"nothing".as_ptr());
        }
        print_cont(c".".as_ptr());
    }
}

/// Picks what a dead enemy carries besides its weapon, from the treasure table for its race.
/// Returns an item number (0 for nothing); characters like shopkeepers (a set figure) carry none.
unsafe fn roll_body_treasure(an: *mut Shape) -> u32 {
    const SETFIG_RACE_BIT: u8 = 0x80;
    const TREASURE_CLASS_SIZE: i32 = 8;
    unsafe {
        if (*an).race & SETFIG_RACE_BIT != 0 {
            return 0;
        }
        let class = (*(&raw const encounter_chart)
            .cast::<Encounter>()
            .add((*an).race as usize))
        .treasure;
        treasure_probs[(class as i32 * TREASURE_CLASS_SIZE + rand8()) as usize] as i32 as u32
    }
}

/// Reveals hidden objects within reach.
unsafe fn look_around() {
    unsafe {
        let mut found = false;
        for i in 0..the_actors.drawn as i32 {
            let an = &raw const the_actors.list[i as usize];
            if (*an).kind == OBJECTS && (*an).race == 0 && actor_distance(i, 0) < LOOK_RANGE {
                found = true;
                change_object(i, 1);
            }
        }
        show_event(if found {
            EVENT_FOUND_HIDDEN
        } else {
            EVENT_NOTHING_HIDDEN
        });
    }
}
