//! The game's variables (from the top of csrc/fmain.c): the actors, the hero's stats, the map
//! position, the screen structures, memory areas and flags. They start out zero (a few of the
//! tables with starting values are in `game_tables`). Each keeps the C name, because the other
//! modules and the C that is left (`statedump.c`) find them by it.
#![allow(non_upper_case_globals)]

use super::asm_blit::SShape;
use super::input::InWork;
use super::types::{Extent, FPage, IOAudio, Missile, SeqInfo, Shape};
use crate::amiga::{
    BitMap, DiskFontHeader, GfxBase, Process, RasInfo, RastPort, TextFont, View, ViewPort,
};
use std::ffi::c_void;

pub static mut GfxBase: *mut GfxBase = unsafe { std::mem::zeroed() };
pub static mut LayersBase: *mut c_void = unsafe { std::mem::zeroed() };
pub static mut active_carrier: i16 = unsafe { std::mem::zeroed() };
pub static mut loaded_actor_file: i16 = unsafe { std::mem::zeroed() };
pub static mut actors_loading: i8 = unsafe { std::mem::zeroed() };
pub static mut actors_on_screen: i8 = unsafe { std::mem::zeroed() };
pub static mut story_font: *mut TextFont = unsafe { std::mem::zeroed() };
pub static mut draw_order: [u8; 20] = unsafe { std::mem::zeroed() };
#[unsafe(export_name = "anim_list")]
pub static mut actor_list: [Shape; 20] = unsafe { std::mem::zeroed() };
#[unsafe(export_name = "anix")]
pub static mut active_actor_count: i16 = unsafe { std::mem::zeroed() };
#[unsafe(export_name = "anix2")]
pub static mut drawn_actor_count: i16 = unsafe { std::mem::zeroed() };
pub static mut aoff: i16 = unsafe { std::mem::zeroed() };
pub static mut audio_open: i16 = unsafe { std::mem::zeroed() };
pub static mut audioport: *mut c_void = unsafe { std::mem::zeroed() };
pub static mut battle_flag: i8 = unsafe { std::mem::zeroed() };
pub static mut bm_draw: *mut BitMap = unsafe { std::mem::zeroed() };
pub static mut bm_lim: *mut BitMap = unsafe { std::mem::zeroed() };
pub static mut bm_page1: *mut BitMap = unsafe { std::mem::zeroed() };
pub static mut bm_page2: *mut BitMap = unsafe { std::mem::zeroed() };
pub static mut bm_scroll: BitMap = unsafe { std::mem::zeroed() };
pub static mut bm_source: *mut BitMap = unsafe { std::mem::zeroed() };
pub static mut bm_text: *mut BitMap = unsafe { std::mem::zeroed() };
pub static mut bmask_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut bmod: i16 = unsafe { std::mem::zeroed() };
pub static mut boff: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut brave: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut brother: i16 = unsafe { std::mem::zeroed() };
pub static mut cheat_enabled: i16 = unsafe { std::mem::zeroed() };
pub static mut current_menu: i16 = unsafe { std::mem::zeroed() };
pub static mut danger_level: u8 = unsafe { std::mem::zeroed() };
#[unsafe(export_name = "daynight")]
pub static mut day_clock: u16 = unsafe { std::mem::zeroed() };
pub static mut day_period: i16 = unsafe { std::mem::zeroed() };
pub static mut encounter_number: u8 = unsafe { std::mem::zeroed() };
pub static mut encounter_type: u16 = unsafe { std::mem::zeroed() };
pub static mut encounter_x: u16 = unsafe { std::mem::zeroed() };
pub static mut encounter_y: u16 = unsafe { std::mem::zeroed() };
pub static mut current_extent: *mut Extent = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut fatigue: i16 = unsafe { std::mem::zeroed() };
pub static mut fiery_death: i8 = unsafe { std::mem::zeroed() };
pub static mut flasher: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut flying: i16 = unsafe { std::mem::zeroed() };
pub static mut font: *mut DiskFontHeader = unsafe { std::mem::zeroed() };
pub static mut drawing_page: *mut FPage = unsafe { std::mem::zeroed() };
pub static mut fp_page1: FPage = unsafe { std::mem::zeroed() };
pub static mut fp_page2: FPage = unsafe { std::mem::zeroed() };
pub static mut viewing_page: *mut FPage = unsafe { std::mem::zeroed() };
pub static mut freeze_timer: i16 = unsafe { std::mem::zeroed() };
pub static mut frustration: i8 = unsafe { std::mem::zeroed() };
pub static mut good_fairy_timer: u8 = unsafe { std::mem::zeroed() };
pub static mut handler_data: InWork = unsafe { std::mem::zeroed() };
pub static mut hero_place: u16 = unsafe { std::mem::zeroed() };
pub static mut hero_sector: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hero_x: u16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hero_y: u16 = unsafe { std::mem::zeroed() };
pub static mut chosen_entry: i8 = unsafe { std::mem::zeroed() };
pub static mut menu_hit_enabled: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hunger: i16 = unsafe { std::mem::zeroed() };
pub static mut i: i32 = unsafe { std::mem::zeroed() };
pub static mut image_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut tile_x: u16 = unsafe { std::mem::zeroed() };
pub static mut tile_y: u16 = unsafe { std::mem::zeroed() };
pub static mut ioaudio: *mut IOAudio = unsafe { std::mem::zeroed() };
pub static mut j: i16 = unsafe { std::mem::zeroed() };
pub static mut julstuff: [u8; 35] = unsafe { std::mem::zeroed() };
pub static mut kevstuff: [u8; 35] = unsafe { std::mem::zeroed() };
pub static mut held_direction: i16 = unsafe { std::mem::zeroed() };
pub static mut fight_key_held: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut kind: i16 = unsafe { std::mem::zeroed() };
pub static mut last_person: i16 = unsafe { std::mem::zeroed() };
pub static mut leader: i16 = unsafe { std::mem::zeroed() };
pub static mut li: *mut c_void = unsafe { std::mem::zeroed() };
pub static mut light_timer: i16 = unsafe { std::mem::zeroed() };
#[unsafe(export_name = "lightlevel")]
pub static mut light_level: u16 = unsafe { std::mem::zeroed() };
pub static mut target_region: u16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut luck: i16 = unsafe { std::mem::zeroed() };
pub static mut map_mem: *mut u8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut map_x: u16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut map_y: u16 = unsafe { std::mem::zeroed() };
pub static mut mask_buffer: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut next_missile: i16 = unsafe { std::mem::zeroed() };
pub static mut missile_slots: [Missile; 6] = unsafe { std::mem::zeroed() };
pub static mut mixflag: i16 = unsafe { std::mem::zeroed() };
pub static mut nearest_object: i16 = unsafe { std::mem::zeroed() };
pub static mut nearest_person: i16 = unsafe { std::mem::zeroed() };
pub static mut new_region: u16 = unsafe { std::mem::zeroed() };
pub static mut nextshape: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut nhinor: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut nhivar: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut oldview: *mut View = unsafe { std::mem::zeroed() };
pub static mut openflags: u16 = unsafe { std::mem::zeroed() };
pub static mut origDir: isize = unsafe { std::mem::zeroed() };
pub static mut pad1: u16 = unsafe { std::mem::zeroed() };
pub static mut pad2: u16 = unsafe { std::mem::zeroed() };
pub static mut pad3: u16 = unsafe { std::mem::zeroed() };
pub static mut pagea: BitMap = unsafe { std::mem::zeroed() };
pub static mut pageb: BitMap = unsafe { std::mem::zeroed() };
pub static mut nearest_person_distance: i16 = unsafe { std::mem::zeroed() };
pub static mut philstuff: [u8; 35] = unsafe { std::mem::zeroed() };
pub static mut planes: *mut *mut u8 = unsafe { std::mem::zeroed() };
pub static mut planesize: i16 = unsafe { std::mem::zeroed() };
pub static mut princess: i16 = unsafe { std::mem::zeroed() };
pub static mut queue_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut quit_requested: i8 = unsafe { std::mem::zeroed() };
pub static mut raft_nearness: i16 = unsafe { std::mem::zeroed() };
pub static mut slot_entries: [i8; 12] = unsafe { std::mem::zeroed() };
pub static mut ri_page1: RasInfo = unsafe { std::mem::zeroed() };
pub static mut ri_page2: RasInfo = unsafe { std::mem::zeroed() };
pub static mut ri_text: RasInfo = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut riding: i16 = unsafe { std::mem::zeroed() };
pub static mut current_rastport: *mut RastPort = unsafe { std::mem::zeroed() };
pub static mut rp_map: RastPort = unsafe { std::mem::zeroed() };
pub static mut rp_text: RastPort = unsafe { std::mem::zeroed() };
pub static mut rp_text2: RastPort = unsafe { std::mem::zeroed() };
pub static mut safe_flag: i8 = unsafe { std::mem::zeroed() };
pub static mut safe_r: u16 = unsafe { std::mem::zeroed() };
pub static mut safe_x: u16 = unsafe { std::mem::zeroed() };
pub static mut safe_y: u16 = unsafe { std::mem::zeroed() };
pub static mut sample: [*mut u8; 6] = unsafe { std::mem::zeroed() };
pub static mut sample_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut sample_size: [u32; 6] = unsafe { std::mem::zeroed() };
pub static mut scoremem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut secret_timer: i16 = unsafe { std::mem::zeroed() };
pub static mut sector_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut secx: i32 = unsafe { std::mem::zeroed() };
pub static mut secy: i32 = unsafe { std::mem::zeroed() };
pub static mut seg: isize = unsafe { std::mem::zeroed() };
pub static mut seq_list: [SeqInfo; 7] = unsafe { std::mem::zeroed() };
pub static mut loaded_setfig_file: i16 = unsafe { std::mem::zeroed() };
pub static mut shadow_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut shape_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut shapedata: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut shift: i16 = unsafe { std::mem::zeroed() };
pub static mut shp: *mut SShape = unsafe { std::mem::zeroed() };
pub static mut sleepwait: i16 = unsafe { std::mem::zeroed() };
pub static mut sprite_data: *mut u8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut stuff: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut terrain_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut panel_font: *mut TextFont = unsafe { std::mem::zeroed() };
pub static mut thistask: *mut Process = unsafe { std::mem::zeroed() };
pub static mut track: [*mut u8; 32] = unsafe { std::mem::zeroed() };
pub static mut turtle_nearness: i16 = unsafe { std::mem::zeroed() };
pub static mut v: View = unsafe { std::mem::zeroed() };
pub static mut view_status: i8 = unsafe { std::mem::zeroed() };
pub static mut volmem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut current_viewport: *mut ViewPort = unsafe { std::mem::zeroed() };
pub static mut vp_page: ViewPort = unsafe { std::mem::zeroed() };
pub static mut vp_text: ViewPort = unsafe { std::mem::zeroed() };
pub static mut wavmem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut wb_bmap: *mut BitMap = unsafe { std::mem::zeroed() };
pub static mut carrier_slot: i16 = unsafe { std::mem::zeroed() };
pub static mut witch_direction: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut wealth: i16 = unsafe { std::mem::zeroed() };
pub static mut witch_on_screen: i8 = unsafe { std::mem::zeroed() };
pub static mut witch_angle: u8 = unsafe { std::mem::zeroed() };
pub static mut wmask: i16 = unsafe { std::mem::zeroed() };
pub static mut work_bm: BitMap = unsafe { std::mem::zeroed() };
pub static mut wt: i16 = unsafe { std::mem::zeroed() };
pub static mut region_origin_x: u16 = unsafe { std::mem::zeroed() };
pub static mut extent_type: u16 = unsafe { std::mem::zeroed() };
pub static mut region_origin_y: u16 = unsafe { std::mem::zeroed() };
