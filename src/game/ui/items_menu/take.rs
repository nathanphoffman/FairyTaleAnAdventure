//! Taking what lies nearby: gold, fruit, paper, a brother's bones and plain items. Chests, urns and
//! sacks are opened in `containers`.

use super::containers::container_contents;
use super::item_list::VIEW_GAME_OVER;
use super::search_body::search_body;
use super::{
    ARROWBASE, ARROWS, CHEST, FRUIT_SLOT, GOLDBASE, OBJECTS, PRQ_VITALITY, SACKS, TALISMAN, URN,
    add_stuff, item_name,
};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::output::Output;

use crate::game::actors::interaction::{eat_food, find_nearest_figure};
use crate::game::data::object_tables::itrans;
use crate::game::render::screen_fx::{map_message, win_colors};
use crate::game::ui::messages::{announce_container, announce_treasure};
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
pub(super) fn take_nearest(game: &mut Game, out: &mut dyn Output) {
    queue_print(&mut game.ui, PRQ_VITALITY);
    find_nearest_figure(game, 0, TAKE_RANGE);

    game.hero.stuff_mut()[ARROWBASE] = 0;
    if game.actors.nearest_object == 0 {
        queue_print(&mut game.ui, PRQ_NOTHING_NEAR);

        return;
    }

    let actor = ActorRef::new(game.actors.nearest_object as usize);

    if game.actor(actor).kind == OBJECTS {
        if take_object(game, out) {
            finish_pickup(game, out);
        }
    } else if game.actor(actor).weapon < 0 {
        // already searched
    } else if game.actor(actor).vitality == 0 || game.world.freeze_timer != 0 {
        search_body(game, out);
    } else {
        show_event(game, out, EVENT_ALIVE);
    }
}

/// Removes the picked-up object from the world, turns any arrows picked up into the arrow
/// count, and ends the game if the talisman was found.
fn finish_pickup(game: &mut Game, out: &mut dyn Output) {
    change_object(
        &mut game.objects,
        &game.actors,
        &mut game.world,
        game.actors.nearest_object as i32,
        2,
    );
    add_stuff(
        game,
        ARROWS,
        (game.hero.stuff()[ARROWBASE] as u32 * ARROWS_PER_QUIVER) as u8,
    );

    if game.hero.stuff()[TALISMAN] != 0 {
        game.ui.quit_requested = 1;
        game.ui.view_status = VIEW_GAME_OVER;
        map_message(game, out);
        out.use_story_font();
        win_colors(game, out);
    }
}

/// Handles taking the object `nearest_object`. Returns true if it was taken (so it must be removed).
fn take_object(game: &mut Game, out: &mut dyn Output) -> bool {
    let actor = ActorRef::new(game.actors.nearest_object as usize);
    let id = game.actor(actor).index as u8 as u32;

    let owner = game.actor(actor).vitality as u32 & OWNER_MASK;

    match classify_take(id, game.hero.hunger, game.world.region) {
        Take::Gold => {
            announce_treasure(game, out, "50 gold pieces");
            game.hero.wealth += GOLD_FOUND;

            true
        }

        Take::Paper { indoors } => read_paper(game, out, indoors),
        Take::Fruit { keep } => take_fruit(game, out, keep),
        Take::Eggs => false, // not taken
        Take::Bones => take_brothers_goods(game, out, owner),

        Take::Container(name) => {
            announce_container(game, out, name);

            container_contents(game, out)
        }

        Take::Left => false,
        Take::Plain => take_plain_item(game, out, id),
    }
}

/// Reads the scrap of paper; what it says depends on being indoors.
fn read_paper(game: &mut Game, out: &mut dyn Output, indoors: bool) -> bool {
    show_event(game, out, EVENT_PAPER);
    show_event(
        game,
        out,
        if indoors {
            EVENT_PAPER_INDOORS
        } else {
            EVENT_PAPER_OUTDOORS
        },
    );

    true
}

/// Fruit is kept in the pack when the hero is not hungry, and eaten otherwise.
fn take_fruit(game: &mut Game, out: &mut dyn Output, keep: bool) -> bool {
    if keep {
        add_stuff(game, FRUIT_SLOT, 1);
        show_event(game, out, EVENT_FRUIT_KEPT);
    } else {
        eat_food(game, out, FRUIT_HUNGER);
    }

    true
}

/// Picks up a fallen brother's bones and everything he carried (`owner` 1 is Julian, otherwise
/// Phillip).
fn take_brothers_goods(game: &mut Game, out: &mut dyn Output, owner: u32) -> bool {
    announce_treasure(game, out, "his brother's bones.");
    game.objects.global[BONES.0].ob_stat = 0;
    game.objects.global[BONES.1].ob_stat = 0;

    for slot in 0..GOLDBASE {
        let other = if owner == 1 {
            game.hero.julstuff[slot]
        } else {
            game.hero.philstuff[slot]
        };

        add_stuff(game, slot, other);
    }

    true
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
    Container(&'static str),
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
        CHEST => Take::Container("a chest"),
        URN => Take::Container("a brass urn"),
        SACKS => Take::Container("some sacks"),
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
fn take_plain_item(game: &mut Game, out: &mut dyn Output, id: u32) -> bool {
    match item_slot(id, &itrans) {
        Some(slot) => {
            add_stuff(game, slot, 1);
            announce_treasure(game, out, "a ");
            out.print_cont(item_name(slot));
            out.print_cont(".");

            true
        }

        None => false,
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
        assert_eq!(classify_take(CHEST, 0, 0), Take::Container("a chest"));
        assert_eq!(classify_take(URN, 0, 0), Take::Container("a brass urn"));
        assert_eq!(classify_take(SACKS, 0, 0), Take::Container("some sacks"));
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
