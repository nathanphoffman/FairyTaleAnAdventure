//! Putting the hero back at the last safe spot with full health.

use crate::game::data::state::{Hero, World, the_actors, the_hero, the_world};
use crate::game::data::types::Shape;
use crate::game::ui::menus::update_menu_availability;
use crate::game::ui::text::queue_print;
use crate::game::world::regions::load_region_now;

pub(super) const STILL: i8 = 13;

/// Where the hero is on the screen: the map is centred on the hero.
pub(super) const CAMERA_OFFSET: (u16, u16) = (144, 90);

/// Vitality of a brother who sets out: 15 plus a quarter of the bravery.
const BASE_VITALITY: i16 = 15;
const BRAVERY_DIVISOR: i16 = 4;

/// The hero's body is marked with this race.
const HERO_RACE: u8 = 255;

/// Actors active when a brother starts: the hero, the raft and the first person.
const START_ACTORS: i16 = 3;

/// Time of day when a brother starts or revives, and the light then.
pub(super) const START_DAYNIGHT: u16 = 8000;
pub(super) const START_LIGHT: u16 = 300;

/// Print queue entries: vitality and stats.
const PRQ_STATS: i32 = 4;
const PRQ_VITALITY: i32 = 7;

/// The hero (re)appears at the last safe spot, and the region there is loaded.
pub(super) unsafe fn put_hero_at_safe_spot(hero: *mut Shape) {
    unsafe {
        move_to_safe_spot(
            &mut *(&raw mut the_hero),
            &mut *hero,
            &mut *(&raw mut the_world),
        );

        load_region_now();
    }
}

/// The hero's position becomes the safe spot, the camera is centred on it, and its region is the
/// one to load next.
pub(super) fn move_to_safe_spot(hero: &mut Hero, body: &mut Shape, world: &mut World) {
    hero.x = hero.safe_x;
    body.abs_x = hero.safe_x;
    hero.y = hero.safe_y;
    body.abs_y = hero.safe_y;
    world.map_x = hero.x.wrapping_sub(CAMERA_OFFSET.0);
    world.map_y = hero.y.wrapping_sub(CAMERA_OFFSET.1);
    world.new_region = hero.safe_r;
}

/// Fresh health, a new day, and no hunger or tiredness.
pub(super) unsafe fn restore_hero(hero: *mut Shape) {
    unsafe {
        restore_stats(
            &mut *(&raw mut the_hero),
            &mut *hero,
            &mut *(&raw mut the_world),
        );

        the_actors.active = START_ACTORS;
        update_menu_availability();
        queue_print(PRQ_VITALITY);
        queue_print(PRQ_STATS);
    }
}

/// Full health for the bravery, standing still on dry land, the morning clock, and no hunger or
/// fatigue.
pub(super) fn restore_stats(hero: &mut Hero, body: &mut Shape, world: &mut World) {
    body.vitality = BASE_VITALITY + hero.brave / BRAVERY_DIVISOR;
    body.environ = 0;
    body.state = STILL;
    body.race = HERO_RACE;
    world.day_clock = START_DAYNIGHT;
    world.light_level = START_LIGHT;
    hero.hunger = 0;
    hero.fatigue = 0;
}
