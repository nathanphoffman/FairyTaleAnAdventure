//! Looking around: revealing hidden objects within reach.

use super::*;

/// `Look` finds hidden objects closer than this.
const LOOK_RANGE: i32 = 40;
const EVENT_NOTHING_HIDDEN: i32 = 20;
const EVENT_FOUND_HIDDEN: i32 = 38;

/// Reveals hidden objects within reach.
pub(super) unsafe fn look_around() {
    unsafe {
        let mut found = false;
        for i in 0..the_actors.drawn as i32 {
            let an = &raw const the_actors.list[i as usize];
            if (*an).kind == OBJECTS && (*an).race == 0 && actor_distance(i, 0) < LOOK_RANGE {
                found = true;
                change_object(i, 1);
            }
        }
        show_event(if found {
            EVENT_FOUND_HIDDEN
        } else {
            EVENT_NOTHING_HIDDEN
        });
    }
}
