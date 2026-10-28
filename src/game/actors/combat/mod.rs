//! Fighting each frame (from the main loop in csrc/fmain.c): swings and bites, arrows and
//! fireballs in flight, and putting the visible missiles in the actor list so they get drawn.

mod melee;

mod missile_sprites;

mod missiles;

pub use self::melee::resolve_melee;
pub use self::missile_sprites::queue_missile_sprites;
pub use self::missiles::advance_missiles;

use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;

// Actor kinds and states (csrc/ftale.h).
const OBJECTS: i8 = 1;
const CARRIER: i8 = 5;
const WALKING: i8 = 12;
const DEAD: i8 = 15;
const RAFT: usize = 1;

/// Slot count of the actor list, and of the missile list.
const ACTOR_SLOTS: i16 = 20;
const MISSILES: usize = 6;

/// Distance between two points as the larger of the x and y differences.
fn chebyshev(dx: i16, dy: i16) -> i16 {
    dx.wrapping_abs().max(dy.wrapping_abs())
}

/// How far actor `target` is from the point (`x`, `y`), by `chebyshev` (the differences wrap in 16
/// bits, as in the original).
fn distance_to_point(game: &mut Game, target: ActorRef, x: i16, y: i16) -> i16 {
    let xd = (game.actors.list[target.slot()].abs_x as i32 - x as i32) as i16;
    let yd = (game.actors.list[target.slot()].abs_y as i32 - y as i32) as i16;

    chebyshev(xd, yd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_is_the_larger_axis() {
        assert_eq!(chebyshev(3, -5), 5);
        assert_eq!(chebyshev(-7, 2), 7);
        assert_eq!(chebyshev(0, 0), 0);
    }
}
