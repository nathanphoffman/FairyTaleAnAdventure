//! The game's main state, gathered into a few structs instead of loose variables.
//!
//! Each struct has one `static mut` instance (`the_hero`, `the_world`, ...).
//! They start out zero apart from the few values listed below.
use crate::amiga::{BitMap, RasInfo, RastPort, View, ViewPort};
use crate::game::data::types::{Extent, FPage, Missile, SeqInfo, Shape};
use crate::game::input::handler::InWork;

/// How many items a brother can carry (the size of the `stuff` list).
pub const STUFF_SLOTS: usize = 35;

/// The actors on the map (the hero, the raft, the first person, the carrier, then monsters), and
/// the missiles in flight.
pub const ACTOR_SLOTS: usize = 20;
pub const MISSILE_SLOTS: usize = 6;

/// Sound effects loaded from disk, and score tracks (4 voices each).
pub const SAMPLE_SLOTS: usize = 6;
pub const TRACK_SLOTS: usize = 32;

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
    pub stuff: *mut u8,
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
    /// The map sector at the top left of the view (set when the map is drawn).
    pub sector_x: i32,
    pub sector_y: i32,
    /// How many frames the hero has stood on a bed.
    pub bed_wait: i16,
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

/// The screen: bitmaps, display pages, rastports and view ports, and what is shown.
pub struct Screen {
    pub bm_draw: *mut BitMap,
    pub bm_lim: *mut BitMap,
    pub bm_page1: *mut BitMap,
    pub bm_page2: *mut BitMap,
    pub bm_scroll: BitMap,
    pub bm_source: *mut BitMap,
    pub bm_text: *mut BitMap,
    pub pagea: BitMap,
    pub pageb: BitMap,
    pub fp_page1: FPage,
    pub fp_page2: FPage,
    pub drawing_page: *mut FPage,
    pub viewing_page: *mut FPage,
    pub ri_page1: RasInfo,
    pub ri_page2: RasInfo,
    pub ri_text: RasInfo,
    pub rp_map: RastPort,
    pub rp_text: RastPort,
    pub rp_text2: RastPort,
    pub vp_page: ViewPort,
    pub vp_text: ViewPort,
    pub work_bm: BitMap,
    pub old_view: *mut View,
    pub current_rastport: *mut RastPort,
    pub current_viewport: *mut ViewPort,
    pub view_status: i8,
    pub planes: *mut *mut u8,
    pub planesize: i16,
    pub wb_bmap: *mut BitMap,
    pub flasher: i8,
}

pub static mut the_screen: Screen = unsafe { std::mem::zeroed() };

/// The big memory areas the game loads its data into.
pub struct Memory {
    pub image_mem: *mut u8,
    pub map_mem: *mut u8,
    pub sector_mem: *mut u8,
    pub terrain_mem: *mut u8,
    pub shadow_mem: *mut u8,
    pub shape_mem: *mut u8,
    pub shapedata: *mut u8,
    pub sprite_data: *mut u8,
    pub mask_buffer: *mut u8,
    pub bmask_mem: *mut u8,
    pub queue_mem: *mut u8,
    pub scoremem: *mut u8,
    pub sample_mem: *mut u8,
    pub wavmem: *mut u8,
    pub volmem: *mut u8,
    pub sample: [*mut u8; SAMPLE_SLOTS],
    pub sample_size: [u32; SAMPLE_SLOTS],
    pub track: [*mut u8; TRACK_SLOTS],
    pub nextshape: *mut u8,
    pub nhinor: *mut u8,
    pub nhivar: *mut u8,
    pub seq_list: [SeqInfo; SPRITE_SHEETS],
}

pub static mut the_memory: Memory = unsafe { std::mem::zeroed() };

/// The menus, the keyboard and mouse state, and test and debug switches.
pub struct Ui {
    pub current_menu: i16,
    pub chosen_entry: i8,
    pub menu_hit_enabled: i8,
    pub held_direction: i16,
    pub fight_key_held: i16,
    pub cheat_enabled: i16,
    pub quit_requested: i8,
    pub handler_data: InWork,
    pub slot_entries: [i8; MENU_SLOTS],
}

pub static mut the_ui: Ui = unsafe { std::mem::zeroed() };

/// The witch and her spinning beam.
pub struct Witch {
    pub direction: i8,
    pub on_screen: i8,
    pub angle: u8,
}

pub static mut the_witch: Witch = unsafe { std::mem::zeroed() };
