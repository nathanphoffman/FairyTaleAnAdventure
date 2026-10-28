//! Searching a dead enemy: the weapon it carried and any treasure.

use super::{ARROWS, GOLDBASE, add_stuff, item, item_name};
use crate::game::data::object_tables::treasure_probs;
use crate::game::data::random::{Rng, rand8};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::data::tables::{encounter_chart, entry};
use crate::game::output::Output;
use crate::game::ui::messages::print_wrapped;

/// Searches the dead body `nearest_object`: takes its weapon and any treasure.
pub(super) fn search_body(game: &mut Game, out: &mut dyn Output) {
    let actor = ActorRef::new(game.actors.nearest_object as usize);
    print_wrapped(game, out, "% searched the body and found");
    out.print("");
    let (body_weapon, hero_weapon) = (game.actor(actor).weapon, game.actors.list[0].weapon);
    let race = game.actor(actor).race;
    let rng = &mut game.rng;
    let loot = body_loot(body_weapon, hero_weapon, |roll| match roll {
        LootRoll::Arrows => rand8(rng) + EXTRA_ARROWS_MIN,
        LootRoll::Treasure => roll_body_treasure(race, rng) as i32,
    });

    game.actor_mut(actor).weapon = -1;

    take_found_weapon(game, out, &loot);

    if let Some(arrows) = loot.arrows {
        out.print_cont(" and ");
        out.print_number(arrows, 1);
        out.print_cont(" Arrows.");
        add_stuff(game, ARROWS, arrows as u8);

        return;
    }

    take_treasure(game, out, &loot);
    out.print_cont(".");
}

/// Says what weapon was found, adds it to the pack, and arms the hero with it if it is better.
fn take_found_weapon(game: &mut Game, out: &mut dyn Output, loot: &BodyLoot) {
    if loot.weapon != 0 {
        out.print_cont("a ");
        out.print_cont(item_name(loot.weapon as usize - 1));
        add_stuff(game, loot.weapon as usize - 1, 1);
        game.actors.list[0].weapon = loot.hero_weapon;
    }
}

/// Says what other treasure was found and takes it: gold into the purse, anything else into the
/// pack. Says "nothing" if there was neither weapon nor treasure.
fn take_treasure(game: &mut Game, out: &mut dyn Output, loot: &BodyLoot) {
    if loot.treasure != 0 {
        if loot.weapon != 0 {
            out.print_cont(" and ");
        }

        if loot.treasure < GOLDBASE as u32 {
            out.print_cont("a ");
        }

        out.print_cont(item_name(loot.treasure as usize));

        if loot.treasure >= GOLDBASE as u32 {
            game.hero.wealth += item(loot.treasure as usize).maxshown as i16;
        } else {
            add_stuff(game, loot.treasure as usize, 1);
        }
    } else if loot.weapon == 0 {
        out.print_cont("nothing");
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

/// The random draws a search can ask for: the arrows only for an archer, the treasure only when
/// there are no arrows.
enum LootRoll {
    Arrows,
    Treasure,
}

fn body_loot(
    body_weapon_raw: i8,
    hero_weapon: i8,
    mut roll: impl FnMut(LootRoll) -> i32,
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
            arrows: Some(roll(LootRoll::Arrows)),
            treasure: 0,
        };
    }

    BodyLoot {
        weapon,
        hero_weapon,
        arrows: None,
        treasure: roll(LootRoll::Treasure) as u32,
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
fn roll_body_treasure(race: u8, rng: &mut Rng) -> u32 {
    const SETFIG_RACE_BIT: u8 = 0x80;
    const TREASURE_CLASS_SIZE: i32 = 8;
    if race & SETFIG_RACE_BIT != 0 {
        return 0;
    }

    let class = entry(&encounter_chart, race as usize).treasure;

    treasure_probs[(class as i32 * TREASURE_CLASS_SIZE + rand8(rng)) as usize] as i32 as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rolls that give the treasure `treasure` and never ask for arrows.
    fn treasure_only(treasure: i32) -> impl FnMut(LootRoll) -> i32 {
        move |roll| match roll {
            LootRoll::Treasure => treasure,
            LootRoll::Arrows => unreachable!(),
        }
    }

    #[test]
    fn body_loot_rules() {
        let l = body_loot(2, 1, treasure_only(7));
        assert_eq!(
            (l.weapon, l.hero_weapon, l.arrows, l.treasure),
            (2, 2, None, 7)
        );

        // a weaker weapon does not replace the one in hand
        assert_eq!(body_loot(2, 3, treasure_only(0)).hero_weapon, 3);

        // an archer gives arrows and nothing else
        let l = body_loot(4, 1, |roll| match roll {
            LootRoll::Arrows => 5,
            LootRoll::Treasure => unreachable!(),
        });
        assert_eq!(
            (l.weapon, l.arrows, l.treasure, l.hero_weapon),
            (4, Some(5), 0, 4)
        );

        // no weapon at all
        let l = body_loot(0, 2, treasure_only(0));
        assert_eq!((l.weapon, l.hero_weapon, l.treasure), (0, 2, 0));

        // an already searched body (-1) has no weapon
        assert_eq!(body_loot(-1, 2, treasure_only(0)).weapon, 0);
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
