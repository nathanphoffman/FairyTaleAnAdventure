//! Arrows and fireballs in flight: moving, hitting actors and stopping at walls.

use super::{CARRIER, DEAD, MISSILES, RAFT, distance_to_point};
use crate::game::actors::interaction::hit_actor;
use crate::game::data::random::{bitrand, rand8};
use crate::game::data::state::Actors;
use crate::game::data::state::Game;
use crate::game::data::state::{ActorRef, MissileRef};
use crate::game::data::types::Missile;
use crate::game::output::Output;
use crate::game::world::map::{step_x, step_y, terrain_at};

/// Whether a missile at `dist` from a target hits it: within `range`, and for the hero (only for the
/// first missile slot, as in the original) a roll above the hero's bravery is needed. The roll is
/// only drawn when it is needed.
pub(super) fn missile_connects(
    slot: usize,
    dist: i16,
    range: i16,
    dodge: i16,
    roll: impl FnOnce() -> i32,
) -> bool {
    (slot != 0 || roll() > dodge as i32) && dist < range
}

pub(super) const MISSILE_NONE: i8 = 0;
pub(super) const MISSILE_ROCK: i8 = 2;
pub(super) const MISSILE_SPENT: i8 = 3;

/// A missile gives up after this many frames in flight.
pub(super) const MAX_FLIGHT: i8 = 40;

/// Terrain a missile stops at: solid wall and the map edge.
pub(super) const WALL_TERRAIN: [i32; 2] = [1, 15];

/// How close a missile has to be to hit (arrows and rocks, fireballs).
pub(super) const ARROW_HIT_RANGE: i16 = 6;
pub(super) const FIREBALL_HIT_RANGE: i16 = 9;

/// Other actors are treated as having this much bravery when it comes to dodging.
pub(super) const OTHER_DODGE: i16 = 20;
pub(super) const DODGE_RANGE: i32 = 512;
pub(super) const MISSILE_DAMAGE_BASE: i32 = 4;

/// `hit_actor` attacker codes for an arrow and for a fireball.
pub(super) const BY_ARROW: i32 = -1;
pub(super) const BY_FIREBALL: i32 = -2;

/// Moves the six arrows and fireballs; they hit actors in the way and stop at walls. Nothing
/// moves while time is frozen.
pub fn advance_missiles(game: &mut Game, out: &mut dyn Output) {
    if game.world.freeze_timer != 0 {
        return;
    }

    for i in 0..MISSILES {
        let ms = MissileRef::new(i);
        fly(game, out, i, ms);
    }
}

/// One missile's flight for this frame.
pub(super) fn fly(game: &mut Game, out: &mut dyn Output, i: usize, ms: MissileRef) {
    let mut s = game.missile(ms).speed as i16 * 2;

    if missile_expired(game.missile_mut(ms), s) {
        game.missile_mut(ms).missile_type = MISSILE_NONE;

        return;
    }

    let xs = game.missile(ms).abs_x as i16;
    let ys = game.missile(ms).abs_y as i16;

    if WALL_TERRAIN.contains(&terrain_at(game, xs as i32, ys as i32)) {
        game.missile_mut(ms).missile_type = MISSILE_NONE;
        s = 0;
    }

    let fc = game.missile(ms).direction as i32;
    hit_first_actor_in_path(game, out, i, ms, xs, ys);
    game.missile_mut(ms).abs_x = step_x(game.missile(ms).abs_x as i32, fc, s as i32) as u16;
    game.missile_mut(ms).abs_y = step_y(game.missile(ms).abs_y as i32, fc, s as i32) as u16;
}

/// Whether the missile is unused, spent, standing still (`s` is its doubled speed) or has been in
/// the air too long. Counts one more tick of flight when it has not expired.
pub(super) fn missile_expired(ms: &mut Missile, s: i16) -> bool {
    let flight = ms.time_of_flight;

    if ms.missile_type == MISSILE_NONE || ms.missile_type == MISSILE_SPENT || s == 0 {
        return true;
    }

    ms.time_of_flight = flight.wrapping_add(1);

    flight > MAX_FLIGHT
}

/// Hits the first actor close enough to the missile at (`xs`, `ys`) and stops the missile.
/// `i` is the missile's own index; the hero (actor 0) gets a dodge chance from `brave`, others a
/// fixed one.
pub(super) fn hit_first_actor_in_path(
    game: &mut Game,
    out: &mut dyn Output,
    i: usize,
    ms: MissileRef,
    xs: i16,
    ys: i16,
) {
    let fc = game.missile(ms).direction as i32;
    let is_rock = game.missile(ms).missile_type == MISSILE_ROCK;
    let range = if is_rock {
        FIREBALL_HIT_RANGE
    } else {
        ARROW_HIT_RANGE
    };

    let mut j = 0;

    while j < actor_count(&game.actors) {
        let dodge = if j == 0 { game.hero.brave } else { OTHER_DODGE };
        let target = j;
        j += 1;

        if !missile_can_hit(&game.actors, ms, target) {
            continue;
        }

        let dist = distance_to_point(&game.actors, ActorRef::new(target), xs, ys);

        // the original tests the missile's own index here, not the target's
        if missile_connects(i, dist, range, dodge, || {
            bitrand(&mut game.rng, DODGE_RANGE)
        }) {
            let by = if is_rock { BY_FIREBALL } else { BY_ARROW };
            let damage = (rand8(&mut game.rng) + MISSILE_DAMAGE_BASE) as i16;
            hit_actor(game, out, by, target as i32, fc, damage);

            game.missile_mut(ms).speed = 0;

            if is_rock {
                game.missile_mut(ms).missile_type = MISSILE_SPENT;
            }

            return;
        }
    }
}

/// A missile cannot hit the raft, its own archer, the dead or a carrier.
fn missile_can_hit(actors: &Actors, ms: MissileRef, target: usize) -> bool {
    let actor = ActorRef::new(target);

    !(target == RAFT
        || actors.missile(ms).archer as i16 == target as i16
        || actors.actor(actor).state == DEAD
        || actors.actor(actor).kind == CARRIER)
}

/// The current number of actors (the loop bound is read again each time, as in the original).
pub fn actor_count(actors: &Actors) -> usize {
    actors.active as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::types::Missile;

    fn missile(kind: i8, speed: i8, time_of_flight: i8) -> Missile {
        Missile {
            missile_type: kind,
            speed,
            time_of_flight,
            ..Default::default()
        }
    }

    #[test]
    fn unused_spent_and_stopped_missiles_expire() {
        assert!(missile_expired(&mut missile(MISSILE_NONE, 5, 0), 10));
        assert!(missile_expired(&mut missile(MISSILE_SPENT, 5, 0), 10));
        assert!(missile_expired(&mut missile(MISSILE_ROCK, 0, 0), 0));
    }

    #[test]
    fn missile_counts_flight_time_until_it_runs_out() {
        let mut ms = missile(MISSILE_ROCK, 5, 0);
        assert!(!missile_expired(&mut ms, 10));
        assert_eq!(ms.time_of_flight, 1);
        let mut old = missile(MISSILE_ROCK, 5, MAX_FLIGHT + 1);
        assert!(missile_expired(&mut old, 10));
    }

    #[test]
    fn missiles_roll_only_for_the_first_slot() {
        let never = || -> i32 { unreachable!("only slot 0 rolls") };
        assert!(missile_connects(3, 5, 6, 100, never));
        assert!(!missile_connects(3, 6, 6, 100, never));
        assert!(missile_connects(0, 5, 6, 100, || 101));
        assert!(!missile_connects(0, 5, 6, 100, || 100));
    }
}
