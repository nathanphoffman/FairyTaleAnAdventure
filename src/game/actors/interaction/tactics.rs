//! Carrying out an enemy's tactic: pursuing, following, evading, wandering, shooting or backing up.

use super::hits::actor_at;
use super::{
    ATTACK2, BACKUP, BUMBLE_SEEK, EGG_SEEK, EVADE, FOLLOW, HERO, PURSUE, RANDOM, SHOOT, SHOOT1,
    WALKING,
};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;

use crate::game::actors::steering::steer_actor;
use crate::game::data::random::fta_rand;

// How an enemy shooting at the hero lines up.
const SHOOT_CLOSE: i16 = 8;
const SHOOT_DIAGONAL_BELOW: i16 = 5;
const SHOOT_DIAGONAL_ABOVE: i16 = 7;

// `steer_actor` modes.
const COURSE_TOWARD: i32 = 0;
const COURSE_AWAY: i32 = 3;
const COURSE_STRAIGHT: i32 = 4;
const COURSE_TURN_ONLY: i32 = 5;
const COURSE_WOBBLE_CLOSE: i32 = 2;

/// An actor picks a new heading one frame in 8 (one in 4 for those that attack harder).
const ACTS_ONE_IN_8: i32 = 7;
const ACTS_ONE_IN_4: i32 = 3;
const FACING_MASK: i32 = 7;

/// The followers aim this far below the one they follow.
const FOLLOW_OFFSET: i32 = 20;

/// Where snakes go to find the eggs.
const EGG_X: i32 = 23087;
const EGG_Y: i32 = 5667;

/// Runs one step of enemy `i`'s tactic. Most tactics only act on some ticks (1 in 8, or 1 in 4
/// for clever enemies), which makes them look less robotic.
///
/// - PURSUE: head toward the hero.  - SHOOT: line up with the hero (same row, column or
///   diagonal) and aim, else approach.  - RANDOM: wander.  - BUMBLE_SEEK: head toward the hero
///   without smart steering.  - BACKUP: head away from the hero.  - FOLLOW: stay near the leader.
/// - EVADE: keep away from another enemy.  - EGG_SEEK: go to the turtle eggs.
pub fn run_tactic(game: &mut Game, i: i32, tactic: i32) {
    let mut acts_now = fta_rand(&mut game.rng) & ACTS_ONE_IN_8 == 0;
    let actor = actor_at(i);
    game.actor_mut(actor).tactic = tactic as i8;

    if game.actor(actor).goal == ATTACK2 {
        acts_now = fta_rand(&mut game.rng) & ACTS_ONE_IN_4 == 0;
    }

    let (hx, hy) = (game.hero.x as i32, game.hero.y as i32);

    match tactic {
        PURSUE => steer_if(game, acts_now, i, hx, hy, COURSE_TOWARD),
        SHOOT => engage_target(game, i, actor, hx, hy),
        RANDOM => wander(game, actor, acts_now),
        BUMBLE_SEEK => steer_if(game, acts_now, i, hx, hy, COURSE_STRAIGHT),
        BACKUP => steer_if(game, acts_now, i, hx, hy, COURSE_AWAY),
        FOLLOW => follow_leader(game, i, actor, acts_now),
        EVADE => evade_neighbour(game, i, acts_now),
        EGG_SEEK => {
            steer_if(game, acts_now, i, EGG_X, EGG_Y, COURSE_TOWARD);
            game.actor_mut(actor).state = WALKING;
        }

        _ => {}
    }
}

/// Steers actor `i` toward a point (or however `mode` says) if this is one of its active ticks.
fn steer_if(game: &mut Game, acts_now: bool, i: i32, x: i32, y: i32, mode: i32) {
    if acts_now {
        steer_actor(game, i, x, y, mode);
    }
}

/// An archer lines up with the hero (same row, column or diagonal) and aims, otherwise approaches.
fn engage_target(game: &mut Game, i: i32, actor: ActorRef, hx: i32, hy: i32) {
    let hero = actor_at(HERO as i32);
    let xd = (game.actor(actor).abs_x as i32 - game.actor(hero).abs_x as i32) as i16;
    let yd = (game.actor(actor).abs_y as i32 - game.actor(hero).abs_y as i32) as i16;
    let (xd, yd) = (xd.wrapping_abs(), yd.wrapping_abs());
    let lined_up = xd < SHOOT_CLOSE
        || yd < SHOOT_CLOSE
        || (xd as i32 > yd as i32 - SHOOT_DIAGONAL_BELOW as i32
            && (xd as i32) < yd as i32 + SHOOT_DIAGONAL_ABOVE as i32);

    if fta_rand(&mut game.rng) & 1 != 0 && lined_up {
        steer_actor(game, i, hx, hy, COURSE_TURN_ONLY);

        if game.actor(actor).state < SHOOT1 {
            game.actor_mut(actor).state = SHOOT1;
        }
    } else {
        steer_actor(game, i, hx, hy, COURSE_TOWARD);
    }
}

/// Wander: now and then turn a random way, and keep walking.
fn wander(game: &mut Game, actor: ActorRef, acts_now: bool) {
    if acts_now {
        game.actor_mut(actor).facing = (fta_rand(&mut game.rng) & FACING_MASK) as i8;
    }

    game.actor_mut(actor).state = WALKING;
}

/// Stay near the leader (an enemy cannot follow itself, so it wanders instead).
fn follow_leader(game: &mut Game, i: i32, actor: ActorRef, acts_now: bool) {
    let leader_slot = game.actors.leader as i32;

    if i == leader_slot {
        game.actor_mut(actor).tactic = RANDOM as i8; // can't follow self
    }

    if acts_now {
        steer_toward_actor(game, i, leader_slot, COURSE_TOWARD);
    }
}

/// Keep away from another enemy (the one before the last, or the one at twice this slot number).
fn evade_neighbour(game: &mut Game, i: i32, acts_now: bool) {
    let other = if i == game.actors.active as i32 {
        i - 1
    } else {
        i + i
    };

    if acts_now {
        steer_toward_actor(game, i, other, COURSE_WOBBLE_CLOSE);
    }
}

/// Steers actor `i` toward (a little below) actor `target`.
fn steer_toward_actor(game: &mut Game, i: i32, target: i32, mode: i32) {
    let target = actor_at(target);
    steer_actor(
        game,
        i,
        game.actor(target).abs_x as i32,
        game.actor(target).abs_y as i32 + FOLLOW_OFFSET,
        mode,
    );
}
