//! How near the hero is to the raft, turtle or swan that can carry them (from the main loop in
//! csrc/fmain.c).

use crate::game::data::state::ActorRef;
use crate::game::data::state::{the_actors, the_hero};

// Actor kinds and states (csrc/ftale.h).
const BIRD: i16 = 11;
const TURTLE_ENVIRON: i8 = -2;

/// The hero counts as near the raft or turtle within this many pixels (very near within the
/// smaller one).
const NEAR: i16 = 16;
const VERY_NEAR: i16 = 9;

/// The carrier is the raft (slot 1), or slot 3 when a turtle or bird is out.
const RAFT_SLOT: i16 = 1;
const CARRIER_SLOT: i16 = 3;

/// The hero's centre is a little off the carrier's.
const CARRIER_OFFSET: i32 = 4;

/// 0 when the hero is not near, 1 when near and 2 when very near the carrier (offsets in pixels).
fn nearness(dx: i16, dy: i16) -> i16 {
    if dx < VERY_NEAR && dx > -VERY_NEAR && dy < VERY_NEAR && dy > -VERY_NEAR {
        2
    } else if dx < NEAR && dx > -NEAR && dy < NEAR && dy > -NEAR {
        1
    } else {
        0
    }
}

/// How close the hero is to the raft or turtle (`raft_nearness` 1 = near, 2 = very near). Riding the
/// bird sets the hero's environ.
pub fn raft_proximity() {
    unsafe {
        the_actors.raft_nearness = 0;
        the_actors.turtle_nearness = 0;
        the_actors.carrier_slot = if the_actors.active_carrier != 0 {
            CARRIER_SLOT
        } else {
            RAFT_SLOT
        };

        let carrier = ActorRef::new(the_actors.carrier_slot as usize);
        let mut hero = ActorRef::new(0);
        let dx = (hero.abs_x as i32 - carrier.abs_x as i32 - CARRIER_OFFSET) as i16;
        let dy = (hero.abs_y as i32 - carrier.abs_y as i32 - CARRIER_OFFSET) as i16;
        the_actors.raft_nearness = nearness(dx, dy);

        if the_hero.riding == BIRD {
            hero.environ = TURTLE_ENVIRON;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearness_of_the_carrier() {
        assert_eq!(nearness(0, 0), 2);
        assert_eq!(nearness(VERY_NEAR - 1, 0), 2);
        assert_eq!(nearness(VERY_NEAR, 0), 1);
        assert_eq!(nearness(-(NEAR - 1), NEAR - 1), 1);
        assert_eq!(nearness(NEAR, 0), 0);
        assert_eq!(nearness(0, -NEAR), 0);
    }
}
