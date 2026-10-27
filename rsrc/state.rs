//! The game's main state, gathered into a few structs instead of loose variables.
//!
//! Each struct has one `static mut` instance (`the_hero`, `the_world`, ...).
//! They start out zero apart from the few values listed below.
use super::types::{Extent, Missile, Shape};

/// The hero: where they are, their stats, and what they carry or ride.
pub struct Hero {
    pub x: u16,
    pub y: u16,
    pub sector: i16,
    pub place: u16,
    pub brave: i16,
    pub luck: i16,
    pub kindness: i16,
    pub wealth: i16,
    pub hunger: i16,
    pub fatigue: i16,
    pub riding: i16,
    pub flying: i16,
    pub brother: i16,
    pub stuff: *mut u8,
}

pub static mut the_hero: Hero = unsafe { std::mem::zeroed() };

/// The world around the hero: the map position, region, clock, light and magic timers.
pub struct World {
    pub map_x: u16,
    pub map_y: u16,
    pub region: u16,
    pub new_region: u16,
    pub target_region: u16,
    pub origin_x: u16,
    pub origin_y: u16,
    pub tile_x: u16,
    pub tile_y: u16,
    pub day_clock: u16,
    pub day_period: i16,
    pub light_level: u16,
    pub light_timer: i16,
    pub freeze_timer: i16,
    pub secret_timer: i16,
    pub extent: *mut Extent,
    pub extent_type: u16,
}

pub static mut the_world: World = World {
    region: 3,
    ..unsafe { std::mem::zeroed() }
};

/// The monster encounter being set up and how dangerous the place is.
pub struct Encounters {
    pub number: u8,
    pub kind: u16,
    pub x: u16,
    pub y: u16,
    pub danger_level: u8,
    pub mix_flag: i16,
    pub weapon_type: i16,
}

pub static mut the_encounter: Encounters = unsafe { std::mem::zeroed() };

/// The actors on the map, the missiles in flight, and who is near the hero.
pub struct Actors {
    pub list: [Shape; 20],
    pub active: i16,
    pub drawn: i16,
    pub missiles: [Missile; 6],
    pub next_missile: i16,
    pub nearest_object: i16,
    pub nearest_person: i16,
    pub last_person: i16,
    pub leader: i16,
}

pub static mut the_actors: Actors = unsafe { std::mem::zeroed() };
