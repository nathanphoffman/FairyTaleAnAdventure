//! Putting the visible missiles in the actor list so they get drawn.

use super::missiles::{MISSILE_NONE, MISSILE_ROCK, MISSILE_SPENT};
use super::{ACTOR_SLOTS, MISSILES, OBJECTS};
use crate::game::data::state::{ActorRef, MissileRef};
use crate::game::data::state::{the_actors, the_world};

/// A missile is added if its sprite would be inside this box on the screen.
pub(super) const SCREEN_LEFT: i16 = -20;
pub(super) const SCREEN_RIGHT: i16 = 340;
pub(super) const SCREEN_TOP: i16 = -20;
pub(super) const SCREEN_BOTTOM: i16 = 190;

/// The sprite's centre is this far from its map position.
pub(super) const SPRITE_OFFSET: (i32, i32) = (9, 26);
pub(super) const MISSILE_ROCK_IMAGE: i8 = 0x59;
pub(super) const MISSILE_SPENT_IMAGE: i8 = 0x58;
pub(super) const NO_RACE: u8 = 0xff;

/// Makes the next free actor a flying object that looks like the missile: an arrow pointing the
/// way it flies, a rock, or the spent rock left behind.
unsafe fn add_missile_actor(ms: MissileRef, (xstart, ystart): (i16, i16)) {
    unsafe {
        let mut actor = ActorRef::new(the_actors.drawn as usize);
        actor.abs_x = ms.abs_x;
        actor.abs_y = ms.abs_y;
        actor.rel_x = xstart as u16;
        actor.rel_y = ystart as u16;
        actor.kind = OBJECTS;
        actor.weapon = 0;
        actor.index = ms.direction;
        actor.race = NO_RACE;

        if ms.missile_type == MISSILE_SPENT {
            actor.index = MISSILE_SPENT_IMAGE;
        } else if ms.missile_type == MISSILE_ROCK {
            actor.index = actor.index.wrapping_add(MISSILE_ROCK_IMAGE);
        }
    }
}

/// Adds the visible missiles to the end of the actor list as objects so the drawing draws them.
/// Stops when the list is full.
pub fn queue_missile_sprites() {
    unsafe {
        for i in 0..MISSILES {
            let ms = MissileRef::new(i);
            let t = ms.missile_type;

            if t == MISSILE_NONE {
                continue;
            }

            let xstart = (ms.abs_x as i32 - the_world.map_x as i32 - SPRITE_OFFSET.0) as i16;
            let ystart = (ms.abs_y as i32 - the_world.map_y as i32 - SPRITE_OFFSET.1) as i16;

            if xstart > SCREEN_LEFT
                && xstart < SCREEN_RIGHT
                && ystart > SCREEN_TOP
                && ystart < SCREEN_BOTTOM
            {
                add_missile_actor(ms, (xstart, ystart));
                the_actors.drawn += 1;

                if the_actors.drawn >= ACTOR_SLOTS {
                    return;
                }
            }
        }
    }
}
