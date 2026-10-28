//! Searching a dead enemy: the weapon it carried and any treasure.

use super::{ARROWS, GOLDBASE, add_stuff, item, item_name};
use crate::game::actors::encounters::Encounter;
use crate::game::data::object_tables::treasure_probs;
use crate::game::data::random::rand8;
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::data::state::{the_actors, the_hero};
use crate::game::data::tables::encounter_chart;
use crate::game::ui::messages::{print, print_cont, print_wrapped};
use crate::game::ui::text::print_number;

/// Searches the dead body `nearest_object`: takes its weapon and any treasure.
pub(super) unsafe fn search_body(game: &mut Game) {
    unsafe {
        let mut actor = ActorRef::new(the_actors.nearest_object as usize);
        print_wrapped(game, "% searched the body and found");
        print(game, "");
        let loot = body_loot(
            actor.weapon,
            the_actors.list[0].weapon,
            || rand8() + EXTRA_ARROWS_MIN,
            || roll_body_treasure(actor),
        );

        actor.weapon = -1;

        take_found_weapon(game, &loot);

        if let Some(arrows) = loot.arrows {
            print_cont(game, " and ");
            print_number(game, arrows, 1);
            print_cont(game, " Arrows.");
            add_stuff(ARROWS, arrows as u8);

            return;
        }

        take_treasure(game, &loot);
        print_cont(game, ".");
    }
}

/// Says what weapon was found, adds it to the pack, and arms the hero with it if it is better.
unsafe fn take_found_weapon(game: &mut Game, loot: &BodyLoot) {
    unsafe {
        if loot.weapon != 0 {
            print_cont(game, "a ");
            print_cont(game, item_name(loot.weapon as usize - 1));
            add_stuff(loot.weapon as usize - 1, 1);
            the_actors.list[0].weapon = loot.hero_weapon;
        }
    }
}

/// Says what other treasure was found and takes it: gold into the purse, anything else into the
/// pack. Says "nothing" if there was neither weapon nor treasure.
unsafe fn take_treasure(game: &mut Game, loot: &BodyLoot) {
    unsafe {
        if loot.treasure != 0 {
            if loot.weapon != 0 {
                print_cont(game, " and ");
            }

            if loot.treasure < GOLDBASE as u32 {
                print_cont(game, "a ");
            }

            print_cont(game, item_name(loot.treasure as usize));

            if loot.treasure >= GOLDBASE as u32 {
                the_hero.wealth += item(loot.treasure as usize).maxshown as i16;
            } else {
                add_stuff(loot.treasure as usize, 1);
            }
        } else if loot.weapon == 0 {
            print_cont(game, "nothing");
        }
    }
}

const ARCHER_WEAPON: u32 = 4;
const EXTRA_ARROWS_MIN: i32 = 2;

/// What searching a body turns up.
#[derive(Debug, PartialEq, Eq)]
struct BodyLoot {
    /// The weapon found (0 for none).
    pub(super) weapon: u32,
    /// The weapon the hero holds afterwards (a better one replaces his).
    pub(super) hero_weapon: i8,
    /// An archer's arrows (the count), instead of any other treasure.
    pub(super) arrows: Option<i32>,
    /// The item number of other treasure (0 for none).
    pub(super) treasure: u32,
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
unsafe fn roll_body_treasure(actor: ActorRef) -> u32 {
    const SETFIG_RACE_BIT: u8 = 0x80;
    const TREASURE_CLASS_SIZE: i32 = 8;
    unsafe {
        // Masks with SETFIG_RACE_BIT, keeping only the bits that matter here.
        if actor.race & SETFIG_RACE_BIT != 0 {
            return 0;
        }

        let class = (*(&raw const encounter_chart)
            .cast::<Encounter>()
            .add(actor.race as usize))
        .treasure;

        treasure_probs[(class as i32 * TREASURE_CLASS_SIZE + rand8()) as usize] as i32 as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
