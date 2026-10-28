//! The game's main state, gathered into a few structs instead of loose variables.
//!
//! Each struct has one `static mut` instance (`the_hero`, `the_world`, ...).
//! They start out zero apart from the few values listed below.
use crate::game::data::tables::extent_list;
use crate::game::data::types::{Extent, Missile, SeqInfo, Shape};
use crate::game::render::screen::Screen;

/// How many items a brother can carry (the size of the `stuff` list).
/// A brother's item counts: 35 slots that are saved, and one more (the last) that the pick-up code
/// uses to count the arrows in a quiver. (In the C original that last one was the first slot of the
/// next brother's list.)
pub const STUFF_SLOTS: usize = 36;

/// The slots of the item counts that are saved.
pub const SAVED_STUFF_SLOTS: usize = 35;

/// The actors on the map (the hero, the raft, the first person, the carrier, then monsters), and
/// the missiles in flight.
pub const ACTOR_SLOTS: usize = 20;
pub const MISSILE_SLOTS: usize = 6;

/// Sprite sheets loaded at once (hero, objects, ... see `seq_list`).
pub const SPRITE_SHEETS: usize = 7;

/// Entries of the menu slot list.
pub const MENU_SLOTS: usize = 12;

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
    pub julstuff: [u8; STUFF_SLOTS],
    pub kevstuff: [u8; STUFF_SLOTS],
    pub philstuff: [u8; STUFF_SLOTS],
    pub safe_flag: i8,
    pub safe_r: u16,
    pub safe_x: u16,
    pub safe_y: u16,
    pub good_fairy_timer: u8,
}

pub static mut the_hero: Hero = unsafe { std::mem::zeroed() };

impl Hero {
    /// The item counts of the brother who is playing (brothers are numbered from 1).
    pub fn stuff(&self) -> &[u8; STUFF_SLOTS] {
        match self.brother {
            2 => &self.philstuff,
            3 => &self.kevstuff,
            _ => &self.julstuff,
        }
    }

    pub fn stuff_mut(&mut self) -> &mut [u8; STUFF_SLOTS] {
        match self.brother {
            2 => &mut self.philstuff,
            3 => &mut self.kevstuff,
            _ => &mut self.julstuff,
        }
    }
}

/// The hero, which the whole game shares (single threaded; no caller holds on to it across a call
/// that uses it again).
pub fn player() -> &'static mut Hero {
    unsafe { &mut *(&raw mut the_hero) }
}

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
    /// The number of the extent the hero is in (see `current_extent`).
    pub extent: usize,
    pub extent_type: u16,
    /// The map sector at the top left of the view (set when the map is drawn).
    pub sector_x: i32,
    pub sector_y: i32,
    /// How many frames the hero has stood on a bed.
    pub bed_wait: i16,
}

/// The extent (special rectangle of the world) the hero is in.
pub fn current_extent() -> &'static mut Extent {
    unsafe { &mut (*(&raw mut extent_list))[the_world.extent] }
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

/// The actors on the map, the missiles in flight, and who is near the hero.
pub struct Actors {
    pub list: [Shape; ACTOR_SLOTS],
    pub active: i16,
    pub drawn: i16,
    pub missiles: [Missile; MISSILE_SLOTS],
    pub next_missile: i16,
    pub nearest_object: i16,
    pub nearest_person: i16,
    pub last_person: i16,
    pub leader: i16,
    pub draw_order: [u8; ACTOR_SLOTS],
    pub loading: i8,
    pub on_screen: i8,
    pub active_carrier: i16,
    pub carrier_slot: i16,
    pub raft_nearness: i16,
    pub turtle_nearness: i16,
    pub nearest_person_distance: i16,
    pub loaded_actor_file: i16,
    pub loaded_setfig_file: i16,
    pub battle_flag: i8,
    pub frustration: i8,
    pub fiery_death: i8,
}

pub static mut the_actors: Actors = unsafe { std::mem::zeroed() };

/// A way to reach one actor of `the_actors.list` by its number, so that code can hold on to "that
/// actor" while it calls others that change the list. Each use looks the actor up again.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ActorRef(usize);

impl ActorRef {
    pub fn new(index: usize) -> ActorRef {
        ActorRef(index)
    }
}

impl std::ops::Deref for ActorRef {
    type Target = Shape;

    fn deref(&self) -> &Shape {
        unsafe { &(*(&raw const the_actors.list))[self.0] }
    }
}

impl std::ops::DerefMut for ActorRef {
    fn deref_mut(&mut self) -> &mut Shape {
        unsafe { &mut (*(&raw mut the_actors.list))[self.0] }
    }
}

/// The same for the missiles in flight.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MissileRef(usize);

impl MissileRef {
    pub fn new(index: usize) -> MissileRef {
        MissileRef(index)
    }
}

impl std::ops::Deref for MissileRef {
    type Target = Missile;

    fn deref(&self) -> &Missile {
        unsafe { &(*(&raw const the_actors.missiles))[self.0] }
    }
}

impl std::ops::DerefMut for MissileRef {
    fn deref_mut(&mut self) -> &mut Missile {
        unsafe { &mut (*(&raw mut the_actors.missiles))[self.0] }
    }
}

/// The screen: bitmaps, display pages, pens and view ports, and what is shown.
/// The big memory areas the game loads its data into.
pub struct Memory {
    /// The tile pictures: five planes of 256 tiles for each of the four image banks.
    pub image: Vec<u8>,
    /// The 256 sectors of 128 tile numbers each, then (from `MAP_OFFSET`) the region map.
    pub sector: Vec<u8>,
    /// The terrain table: four bytes per tile.
    pub terrain: Vec<u8>,
    /// The masks that hide sprites behind scenery tiles.
    pub shadow: Vec<u8>,
    /// The sprite sheets: each sheet is its frames (five planes each) followed by their masks.
    pub shape: Vec<u8>,
    /// Where the image and the mask of the sprite being drawn start in `shape`.
    pub shapedata: usize,
    pub mask_buffer: usize,
    /// Where the next sprite sheet loads in `shape`.
    pub nextshape: usize,
    pub seq_list: [SeqInfo; SPRITE_SHEETS],
}

/// Where the region map starts in `Memory::sector`.
pub const MAP_OFFSET: usize = 128 * 256;

impl Memory {
    pub const fn new() -> Memory {
        Memory {
            image: Vec::new(),
            sector: Vec::new(),
            terrain: Vec::new(),
            shadow: Vec::new(),
            shape: Vec::new(),
            shapedata: 0,
            mask_buffer: 0,
            nextshape: 0,
            seq_list: [const { SeqInfo::EMPTY }; SPRITE_SHEETS],
        }
    }

    /// The sector number at byte `index` of the region map (0 outside it).
    pub fn map_byte(&self, index: usize) -> u8 {
        self.sector.get(MAP_OFFSET + index).copied().unwrap_or(0)
    }

    /// Changes the sector number at byte `index` of the region map.
    pub fn set_map_byte(&mut self, index: usize, value: u8) {
        if let Some(byte) = self.sector.get_mut(MAP_OFFSET + index) {
            *byte = value;
        }
    }

    /// The tile number at byte `index` of the sector table (0 outside it).
    pub fn sector_byte(&self, index: usize) -> u8 {
        self.sector.get(index).copied().unwrap_or(0)
    }

    /// Changes the tile number at byte `index` of the sector table.
    pub fn set_sector_byte(&mut self, index: usize, value: u8) {
        if let Some(byte) = self.sector.get_mut(index) {
            *byte = value;
        }
    }

    /// Byte `index` of the terrain table (0 outside it).
    pub fn terrain_byte(&self, index: usize) -> u8 {
        self.terrain.get(index).copied().unwrap_or(0)
    }
}

/// The menus, the keyboard and mouse state, and test and debug switches.
pub struct Ui {
    pub current_menu: i16,
    pub chosen_entry: i8,
    pub menu_hit_enabled: i8,
    pub held_direction: i16,
    pub fight_key_held: i16,
    pub cheat_enabled: i16,
    pub quit_requested: i8,
    /// The file menu is for saving (else for loading).
    pub saving: bool,
    pub slot_entries: [i8; MENU_SLOTS],
}

/// The witch and her spinning beam.
pub struct Witch {
    pub direction: i8,
    pub on_screen: i8,
    pub angle: u8,
}

/// Everything that changes while the game runs, in one value that is passed down from `run_game`.
/// The pieces move in from the `static mut` variables one at a time; what is still a static is
/// listed in modern_engine_plan.md.
pub struct Game {
    pub witch: Witch,
    pub ui: Ui,
    pub encounter: Encounters,
    pub memory: Memory,
    pub screen: Screen,
}

impl Game {
    pub fn new() -> Game {
        Game {
            witch: Witch {
                direction: 0,
                on_screen: 0,
                angle: 0,
            },
            ui: unsafe { std::mem::zeroed() },
            encounter: unsafe { std::mem::zeroed() },
            screen: Screen::new(),
            memory: Memory::new(),
        }
    }
}
