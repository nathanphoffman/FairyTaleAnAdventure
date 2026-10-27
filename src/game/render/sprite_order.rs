//! The order the actors are drawn in, back to front (from the main loop in csrc/fmain.c), and who
//! is the nearest person.

use crate::game::actors::interaction::actor_distance;
use crate::game::data::state::{the_actors, the_hero};

// Actor kinds and states (csrc/ftale.h).
const OBJECTS: i8 = 1;
const DEAD: i8 = 15;
const BIRD: i16 = 11;

/// Closest a person can be to count as `nearest_person`.
const PERSON_RANGE: i16 = 50;
/// Dead actors, the hero on a mount and the raft are drawn this much lower in the order.
const DRAW_LOWER: i16 = 32;
const WADING: i8 = 25;
const RAFT_ACTOR: u8 = 1;

/// Finds the nearest person, and sorts `draw_order` by y so that actors lower on the screen are
/// drawn later.
pub fn sort_actors() {
    unsafe {
        the_actors.nearest_person = 0;
        the_actors.nearest_person_distance = PERSON_RANGE;
        for i in 0..the_actors.drawn as usize {
            the_actors.draw_order[i] = i as u8;
        }
        for i in 0..the_actors.drawn as usize {
            let an = &raw const the_actors.list[i];
            if i != 0 && (*an).kind != OBJECTS && (*an).state != DEAD && the_hero.riding != BIRD {
                let dist = actor_distance(i as i32, 0) as i16;
                if dist < the_actors.nearest_person_distance {
                    the_actors.nearest_person_distance = dist;
                    the_actors.nearest_person = i as i16;
                }
            }
            for j in 1..the_actors.drawn as usize {
                bubble_step(j);
            }
        }
    }
}

/// One compare-and-swap of neighbours `j - 1` and `j` in the draw order.
unsafe fn bubble_step(j: usize) {
    unsafe {
        let k1 = the_actors.draw_order[j - 1];
        let k2 = the_actors.draw_order[j];
        let a = &raw const the_actors.list[k1 as usize];
        let b = &raw const the_actors.list[k2 as usize];
        let mut y1 = (*a).abs_y as i16;
        let mut y2 = (*b).abs_y as i16;
        if (*a).state == DEAD || (k2 == 0 && the_hero.riding != 0) || k1 == RAFT_ACTOR {
            y1 = y1.wrapping_sub(DRAW_LOWER);
        }
        if (*b).state == DEAD || (k1 == 0 && the_hero.riding != 0) || k2 == RAFT_ACTOR {
            y2 = y2.wrapping_sub(DRAW_LOWER);
        }
        if (*a).environ > WADING {
            y1 = y1.wrapping_add(DRAW_LOWER);
        }
        if (*b).environ > WADING {
            y2 = y2.wrapping_add(DRAW_LOWER);
        }
        if y2 < y1 {
            the_actors.draw_order[j - 1] = k2;
            the_actors.draw_order[j] = k1;
        }
    }
}
