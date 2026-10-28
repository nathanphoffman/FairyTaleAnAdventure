//! Placing monsters and carriers, and what happens when an actor dies (from csrc/fmain.c).

use crate::game::actors::interaction::blocked_at;
use crate::game::data::object_tables::weapon_probs;
use crate::game::data::random::{bitrand, rand2, rand4, rnd};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::data::state::Hero;
use crate::game::data::tables::{encounter_chart, entry};
use crate::game::data::types::Shape;
use crate::game::files::disk::motor_off;
use crate::game::files::loading::{finish_sprite_load, load_sprite_file};
use crate::game::ui::music::set_mood;
use crate::game::ui::text::{queue_print, show_event, speak};
use crate::game::world::map::terrain_at;

/// One kind of monster (`struct encounter`): hit points, how aggressive, which weapon row it uses,
/// how clever, what treasure it drops, and which sprite file it needs.
#[repr(C)]
pub struct Encounter {
    pub hitpoints: i8,
    pub agressive: i8,
    pub arms: i8,
    pub cleverness: i8,
    pub treasure: i8,
    pub file_id: i8,
}

// Actor kinds (`type`), states and goals (see csrc/ftale.h).
const ENEMY: i8 = 2;
const SETFIG: i8 = 4;
const CARRIER: i8 = 5;
const DRAGON: i8 = 6;
const STILL: i8 = 13;
const DYING: i8 = 14;
const DEAD: i8 = 15;
const ATTACK1: i8 = 1;
const ARCHER1: i8 = 3;
const DEATH: i8 = 7;

/// A weapon with this bit set is a bow, so the monster is an archer.
const BOW_BIT: i8 = 4;

/// Tries this many random spots before giving up.
const MAX_TRY: i32 = 15;

/// The encounter's `mix_flag` bits: 2 picks one of a pair of races, 4 picks the weapon at random.
const MIX_RANDOM_WEAPON: i16 = 4;
const MIX_RACE_PAIR: i16 = 2;

/// The enemy kind that never mixes races, and the mask that rounds a race down to the first of its
/// pair.
const KIND_NO_MIX: u16 = 4;
const PAIR_MASK: u16 = 0xfffe;

/// Each chart row has four weapon choices.
const WEAPON_CHOICES: i32 = 4;

/// An enemy starts this far off from the screen's corner (its sprite's anchor, in pixels).
const ENEMY_ANCHOR: (u16, u16) = (8, 26);

/// A carrier appears this far from its extent's corner.
const CARRIER_OFFSET: (u16, u16) = (250, 200);

/// Actors active once a carrier is out: the hero, the raft, the first person, and the carrier.
const ACTORS_WITH_CARRIER: i16 = 4;

/// The extent value (`v3`) of the special encounter that always appears at a fixed place.
const FIXED_PLACE_ENCOUNTER: u8 = 7;
const FIXED_PLACE: (u16, u16) = (21635, 25762);

/// The spider pit extent type, where monsters may stand on terrain 7.
const SPIDER_PIT: u16 = 52;
const TERRAIN_SWAMP: i32 = 7;

/// The monster that is the dark knight and the kinds of people that matter in `check_death`.
const RACE_DARK_KNIGHT: u8 = 7;
const RACE_LASSO_KEEPER: u8 = 0x89;
const SPEAK_DARK_KNIGHT_DIES: i32 = 42;

/// Penalties and rewards for a death.
const KINDNESS_LOSS: i16 = 3;
const LUCK_LOSS: i16 = 5;
const DYING_COUNTDOWN: i8 = 7;

/// Print queue entries: vitality and stats.
const PRQ_STATS: i32 = 4;
const PRQ_VITALITY: i32 = 7;

/// Carrier numbers.
const CARRIER_DRAGON: i16 = 10;
const CARRIER_TURTLE: i16 = 5;
const VITALITY_CARRIER: i16 = 50;
const SEQ_ENEMY: usize = 2;
const FIRST_CARRIER_SLOT: usize = 3;

/// Places a new monster in slot `i` of `actor_list` somewhere within `spread` pixels of the
/// encounter spot (`encounter_x`, `encounter_y`), on a free square. Its race is `encounter_type`
/// (mixed with the one next to it when `mixflag` bit 1 is set). Returns 1 if it was placed and
/// 0 if no free spot was found in 15 tries.
pub fn place_encounter(game: &mut Game, i: u16, spread: u16) -> i32 {
    let actor = ActorRef::new(i as usize);
    let Some((xtest, ytest)) = find_open_spot(game, i, spread) else {
        return 0;
    };

    game.actor_mut(actor).abs_x = xtest;
    game.actor_mut(actor).abs_y = ytest;
    game.actor_mut(actor).kind = ENEMY;
    let race = encounter_race(game);
    game.actor_mut(actor).race = race as u8;
    arm_enemy(game, actor, race);

    game.actor_mut(actor).rel_x = game
        .actor(actor)
        .abs_x
        .wrapping_sub(game.world.map_x)
        .wrapping_sub(ENEMY_ANCHOR.0);
    game.actor_mut(actor).rel_y = game
        .actor(actor)
        .abs_y
        .wrapping_sub(game.world.map_y)
        .wrapping_sub(ENEMY_ANCHOR.1);

    1
}

/// Gives the new enemy its weapon (picked from the race's chart, at random for a mixed group),
/// its goal (archers shoot, the rest attack, both as clever as the race is) and its hit points.
fn arm_enemy(game: &mut Game, actor: ActorRef, race: i32) {
    if game.encounter.mix_flag & MIX_RANDOM_WEAPON != 0 {
        game.encounter.weapon_type = rand4(&mut game.rng) as i16;
    }

    let chart = entry(&encounter_chart, race as usize);
    let w = (chart.arms as i32 * WEAPON_CHOICES) + game.encounter.weapon_type as i32;
    game.actor_mut(actor).weapon = weapon_probs[w as usize];
    game.actor_mut(actor).state = STILL;
    game.actor_mut(actor).environ = 0;
    game.actor_mut(actor).facing = 0;

    if game.actor(actor).weapon & BOW_BIT != 0 {
        game.actor_mut(actor).goal = ARCHER1 + chart.cleverness;
    } else {
        game.actor_mut(actor).goal = ATTACK1 + chart.cleverness;
    }

    game.actor_mut(actor).vitality = chart.hitpoints as i16;
}

/// Finds a place near the encounter point for actor `i`, trying up to `MAX_TRY` random spots
/// within `spread` pixels. Some encounters always use one fixed place. Returns `None` if every
/// spot was blocked.
fn find_open_spot(game: &mut Game, i: u16, spread: u16) -> Option<(u16, u16)> {
    if game.extents[game.world.extent].v3 == FIXED_PLACE_ENCOUNTER {
        return Some(FIXED_PLACE);
    }

    for _ in 0..MAX_TRY {
        let xtest = (game.encounter.x as i32 + bitrand(&mut game.rng, spread as i32)
            - (spread as i32 / 2)) as u16;
        let ytest = (game.encounter.y as i32 + bitrand(&mut game.rng, spread as i32)
            - (spread as i32 / 2)) as u16;

        if blocked_at(game, xtest as i16, ytest as i16, i as i16) == 0 {
            return Some((xtest, ytest));
        }

        if game.world.extent_type == SPIDER_PIT
            && terrain_at(game, xtest as i32, ytest as i32) == TERRAIN_SWAMP
        {
            return Some((xtest, ytest));
        }
    }

    None
}

/// The race of the next enemy: for mixed groups the two races of the pair are picked at random
/// (except encounter type 4).
fn encounter_race(game: &mut Game) -> i32 {
    if (game.encounter.mix_flag & MIX_RACE_PAIR) != 0 && game.encounter.kind != KIND_NO_MIX {
        (game.encounter.kind & PAIR_MASK) as i32 + rand2(&mut game.rng)
    } else {
        game.encounter.kind as i32
    }
}

/// Starts dying if actor `i` is out of vitality (and is not already dying or dead): it becomes
/// DYING, the hero gains bravery for a kill, and killing a person (not the lasso keeper) costs
/// 3 kindness. If it is the hero, `dtype` is the death message, luck drops by 5 and the music
/// changes. The hero's stats are always refreshed on screen.
pub fn check_death(game: &mut Game, i: i32, dtype: i32) {
    let actor = ActorRef::new(i as usize);

    if let Some(death) = start_dying(i, &mut game.actors.list[actor.slot()], &mut game.hero) {
        if death.dark_knight {
            speak(game, SPEAK_DARK_KNIGHT_DIES);
        }

        if i == 0 {
            show_event(game, dtype);
            set_mood(game, 1);
        }

        queue_print(game, PRQ_VITALITY);
    }

    if i == 0 {
        queue_print(game, PRQ_STATS);
    }
}

/// What a new death needs shown or said.
#[derive(Debug, PartialEq)]
struct Death {
    dark_knight: bool,
}

/// Starts actor `i` (`actor`) dying if it is out of vitality and not already dying or dead: it
/// goes into the dying state with a countdown. Killing a person (except the lasso keeper) costs
/// the hero kindness; killing anything else gives bravery; the hero's own death costs luck.
/// Returns `None` if nothing happened.
fn start_dying(i: i32, actor: &mut Shape, hero: &mut Hero) -> Option<Death> {
    if actor.vitality >= 1 || actor.state == DYING || actor.state == DEAD {
        return None;
    }

    actor.vitality = 0;
    actor.tactic = DYING_COUNTDOWN;
    actor.goal = DEATH;
    actor.state = DYING;
    let dark_knight = actor.race == RACE_DARK_KNIGHT;

    if !dark_knight && actor.kind == SETFIG && actor.race != RACE_LASSO_KEEPER {
        hero.kindness -= KINDNESS_LOSS;
    }

    if i != 0 {
        hero.brave += 1;
    } else {
        hero.luck -= LUCK_LOSS;
    }

    if hero.kindness < 0 {
        hero.kindness = 0;
    }

    Some(Death { dark_knight })
}

/// Puts carrier `n` (5 turtle, 10 dragon, otherwise the swan) in actor slot 3, loading its sprites
/// if they are not the ones already loaded, in the middle of its extent.
pub fn load_carrier(game: &mut Game, n: i16) {
    let actor = ActorRef::new(FIRST_CARRIER_SLOT);

    game.actor_mut(actor).kind = if n == CARRIER_DRAGON { DRAGON } else { CARRIER };
    let extent = if n == CARRIER_DRAGON {
        2
    } else if n == CARRIER_TURTLE {
        1
    } else {
        0
    };

    if game.actors.loaded_actor_file != n {
        game.memory.nextshape = game.memory.seq_list[SEQ_ENEMY].location;
        load_sprite_file(game, n as i32);
        finish_sprite_load(game, game.actor(actor).kind as i16);
        motor_off();
    }

    let e = game.extents[extent];
    game.actor_mut(actor).abs_x = e.x1.wrapping_add(CARRIER_OFFSET.0);
    game.actor_mut(actor).abs_y = e.y1.wrapping_add(CARRIER_OFFSET.1);
    game.actor_mut(actor).index = 0;
    game.actor_mut(actor).weapon = 0;
    game.actor_mut(actor).environ = 0;
    game.actor_mut(actor).state = STILL;
    game.actor_mut(actor).vitality = VITALITY_CARRIER;
    game.actors.active = ACTORS_WITH_CARRIER;
    game.actors.active_carrier = n;
    game.actors.loaded_actor_file = n;
    game.actor_mut(actor).race = n as u8;
}

/// The actors' sprite file the monsters of this encounter need: sets how many appear and, if that
/// file is not the one loaded, loads it (the carrier is dropped then).
pub fn load_enemy_sprites(game: &mut Game) {
    const FIRST_ENEMY_SLOT: i16 = 3;
    game.encounter.number = (game.extents[game.world.extent].v1 as i32
        + rnd(&mut game.rng, game.extents[game.world.extent].v2 as i32))
        as u8;
    let file = entry(&encounter_chart, game.encounter.kind as usize).file_id as i16;

    if game.actors.loaded_actor_file != file {
        game.actors.loaded_actor_file = file;
        game.actors.active = FIRST_ENEMY_SLOT;
        game.memory.nextshape = game.memory.seq_list[ENEMY as usize].location;
        load_sprite_file(game, game.actors.loaded_actor_file as i32);
        game.actors.loading = 1;
        game.actors.active_carrier = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(vitality: i16, state: i8) -> Shape {
        Shape {
            vitality,
            state,
            ..Default::default()
        }
    }

    fn hero(brave: i16, luck: i16, kindness: i16) -> Hero {
        Hero {
            brave,
            luck,
            kindness,
            ..Default::default()
        }
    }

    #[test]
    fn a_living_actor_does_not_die() {
        let (mut a, mut h) = (actor(1, 0), hero(10, 10, 10));
        assert_eq!(start_dying(5, &mut a, &mut h), None);
        assert_eq!(a.state, 0);
    }

    #[test]
    fn dying_and_dead_actors_are_not_killed_twice() {
        let mut h = hero(10, 10, 10);
        assert_eq!(start_dying(5, &mut actor(0, DYING), &mut h), None);
        assert_eq!(start_dying(5, &mut actor(0, DEAD), &mut h), None);
        assert_eq!(h.brave, 10);
    }

    #[test]
    fn killing_an_enemy_starts_the_death_and_gives_bravery() {
        let (mut a, mut h) = (actor(-4, 0), hero(10, 10, 10));
        assert_eq!(
            start_dying(5, &mut a, &mut h),
            Some(Death { dark_knight: false })
        );

        assert_eq!(
            (a.vitality, a.state, a.goal, a.tactic),
            (0, DYING, DEATH, DYING_COUNTDOWN)
        );

        assert_eq!((h.brave, h.luck, h.kindness), (11, 10, 10));
    }

    #[test]
    fn killing_a_person_costs_kindness_but_not_below_zero() {
        let mut a = actor(0, 0);
        a.kind = SETFIG;
        let mut h = hero(10, 10, 5);
        start_dying(5, &mut a, &mut h);
        assert_eq!(h.kindness, 2);
        let mut a = actor(0, 0);
        a.kind = SETFIG;
        start_dying(5, &mut a, &mut h);
        assert_eq!(h.kindness, 0);
    }

    #[test]
    fn the_lasso_keeper_is_not_a_murder() {
        let mut a = actor(0, 0);
        a.kind = SETFIG;
        a.race = RACE_LASSO_KEEPER;
        let mut h = hero(10, 10, 5);
        start_dying(5, &mut a, &mut h);
        assert_eq!(h.kindness, 5);
    }

    #[test]
    fn the_dark_knight_speaks_and_costs_no_kindness() {
        let mut a = actor(0, 0);
        a.kind = SETFIG;
        a.race = RACE_DARK_KNIGHT;
        let mut h = hero(10, 10, 5);
        assert_eq!(
            start_dying(5, &mut a, &mut h),
            Some(Death { dark_knight: true })
        );

        assert_eq!(h.kindness, 5);
    }

    #[test]
    fn the_heros_own_death_costs_luck_not_bravery() {
        let (mut a, mut h) = (actor(0, 0), hero(10, 20, 10));
        start_dying(0, &mut a, &mut h);
        assert_eq!((h.brave, h.luck), (10, 15));
    }
}
