//! Where encounters and carriers appear: picking a spot, summoning the turtle and centring an extent.

use super::*;

/// Places the turtle: tries up to 25 random spots around the hero for one on water (terrain 5),
/// then moves the turtle extent there and loads the turtle as the carrier. Called when turtle
/// eggs are in view. Leaves the loop count in C's global `i`, as the original did.
pub fn summon_turtle() {
    const TRIES: i32 = 25;
    const WATER: i32 = 5;
    const TURTLE_EXTENT: i16 = 1;
    const TURTLE_CARRIER: i16 = 5;
    unsafe {
        let mut tries_used = 0;
        while tries_used < TRIES {
            pick_encounter_spot();
            if terrain_at(the_encounter.x as i32, the_encounter.y as i32) == WATER {
                break;
            }
            tries_used += 1;
        }
        if tries_used == TRIES {
            return;
        }
        center_extent_on(
            TURTLE_EXTENT,
            the_encounter.x as i16,
            the_encounter.y as i16,
        );
        load_carrier(TURTLE_CARRIER);
    }
}

/// Centres extent `e` on a point (it covers 500 by 400 pixels around it).
pub fn center_extent_on(e: i16, x: i16, y: i16) {
    const HALF_WIDTH: i32 = 250;
    const HALF_HEIGHT: i32 = 200;
    unsafe {
        let extent = &mut *(&raw mut extent_list).cast::<Extent>().add(e as usize);
        extent.x1 = (x as i32 - HALF_WIDTH) as u16;
        extent.y1 = (y as i32 - HALF_HEIGHT) as u16;
        extent.x2 = (x as i32 + HALF_WIDTH) as u16;
        extent.y2 = (y as i32 + HALF_HEIGHT) as u16;
    }
}

// An encounter appears this far away: 150 plus up to 63.
const ENCOUNTER_MIN_DISTANCE: i32 = 150;

/// Picks where a new encounter appears: a random direction and distance from the hero
/// (150 to 213 pixels).
pub fn pick_encounter_spot() {
    unsafe {
        let direction = rand8();
        let distance = ENCOUNTER_MIN_DISTANCE + rand64();
        the_encounter.x = step_x(the_hero.x as i32, direction, distance) as u16;
        the_encounter.y = step_y(the_hero.y as i32, direction, distance) as u16;
    }
}
