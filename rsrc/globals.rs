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
pub static mut actor_file: i16 = unsafe { std::mem::zeroed() };
pub static mut actors_loading: i8 = unsafe { std::mem::zeroed() };
pub static mut actors_on_screen: i8 = unsafe { std::mem::zeroed() };
pub static mut afont: *mut TextFont = unsafe { std::mem::zeroed() };
pub static mut anim_index: [u8; 20] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut anim_list: [Shape; 20] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut anix: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut anix2: i16 = unsafe { std::mem::zeroed() };
pub static mut aoff: i16 = unsafe { std::mem::zeroed() };
pub static mut audio_open: i16 = unsafe { std::mem::zeroed() };
pub static mut audioport: *mut c_void = unsafe { std::mem::zeroed() };
pub static mut battleflag: i8 = unsafe { std::mem::zeroed() };
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
pub static mut cheat1: i16 = unsafe { std::mem::zeroed() };
pub static mut cmode: i16 = unsafe { std::mem::zeroed() };
pub static mut danger_level: u8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut daynight: u16 = unsafe { std::mem::zeroed() };
pub static mut dayperiod: i16 = unsafe { std::mem::zeroed() };
pub static mut encounter_number: u8 = unsafe { std::mem::zeroed() };
pub static mut encounter_type: u16 = unsafe { std::mem::zeroed() };
pub static mut encounter_x: u16 = unsafe { std::mem::zeroed() };
pub static mut encounter_y: u16 = unsafe { std::mem::zeroed() };
pub static mut extn: *mut Extent = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut fatigue: i16 = unsafe { std::mem::zeroed() };
pub static mut fiery_death: i8 = unsafe { std::mem::zeroed() };
pub static mut flasher: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut flying: i16 = unsafe { std::mem::zeroed() };
pub static mut font: *mut DiskFontHeader = unsafe { std::mem::zeroed() };
pub static mut fp_drawing: *mut FPage = unsafe { std::mem::zeroed() };
pub static mut fp_page1: FPage = unsafe { std::mem::zeroed() };
pub static mut fp_page2: FPage = unsafe { std::mem::zeroed() };
pub static mut fp_viewing: *mut FPage = unsafe { std::mem::zeroed() };
pub static mut freeze_timer: i16 = unsafe { std::mem::zeroed() };
pub static mut frustflag: i8 = unsafe { std::mem::zeroed() };
pub static mut goodfairy: u8 = unsafe { std::mem::zeroed() };
pub static mut handler_data: InWork = unsafe { std::mem::zeroed() };
pub static mut hero_place: u16 = unsafe { std::mem::zeroed() };
pub static mut hero_sector: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hero_x: u16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hero_y: u16 = unsafe { std::mem::zeroed() };
pub static mut hit: i8 = unsafe { std::mem::zeroed() };
pub static mut hitgo: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hunger: i16 = unsafe { std::mem::zeroed() };
pub static mut i: i32 = unsafe { std::mem::zeroed() };
pub static mut image_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut img_x: u16 = unsafe { std::mem::zeroed() };
pub static mut img_y: u16 = unsafe { std::mem::zeroed() };
pub static mut ioaudio: *mut IOAudio = unsafe { std::mem::zeroed() };
pub static mut j: i16 = unsafe { std::mem::zeroed() };
pub static mut julstuff: [u8; 35] = unsafe { std::mem::zeroed() };
pub static mut kevstuff: [u8; 35] = unsafe { std::mem::zeroed() };
pub static mut keydir: i16 = unsafe { std::mem::zeroed() };
pub static mut keyfight: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut kind: i16 = unsafe { std::mem::zeroed() };
pub static mut last_person: i16 = unsafe { std::mem::zeroed() };
pub static mut leader: i16 = unsafe { std::mem::zeroed() };
pub static mut li: *mut c_void = unsafe { std::mem::zeroed() };
pub static mut light_timer: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut lightlevel: u16 = unsafe { std::mem::zeroed() };
pub static mut lregion: u16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut luck: i16 = unsafe { std::mem::zeroed() };
pub static mut map_mem: *mut u8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut map_x: u16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut map_y: u16 = unsafe { std::mem::zeroed() };
pub static mut mask_buffer: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut mdex: i16 = unsafe { std::mem::zeroed() };
pub static mut missile_list: [Missile; 6] = unsafe { std::mem::zeroed() };
pub static mut mixflag: i16 = unsafe { std::mem::zeroed() };
pub static mut nearest: i16 = unsafe { std::mem::zeroed() };
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
pub static mut perdist: i16 = unsafe { std::mem::zeroed() };
pub static mut philstuff: [u8; 35] = unsafe { std::mem::zeroed() };
pub static mut planes: *mut *mut u8 = unsafe { std::mem::zeroed() };
pub static mut planesize: i16 = unsafe { std::mem::zeroed() };
pub static mut princess: i16 = unsafe { std::mem::zeroed() };
pub static mut queue_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut quitflag: i8 = unsafe { std::mem::zeroed() };
pub static mut raftprox: i16 = unsafe { std::mem::zeroed() };
pub static mut real_options: [i8; 12] = unsafe { std::mem::zeroed() };
pub static mut ri_page1: RasInfo = unsafe { std::mem::zeroed() };
pub static mut ri_page2: RasInfo = unsafe { std::mem::zeroed() };
pub static mut ri_text: RasInfo = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut riding: i16 = unsafe { std::mem::zeroed() };
pub static mut rp: *mut RastPort = unsafe { std::mem::zeroed() };
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
pub static mut set_file: i16 = unsafe { std::mem::zeroed() };
pub static mut shadow_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut shape_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut shapedata: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut shift: i16 = unsafe { std::mem::zeroed() };
pub static mut shp: *mut SShape = unsafe { std::mem::zeroed() };
pub static mut sleepwait: i16 = unsafe { std::mem::zeroed() };
pub static mut sprite_data: *mut u8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut stuff: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut terra_mem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut tfont: *mut TextFont = unsafe { std::mem::zeroed() };
pub static mut thistask: *mut Process = unsafe { std::mem::zeroed() };
pub static mut track: [*mut u8; 32] = unsafe { std::mem::zeroed() };
pub static mut turtleprox: i16 = unsafe { std::mem::zeroed() };
pub static mut v: View = unsafe { std::mem::zeroed() };
pub static mut viewstatus: i8 = unsafe { std::mem::zeroed() };
pub static mut volmem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut vp: *mut ViewPort = unsafe { std::mem::zeroed() };
pub static mut vp_page: ViewPort = unsafe { std::mem::zeroed() };
pub static mut vp_text: ViewPort = unsafe { std::mem::zeroed() };
pub static mut wavmem: *mut u8 = unsafe { std::mem::zeroed() };
pub static mut wb_bmap: *mut BitMap = unsafe { std::mem::zeroed() };
pub static mut wcarry: i16 = unsafe { std::mem::zeroed() };
pub static mut wdir: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut wealth: i16 = unsafe { std::mem::zeroed() };
pub static mut witchflag: i8 = unsafe { std::mem::zeroed() };
pub static mut witchindex: u8 = unsafe { std::mem::zeroed() };
pub static mut wmask: i16 = unsafe { std::mem::zeroed() };
pub static mut work_bm: BitMap = unsafe { std::mem::zeroed() };
pub static mut wt: i16 = unsafe { std::mem::zeroed() };
pub static mut xreg: u16 = unsafe { std::mem::zeroed() };
pub static mut xtype: u16 = unsafe { std::mem::zeroed() };
pub static mut yreg: u16 = unsafe { std::mem::zeroed() };
