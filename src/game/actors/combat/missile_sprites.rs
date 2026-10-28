//! Putting the visible missiles in the actor list so they get drawn.

use super::missiles::{MISSILE_NONE, MISSILE_ROCK, MISSILE_SPENT};
use super::{ACTOR_SLOTS, MISSILES, OBJECTS};
use crate::game::data::state::Game;
use crate::game::data::state::{ActorRef, MissileRef};

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
fn add_missile_actor(game: &mut Game, ms: MissileRef, (xstart, ystart): (i16, i16)) {
    let actor = ActorRef::new(game.actors.drawn as usize);
    game.actors.list[actor.slot()].abs_x = game.actors.missiles[ms.slot()].abs_x;
    game.actors.list[actor.slot()].abs_y = game.actors.missiles[ms.slot()].abs_y;
    game.actors.list[actor.slot()].rel_x = xstart as u16;
    game.actors.list[actor.slot()].rel_y = ystart as u16;
    game.actors.list[actor.slot()].kind = OBJECTS;
    game.actors.list[actor.slot()].weapon = 0;
    game.actors.list[actor.slot()].index = game.actors.missiles[ms.slot()].direction;
    game.actors.list[actor.slot()].race = NO_RACE;

    if game.actors.missiles[ms.slot()].missile_type == MISSILE_SPENT {
        game.actors.list[actor.slot()].index = MISSILE_SPENT_IMAGE;
    } else if game.actors.missiles[ms.slot()].missile_type == MISSILE_ROCK {
        game.actors.list[actor.slot()].index = game.actors.list[actor.slot()]
            .index
            .wrapping_add(MISSILE_ROCK_IMAGE);
    }
}

/// Adds the visible missiles to the end of the actor list as objects so the drawing draws them.
/// Stops when the list is full.
pub fn queue_missile_sprites(game: &mut Game) {
    for i in 0..MISSILES {
        let ms = MissileRef::new(i);
        let t = game.actors.missiles[ms.slot()].missile_type;

        if t == MISSILE_NONE {
            continue;
        }

        let xstart = (game.actors.missiles[ms.slot()].abs_x as i32
            - game.world.map_x as i32
            - SPRITE_OFFSET.0) as i16;
        let ystart = (game.actors.missiles[ms.slot()].abs_y as i32
            - game.world.map_y as i32
            - SPRITE_OFFSET.1) as i16;

        if xstart > SCREEN_LEFT
            && xstart < SCREEN_RIGHT
            && ystart > SCREEN_TOP
            && ystart < SCREEN_BOTTOM
        {
            add_missile_actor(game, ms, (xstart, ystart));
            game.actors.drawn += 1;

            if game.actors.drawn >= ACTOR_SLOTS {
                return;
            }
        }
    }
}
