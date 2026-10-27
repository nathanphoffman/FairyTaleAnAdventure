//! The Items menu actions (from `choose_menu_entry` in csrc/fmain.c): showing the item list, taking what
//! is nearby (objects, containers, dead bodies) and looking around.

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
        let bm = (*(*the_screen.drawing_page).ri_page).BitMap;
        the_screen.rp_map.BitMap = bm;
        SetRast(&raw mut the_screen.rp_map, 0);

        InitBitMap(
            &raw mut the_screen.pagea,
            ITEM_IMAGE_PLANES,
            ITEM_IMAGE_WIDTH,
            ITEM_IMAGE_ROWS,
        );
        let data = (*(&raw mut the_memory.seq_list)
            .cast::<SeqInfo>()
            .add(OBJECTS as usize))
        .location;
        for plane in 0..ITEM_IMAGE_PLANES as usize {
            the_screen.pagea.Planes[plane] = data.add(plane * ITEM_IMAGE_BYTES);
        }

        draw_item_icons(bm);

        SetRGB4(&raw mut the_screen.vp_page, BACKGROUND_PEN as i32, 0, 0, 0);
        show_still_screen();
        LoadRGB4(
            &raw mut the_screen.vp_page,
            (&raw const page_colors).cast(),
            COLOURS_SHOWN,
        );
        the_screen.view_status = VIEW_ITEMS;
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
                    &raw mut the_screen.pagea,
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
            the_ui.quit_requested = 1;
            the_screen.view_status = VIEW_GAME_OVER;
            map_message();
            SetFont(the_screen.current_rastport, story_font);
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
        match classify_take(id, the_hero.hunger, the_world.region) {
            Take::Gold => {
                announce_treasure(c"50 gold pieces".as_ptr());
                the_hero.wealth += GOLD_FOUND;
                true
            }
            Take::Paper { indoors } => {
                show_event(EVENT_PAPER);
                show_event(if indoors {
                    EVENT_PAPER_INDOORS
                } else {
                    EVENT_PAPER_OUTDOORS
                });
                true
            }
            Take::Fruit { keep } => {
                if keep {
                    add_stuff(FRUIT_SLOT, 1);
                    show_event(EVENT_FRUIT_KEPT);
                } else {
                    eat_food(FRUIT_HUNGER);
                }
                true
            }
            Take::Eggs => false, // not taken
            Take::Bones => {
                announce_treasure(c"his brother's bones.".as_ptr());
                ob_listg[BONES.0].ob_stat = 0;
                ob_listg[BONES.1].ob_stat = 0;
                for slot in 0..GOLDBASE {
                    let other = if owner == 1 {
                        the_hero.julstuff[slot]
                    } else {
                        the_hero.philstuff[slot]
                    };
                    add_stuff(slot, other);
                }
                true
            }
            Take::Container(name) => {
                announce_container(name.as_ptr());
                container_contents()
            }
            Take::Left => false,
            Take::Plain => take_plain_item(id),
        }
    }
}

/// What taking an object does, from its id.
#[derive(Debug, PartialEq, Eq)]
enum Take {
    Gold,
    /// The scrap of paper; the second message depends on being indoors.
    Paper {
        indoors: bool,
    },
    /// Fruit: kept in the pack when not hungry, otherwise eaten.
    Fruit {
        keep: bool,
    },
    /// Turtle eggs cannot be taken.
    Eggs,
    Bones,
    /// A chest, urn or sacks (with the name announced).
    Container(&'static core::ffi::CStr),
    /// An empty chest or a footstool: nothing happens.
    Left,
    /// Anything else: looked up in `itrans`.
    Plain,
}

fn classify_take(id: u32, hunger: i16, region: u16) -> Take {
    match id {
        GOLD_PILE => Take::Gold,
        SCRAP_OF_PAPER => Take::Paper {
            indoors: region > FIRST_INDOOR_REGION,
        },
        FRUIT => Take::Fruit {
            keep: hunger < NOT_HUNGRY,
        },
        TURTLE_EGGS => Take::Eggs,
        BROTHER_BONES => Take::Bones,
        CHEST => Take::Container(c"a chest"),
        URN => Take::Container(c"a brass urn"),
        SACKS => Take::Container(c"some sacks"),
        EMPTY_CHEST | FOOTSTOOL => Take::Left,
        _ => Take::Plain,
    }
}

/// The inventory slot for object `id` in the (id, slot) pair table `itrans`, which ends with a 0 id.
fn item_slot(id: u32, table: &[u8]) -> Option<usize> {
    let mut k = 0;
    while table[k] != 0 {
        if id == table[k] as u32 {
            return Some(table[k + 1] as usize);
        }
        k += 2;
    }
    None
}

/// The item a container holds when it rolls `roll` (0-7) for one item: slot 8 means arrows.
fn container_item(roll: i32) -> usize {
    let i = roll as usize + 8;
    if i == 8 { ARROWBASE } else { i }
}

/// A key found in a container: the two odd key types fold onto the first of their kind.
fn container_key(roll: i32) -> i32 {
    match roll + KEYBASE {
        22 => 16,
        23 => 20,
        key => key,
    }
}

/// A single item lying about: the slot is found in the `itrans` table. Unknown objects are left.
unsafe fn take_plain_item(id: u32) -> bool {
    unsafe {
        match item_slot(id, &itrans) {
            Some(slot) => {
                add_stuff(slot, 1);
                announce_treasure(c"a ".as_ptr());
                print_cont(item_name(slot));
                print_cont(c".".as_ptr());
                true
            }
            None => false,
        }
    }
}

/// What was inside a container. Returns true (the container is always used up).
unsafe fn container_contents() -> bool {
    unsafe {
        match roll_contents(rand4, rand8) {
            Contents::Nothing => print(c"nothing.".as_ptr()),
            Contents::One(i) => {
                print(c"a ".as_ptr());
                print_cont(item_name(i));
                print_cont(c".".as_ptr());
                add_stuff(i, 1);
            }
            Contents::Two { first, second } => {
                if first == HUNDRED_GOLD as usize {
                    the_hero.wealth += CONTAINER_GOLD;
                } else {
                    print_cont(c" a".as_ptr());
                }
                print(item_name(first));
                print_cont(c" and a ".as_ptr());
                print_cont(item_name(second));
                if first < GOLDBASE {
                    add_stuff(first, 1);
                }
                add_stuff(second, 1);
            }
            Contents::ThreeKeys(keys) => {
                print(c"3 keys.".as_ptr());
                for key in keys {
                    add_stuff(key as usize, 1);
                }
            }
            Contents::ThreeOf(i) => {
                print(c"3 ".as_ptr());
                print_cont(item_name(i));
                print_cont(c"s.".as_ptr());
                add_stuff(i, 3);
            }
        }
        true
    }
}

/// What a container holds: nothing, an item, two things (the first may be 100 gold pieces), three
/// keys, or three of one item.
#[derive(Debug, PartialEq, Eq)]
enum Contents {
    Nothing,
    One(usize),
    Two { first: usize, second: usize },
    ThreeKeys([i32; 3]),
    ThreeOf(usize),
}

/// Rolls the contents with the game's random draws (`pick4` 0-3, `pick8` 0-7), in the same order
/// as the original: the kind first, then the item rolls.
fn roll_contents(mut pick4: impl FnMut() -> i32, mut pick8: impl FnMut() -> i32) -> Contents {
    match pick4() {
        0 => Contents::Nothing,
        1 => Contents::One(container_item(pick8())),
        2 => {
            let mut first = pick8() + 8;
            if first == 8 {
                first = HUNDRED_GOLD;
            }
            let mut second;
            loop {
                second = pick8() + 8;
                if second != first {
                    break;
                }
            }
            if second == 8 {
                second = ARROWBASE as i32;
            }
            Contents::Two {
                first: first as usize,
                second: second as usize,
            }
        }
        _ => {
            let i = pick8() + 8;
            if i == 8 {
                Contents::ThreeKeys([0; 3].map(|_| container_key(pick8())))
            } else {
                Contents::ThreeOf(i as usize)
            }
        }
    }
}

/// Searches the dead body `nearest_object`: takes its weapon and any treasure.
unsafe fn search_body() {
    unsafe {
        let an = &raw mut the_actors.list[the_actors.nearest_object as usize];
        print_wrapped(c"% searched the body and found".as_ptr());
        print(c"".as_ptr());
        let loot = body_loot(
            (*an).weapon,
            the_actors.list[0].weapon,
            || rand8() + EXTRA_ARROWS_MIN,
            || roll_body_treasure(an),
        );
        (*an).weapon = -1;
        if loot.weapon != 0 {
            print_cont(c"a ".as_ptr());
            print_cont(item_name(loot.weapon as usize - 1));
            add_stuff(loot.weapon as usize - 1, 1);
            the_actors.list[0].weapon = loot.hero_weapon;
        }
        if let Some(arrows) = loot.arrows {
            print_cont(c" and ".as_ptr());
            print_number(arrows, 1);
            print_cont(c" Arrows.".as_ptr());
            add_stuff(ARROWS, arrows as u8);
            return;
        }
        if loot.treasure != 0 {
            if loot.weapon != 0 {
                print_cont(c" and ".as_ptr());
            }
            if loot.treasure < GOLDBASE as u32 {
                print_cont(c"a ".as_ptr());
            }
            print_cont(item_name(loot.treasure as usize));
            if loot.treasure >= GOLDBASE as u32 {
                the_hero.wealth += item(loot.treasure as usize).maxshown as i16;
            } else {
                add_stuff(loot.treasure as usize, 1);
            }
        } else if loot.weapon == 0 {
            print_cont(c"nothing".as_ptr());
        }
        print_cont(c".".as_ptr());
    }
}

const ARCHER_WEAPON: u32 = 4;
const EXTRA_ARROWS_MIN: i32 = 2;

/// What searching a body turns up.
#[derive(Debug, PartialEq, Eq)]
struct BodyLoot {
    /// The weapon found (0 for none).
    weapon: u32,
    /// The weapon the hero holds afterwards (a better one replaces his).
    hero_weapon: i8,
    /// An archer's arrows (the count), instead of any other treasure.
    arrows: Option<i32>,
    /// The item number of other treasure (0 for none).
    treasure: u32,
}

/// `arrow_roll` and `treasure_roll` are the random draws: the arrows only for an archer, the
/// treasure only when there are no arrows.
fn body_loot(
    body_weapon_raw: i8,
    hero_weapon: i8,
    arrow_roll: impl FnOnce() -> i32,
    treasure_roll: impl FnOnce() -> u32,
) -> BodyLoot {
    let weapon = body_weapon(body_weapon_raw);
    let hero_weapon = if weapon != 0 {
        better_weapon(hero_weapon, weapon)
    } else {
        hero_weapon
    };
    if weapon == ARCHER_WEAPON {
        return BodyLoot {
            weapon,
            hero_weapon,
            arrows: Some(arrow_roll()),
            treasure: 0,
        };
    }
    BodyLoot {
        weapon,
        hero_weapon,
        arrows: None,
        treasure: treasure_roll(),
    }
}

/// The heaviest weapon an enemy can carry and be searched for.
const MAX_BODY_WEAPON: i8 = 5;

/// The weapon a body yields (0 for none): the unarmed and odd values count as nothing.
fn body_weapon(raw: i8) -> u32 {
    if raw as u8 as u32 > MAX_BODY_WEAPON as u32 {
        0
    } else {
        raw as u8 as u32
    }
}

/// The hero keeps a better weapon in hand and only takes up a found one that is better.
fn better_weapon(current: i8, found: u32) -> i8 {
    if found as i8 > current {
        found as i8
    } else {
        current
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_contents_follow_the_rolls() {
        let rolls = |kind: i32, items: &[i32]| {
            let mut next = items.iter().copied();
            roll_contents(|| kind, || next.next().unwrap())
        };
        assert_eq!(rolls(0, &[]), Contents::Nothing);
        assert_eq!(rolls(1, &[0]), Contents::One(ARROWBASE));
        assert_eq!(rolls(1, &[3]), Contents::One(11));
        // two things: a first item and a different second one
        assert_eq!(
            rolls(2, &[2, 5]),
            Contents::Two {
                first: 10,
                second: 13
            }
        );
        // 100 gold first can never match the second roll, so the arrows are accepted
        assert_eq!(
            rolls(2, &[0, 0]),
            Contents::Two {
                first: HUNDRED_GOLD as usize,
                second: ARROWBASE
            }
        );
        assert_eq!(
            rolls(2, &[0, 4]),
            Contents::Two {
                first: HUNDRED_GOLD as usize,
                second: 12
            }
        );
        assert_eq!(
            rolls(2, &[3, 3, 0]),
            Contents::Two {
                first: 11,
                second: ARROWBASE
            }
        );
        // three of one thing, or three keys
        assert_eq!(rolls(3, &[4]), Contents::ThreeOf(12));
        assert_eq!(rolls(3, &[0, 0, 5, 6]), Contents::ThreeKeys([16, 21, 16]));
    }

    #[test]
    fn body_loot_rules() {
        let none = || -> u32 { 0 };
        let l = body_loot(2, 1, || unreachable!(), || 7);
        assert_eq!(
            (l.weapon, l.hero_weapon, l.arrows, l.treasure),
            (2, 2, None, 7)
        );
        // a weaker weapon does not replace the one in hand
        assert_eq!(body_loot(2, 3, || unreachable!(), none).hero_weapon, 3);
        // an archer gives arrows and nothing else
        let l = body_loot(4, 1, || 5, || unreachable!());
        assert_eq!(
            (l.weapon, l.arrows, l.treasure, l.hero_weapon),
            (4, Some(5), 0, 4)
        );
        // no weapon at all
        let l = body_loot(0, 2, || unreachable!(), none);
        assert_eq!((l.weapon, l.hero_weapon, l.treasure), (0, 2, 0));
        // an already searched body (-1) has no weapon
        assert_eq!(body_loot(-1, 2, || unreachable!(), none).weapon, 0);
    }

    #[test]
    fn bodies_yield_only_real_weapons() {
        assert_eq!(body_weapon(0), 0);
        assert_eq!(body_weapon(1), 1);
        assert_eq!(body_weapon(5), 5);
        assert_eq!(body_weapon(6), 0);
        assert_eq!(body_weapon(-1), 0); // already searched
    }

    #[test]
    fn a_found_weapon_is_only_taken_up_if_better() {
        assert_eq!(better_weapon(1, 3), 3);
        assert_eq!(better_weapon(4, 3), 4);
        assert_eq!(better_weapon(3, 3), 3);
    }

    #[test]
    fn take_classes() {
        assert_eq!(classify_take(GOLD_PILE, 0, 0), Take::Gold);
        assert_eq!(
            classify_take(SCRAP_OF_PAPER, 0, FIRST_INDOOR_REGION),
            Take::Paper { indoors: false }
        );
        assert_eq!(
            classify_take(SCRAP_OF_PAPER, 0, FIRST_INDOOR_REGION + 1),
            Take::Paper { indoors: true }
        );
        assert_eq!(
            classify_take(FRUIT, NOT_HUNGRY - 1, 0),
            Take::Fruit { keep: true }
        );
        assert_eq!(
            classify_take(FRUIT, NOT_HUNGRY, 0),
            Take::Fruit { keep: false }
        );
        assert_eq!(classify_take(TURTLE_EGGS, 0, 0), Take::Eggs);
        assert_eq!(classify_take(BROTHER_BONES, 0, 0), Take::Bones);
        assert_eq!(classify_take(CHEST, 0, 0), Take::Container(c"a chest"));
        assert_eq!(classify_take(URN, 0, 0), Take::Container(c"a brass urn"));
        assert_eq!(classify_take(SACKS, 0, 0), Take::Container(c"some sacks"));
        assert_eq!(classify_take(EMPTY_CHEST, 0, 0), Take::Left);
        assert_eq!(classify_take(FOOTSTOOL, 0, 0), Take::Left);
        assert_eq!(classify_take(1, 0, 0), Take::Plain);
    }

    #[test]
    fn item_slot_lookup_stops_at_the_zero_pair() {
        let table = [5u8, 35, 7, 9, 0, 0, 9, 1];
        assert_eq!(item_slot(7, &table), Some(9));
        assert_eq!(item_slot(5, &table), Some(35));
        assert_eq!(item_slot(9, &table), None);
        assert_eq!(item_slot(3, &table), None);
    }

    #[test]
    fn container_rolls() {
        assert_eq!(container_item(0), ARROWBASE);
        assert_eq!(container_item(1), 9);
        assert_eq!(container_item(7), 15);
        assert_eq!(container_key(0), 16);
        assert_eq!(container_key(5), 21);
        assert_eq!(container_key(6), 16);
        assert_eq!(container_key(7), 20);
    }
}
