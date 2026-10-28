//! The game's main state, gathered into a few structs instead of loose variables.
//!
//! `Game` holds all of them (`game.hero`, `game.world`, ...). They start out zero apart from the few
//! values set in `Game::new`.
use crate::game::data::object_tables::{ObjectLists, initial_page_colors};
use crate::game::data::random::Rng;
use crate::game::data::tables::{initial_extents, initial_menus};
use crate::game::data::types::{Extent, Menu, Missile, Need, SeqInfo, Shape};
use crate::game::render::blit::SShape;
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
    pub princesses_rescued: i16,
}

impl Default for Hero {
    fn default() -> Hero {
        Hero {
            x: 0,
            y: 0,
            sector: 0,
            place: 0,
            brave: 0,
            luck: 0,
            kindness: 0,
            wealth: 0,
            hunger: 0,
            fatigue: 0,
            riding: 0,
            flying: 0,
            brother: 0,
            julstuff: [0; STUFF_SLOTS],
            kevstuff: [0; STUFF_SLOTS],
            philstuff: [0; STUFF_SLOTS],
            safe_flag: 0,
            safe_r: 0,
            safe_x: 0,
            safe_y: 0,
            good_fairy_timer: 0,
            princesses_rescued: 0,
        }
    }
}

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

/// The world around the hero: the map position, region, clock, light and magic timers.
#[derive(Default)]
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
    /// Set when the hero walks into a locked door.
    pub bumped_locked_door: i8,
}

/// The monster encounter being set up and how dangerous the place is.
#[derive(Default)]
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
#[derive(Default)]
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
    /// Where the actor being moved would be after this step.
    pub test_x: u16,
    pub test_y: u16,
    /// The first free actor slot while the object lists are being turned into actors.
    pub free_slot: i16,
    /// The slot of the turtle's eggs on the actor list (0 if there are none).
    pub turtle_eggs: i8,
}

/// A way to name one actor of `Game::actors.list` by its number, so that code can hold on to "that
/// actor" while it calls others that change the list. Read it with `game.actors.list[r.slot()]`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ActorRef(usize);

impl ActorRef {
    pub fn new(index: usize) -> ActorRef {
        ActorRef(index)
    }

    pub fn slot(self) -> usize {
        self.0
    }
}

/// The same for the missiles in flight.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MissileRef(usize);

impl MissileRef {
    pub fn new(index: usize) -> MissileRef {
        MissileRef(index)
    }

    pub fn slot(self) -> usize {
        self.0
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
    /// Which disk blocks are loaded now (image banks, terrain, sectors, region, characters).
    pub loaded_blocks: Need,
    /// The overview map of the world sectors (see `world::regions`).
    pub minimap: [i16; 120],
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
            loaded_blocks: Need {
                image: [0, 0, 0, 0],
                terra1: 1,
                terra2: 2,
                sector: 0,
                region: 0,
                setchar: 0,
            },
            minimap: [0; 120],
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
#[derive(Default)]
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
    pub print_queue: PrintQueue,
    pub intro_skip_requested: i8,
    /// The direction lit on the compass (9 when none).
    pub compass_lit_direction: i16,
}

/// The sprite blitter's work values for the sprite being drawn.
pub struct Blitter {
    pub sshape: SShape,
    pub shadow_mask_offset: i16,
    pub sprite_mask_offset: i16,
    pub sprite_mask_modulo: i16,
    pub sprite_x_shift: i16,
    pub last_word_mask: i16,
}

impl Blitter {
    pub fn new() -> Blitter {
        Blitter {
            sshape: SShape::new(),
            shadow_mask_offset: 0,
            sprite_mask_offset: 0,
            sprite_mask_modulo: 0,
            sprite_x_shift: 0,
            last_word_mask: 0,
        }
    }
}

/// The text printing queue: requests wait here until the screen code takes them.
#[derive(Default)]
pub struct PrintQueue {
    pub items: [i8; 32],
    /// Where the next request is added, and where the next one is taken from.
    pub write: i16,
    pub read: i16,
}

/// The witch and her spinning beam.
pub struct Witch {
    pub direction: i8,
    pub on_screen: i8,
    pub angle: u8,
    /// Which side of the beam's two edges the hero is on (see `render::witch::side`).
    pub side_1: i16,
    pub side_2: i16,
}

/// Everything that changes while the game runs, in one value that is passed down from `run_game`.
pub struct Game {
    pub witch: Witch,
    pub ui: Ui,
    pub encounter: Encounters,
    pub memory: Memory,
    pub screen: Screen,
    pub blitter: Blitter,
    pub save_padding: [u16; 3],
    pub rng: Rng,
    pub extents: [Extent; 23],
    pub menus: [Menu; 10],
    pub page_colors: [u16; 32],
    pub objects: ObjectLists,
    pub actors: Actors,
    pub world: World,
    pub hero: Hero,
}

impl Game {
    pub fn new() -> Game {
        Game {
            witch: Witch {
                direction: 0,
                on_screen: 0,
                angle: 0,
                side_1: 0,
                side_2: 0,
            },
            ui: Ui {
                compass_lit_direction: 9,
                ..Default::default()
            },
            encounter: Default::default(),
            screen: Screen::new(),
            blitter: Blitter::new(),
            save_padding: [0; 3],
            rng: Rng::new(),
            extents: initial_extents,
            menus: initial_menus,
            page_colors: initial_page_colors,
            objects: ObjectLists::new(),
            actors: Default::default(),
            world: World {
                region: 3,
                ..Default::default()
            },
            hero: Default::default(),
            memory: Memory::new(),
        }
    }
}
