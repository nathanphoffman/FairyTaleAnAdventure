//! What blocks an actor's way (terrain or another actor) and taking a step.

use super::hits::{RACE_WRAITH, actor_at};
use super::{DEAD, ENEMY, RAFT_SLOT, RAFT_TYPE};
use crate::game::data::state::the_actors;
use crate::game::world::map::{probe_collision, step_x, step_y};

/// Collision box around an actor.
const BLOCK_HALF_WIDTH: i32 = 11;
const BLOCK_HALF_HEIGHT: i32 = 9;

/// Value `blocked_at` returns when another actor is in the way.
const BLOCKED_BY_ACTOR: i32 = 16;

/// Terrain types that never stop the hero.
const SOFT_TERRAIN: [i32; 2] = [8, 9];

/// What blocks actor `i` at (x, y): a terrain type, 16 for another actor, or 0 if clear.
///
/// Terrain is checked first (wraiths float, so they ignore it; the hero ignores terrain types 8
/// and 9). Then every other actor is checked: it blocks if it is within 10 pixels sideways and
/// 8 pixels up or down.
/// Dead actors and the raft can be walked over.
pub fn blocked_at(x: i16, y: i16, i: i16) -> i32 {
    unsafe {
        let me = actor_at(i as i32);

        if (*me).kind != ENEMY || (*me).race != RACE_WRAITH {
            let mut terrain = probe_collision(x as i32, y as i32);

            if i == 0 && SOFT_TERRAIN.contains(&terrain) {
                terrain = 0;
            }

            if terrain != 0 {
                return terrain;
            }
        }

        for j in 0..the_actors.active as i32 {
            let other = actor_at(j);

            // dead actors and rafts can be walked over
            if i as i32 != j
                && j != RAFT_SLOT as i32
                && (*other).kind != RAFT_TYPE
                && (*other).state != DEAD
            {
                let dx = x as i32 - (*other).abs_x as i32;
                let dy = y as i32 - (*other).abs_y as i32;

                if dx < BLOCK_HALF_WIDTH
                    && dx > -BLOCK_HALF_WIDTH
                    && dy < BLOCK_HALF_HEIGHT
                    && dy > -BLOCK_HALF_HEIGHT
                {
                    return BLOCKED_BY_ACTOR;
                }
            }
        }

        0
    }
}

/// Moves an actor `dist` in direction `dir` if nothing blocks it. Returns TRUE if it moved.
pub fn step_figure(fig: i16, dir: i16, dist: i16) -> i32 {
    unsafe {
        let a = actor_at(fig as i32);
        let xtest = step_x((*a).abs_x as i32, dir as i32, dist as i32) as u16;
        let ytest = step_y((*a).abs_y as i32, dir as i32, dist as i32) as u16;

        if blocked_at(xtest as i16, ytest as i16, fig) != 0 {
            return 0;
        }

        (*a).abs_x = xtest;
        (*a).abs_y = ytest;

        1
    }
}
