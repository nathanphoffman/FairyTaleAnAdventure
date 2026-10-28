//! How actors affect each other and the world (from csrc/fmain2.c): hits (`hits`), what blocks a step
//! (`blocking`), the nearest figure (`nearest`), carrying out enemy tactics (`tactics`), the princess's
//! rescue (`rescue`) and where encounters and carriers appear (`extents`).
//!
//! Actors live in `actor_list`: slot 0 is the hero, slot 1 the raft (or other carrier), slots 3 and
//! up the monsters, and people and objects after them. Positions are in world pixels. Directions
//! are numbered 0-7 clockwise from north-west (0 NW, 1 N, 2 NE, 3 E, 4 SE, 5 S, 6 SW, 7 W).

mod blocking;

mod extents;

mod hits;

mod nearest;

mod rescue;

mod tactics;

pub use self::blocking::blocked_at;
pub use self::extents::{center_extent_on, pick_encounter_spot, summon_turtle};
pub use self::hits::{hit_actor, report_battle_result};
pub use self::nearest::{actor_distance, find_nearest_figure};
pub use self::rescue::rescue_princess;
pub use self::tactics::run_tactic;

use crate::game::data::state::Game;
use crate::game::output::Output;
use crate::game::ui::text::show_event;

// Actor kinds (csrc/ftale.h `enum sequences`, used for the `type` field).
const OBJECTS: i8 = 1;
const ENEMY: i8 = 2;
const SETFIG: i8 = 4;

/// Rafts and other carriers do not block movement.
const RAFT_TYPE: i8 = 5;
const DRAGON: i8 = 6;

// Actor states.
const WALKING: i8 = 12;
const DEAD: i8 = 15;
const SHOOT1: i8 = 24;

// Goals.
const ATTACK2: i8 = 2;
const FLEE: i8 = 5;

// Tactics.
const PURSUE: i32 = 1;
const FOLLOW: i32 = 2;
const BUMBLE_SEEK: i32 = 3;
const RANDOM: i32 = 4;
const BACKUP: i32 = 5;
const EVADE: i32 = 6;
const SHOOT: i32 = 8;
const EGG_SEEK: i32 = 10;
const HERO: usize = 0;
const RAFT_SLOT: usize = 1;

/// Eats: lowers hunger by `amt`. If that would go below zero the hero is simply full
/// (message 13); otherwise "Yum!".
pub fn eat_food(game: &mut Game, out: &mut dyn Output, amt: i32) {
    const HUNGER_FULL_MESSAGE: i32 = 13;
    game.hero.hunger = (game.hero.hunger as i32 - amt) as i16;

    if game.hero.hunger < 0 {
        game.hero.hunger = 0;
        show_event(game, out, HUNGER_FULL_MESSAGE);
    } else {
        out.print("Yum!");
    }
}

#[cfg(test)]
mod tests {

    use crate::game::data::types::Extent;

    #[test]
    fn struct_sizes() {
        assert_eq!(size_of::<Extent>(), 12);
    }
}
