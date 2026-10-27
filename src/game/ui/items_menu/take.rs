//! Taking what lies nearby: gold, fruit, paper, a brother's bones and plain items. Chests, urns and
//! sacks are opened in `containers`.

use super::containers::container_contents;
use super::item_list::VIEW_GAME_OVER;
use super::search_body::search_body;
use super::{
    ARROWBASE, ARROWS, CHEST, FRUIT_SLOT, GOLDBASE, OBJECTS, PRQ_VITALITY, SACKS, SetFont,
    TALISMAN, URN, add_stuff, item_name,
};

use crate::game::actors::interaction::{eat_food, find_nearest_figure};
use crate::game::data::globals::story_font;
use crate::game::data::object_tables::{itrans, ob_listg};
use crate::game::data::state::{the_actors, the_hero, the_screen, the_ui, the_world};
use crate::game::render::screen_fx::{map_message, win_colors};
use crate::game::ui::messages::{announce_container, announce_treasure, print_cont};
use crate::game::ui::text::{queue_print, show_event};
use crate::game::world::objects::change_object;

// Object ids of things that are not plain pickups.
/// Picking up a quiver gives ten arrows each; a dead brother's bones keep the owner in the low 7 bits.
const ARROWS_PER_QUIVER: u32 = 10;
const OWNER_MASK: u32 = 0x7f;

const GOLD_PILE: u32 = 0x0d;
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

/// Search range for the nearest thing to take.
const TAKE_RANGE: i16 = 30;
const PRQ_NOTHING_NEAR: i32 = 10;
const EVENT_PAPER: i32 = 17;
const EVENT_PAPER_OUTDOORS: i32 = 18;
const EVENT_PAPER_INDOORS: i32 = 19;
const EVENT_ALIVE: i32 = 35;
const EVENT_FRUIT_KEPT: i32 = 36;

/// Regions numbered above this are indoors.
const FIRST_INDOOR_REGION: u16 = 7;

/// Takes the nearest object, or searches the nearest dead body.
pub(super) unsafe fn take_nearest() {
    unsafe {
        queue_print(PRQ_VITALITY);
        find_nearest_figure(0, TAKE_RANGE);

        *the_hero.stuff.add(ARROWBASE) = 0;
        if the_actors.nearest_object == 0 {
            queue_print(PRQ_NOTHING_NEAR);

            return;
        }

        let actor = &raw mut the_actors.list[the_actors.nearest_object as usize];

        if (*actor).kind == OBJECTS {
            if take_object() {
                finish_pickup();
            }
        } else if (*actor).weapon < 0 {
            // already searched
        } else if (*actor).vitality == 0 || the_world.freeze_timer != 0 {
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
        add_stuff(
            ARROWS,
            (*the_hero.stuff.add(ARROWBASE) as u32 * ARROWS_PER_QUIVER) as u8,
        );

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
        let actor = &raw mut the_actors.list[the_actors.nearest_object as usize];
        let id = (*actor).index as u8 as u32;

        // Masks with OWNER_MASK, keeping only the bits that matter here.
        let owner = (*actor).vitality as u32 & OWNER_MASK;

        match classify_take(id, the_hero.hunger, the_world.region) {
            Take::Gold => {
                announce_treasure(c"50 gold pieces".as_ptr());
                the_hero.wealth += GOLD_FOUND;

                true
            }

            Take::Paper { indoors } => read_paper(indoors),
            Take::Fruit { keep } => take_fruit(keep),
            Take::Eggs => false, // not taken
            Take::Bones => take_brothers_goods(owner),

            Take::Container(name) => {
                announce_container(name.as_ptr());

                container_contents()
            }

            Take::Left => false,
            Take::Plain => take_plain_item(id),
        }
    }
}

/// Reads the scrap of paper; what it says depends on being indoors.
unsafe fn read_paper(indoors: bool) -> bool {
    show_event(EVENT_PAPER);
    show_event(if indoors {
        EVENT_PAPER_INDOORS
    } else {
        EVENT_PAPER_OUTDOORS
    });

    true
}

/// Fruit is kept in the pack when the hero is not hungry, and eaten otherwise.
unsafe fn take_fruit(keep: bool) -> bool {
    unsafe {
        if keep {
            add_stuff(FRUIT_SLOT, 1);
            show_event(EVENT_FRUIT_KEPT);
        } else {
            eat_food(FRUIT_HUNGER);
        }

        true
    }
}

/// Picks up a fallen brother's bones and everything he carried (`owner` 1 is Julian, otherwise
/// Phillip).
unsafe fn take_brothers_goods(owner: u32) -> bool {
    unsafe {
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

/// Sorts a nearby object by what taking it does. Fruit is kept when the hero is not hungry and eaten
/// otherwise; the scrap of paper reads differently indoors.
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

#[cfg(test)]
mod tests {
    use super::super::CHEST;
    use super::super::SACKS;
    use super::super::URN;
    use super::*;

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
}
