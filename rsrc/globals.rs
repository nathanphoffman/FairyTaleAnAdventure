//! The game's variables (from the top of csrc/fmain.c): the actors, the hero's stats, the map
//! position, the screen structures, memory areas and flags. They start out zero (a few of the
//! tables with starting values are in `game_tables`). Each keeps the C name, because the other
//! modules and the C that is left (`statedump.c`) find them by it.
#![allow(non_upper_case_globals)]

use super::input::InWork;
use super::types::{FPage, Missile, SeqInfo, Shape};
use crate::amiga::{BitMap, RasInfo, RastPort, View, ViewPort};
use std::ffi::c_void;

#[unsafe(no_mangle)]
pub static mut GfxBase: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut LayersBase: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut active_carrier: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut actor_file: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut actors_loading: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut actors_on_screen: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut afont: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut anim_index: [u8; 20] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut anim_list: [Shape; 20] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut anix: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut anix2: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut aoff: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut audio_open: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut audioport: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut battleflag: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut bm_draw: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut bm_lim: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut bm_page1: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut bm_page2: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut bm_scroll: BitMap = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut bm_source: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut bm_text: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut bmask_mem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut bmod: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut boff: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut brave: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut brother: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut cheat1: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut cmode: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut danger_level: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut daynight: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut dayperiod: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut dbg: i32 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut encounter_number: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut encounter_type: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut encounter_x: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut encounter_y: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut extn: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut fatigue: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut fiery_death: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut flasher: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut flying: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut font: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut fp_drawing: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut fp_page1: FPage = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut fp_page2: FPage = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut fp_viewing: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut freeze_timer: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut frustflag: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut goodfairy: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut handlerStuff: [u8; 24] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut handler_data: InWork = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hero_place: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hero_sector: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hero_x: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hero_y: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hit: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hitgo: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut hunger: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut i: i32 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut image_mem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut img_x: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut img_y: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut inputDevPort: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut inputRequestBlock: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut ioaudio: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut j: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut julstuff: [u8; 35] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut k: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut kevstuff: [u8; 35] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut keydir: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut keyfight: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut kind: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut last_person: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut leader: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut li: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut light_timer: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut lightlevel: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut lregion: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut luck: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut map_mem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut map_x: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut map_y: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut mask_buffer: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut mdex: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut missile_list: [Missile; 6] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut mixflag: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut n: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut nearest: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut nearest_person: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut new_region: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut nextshape: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut nhinor: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut nhivar: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut oldview: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut openflags: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut origDir: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pad1: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pad2: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pad3: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pad4: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pad5: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pad6: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pad7: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pagea: BitMap = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pageb: BitMap = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pass: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut passmode: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut perdist: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut philstuff: [u8; 35] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut pl: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut planes: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut planesize: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut princess: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut queue_mem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut quitflag: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut raftprox: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut real_options: [i8; 12] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut ri_page1: RasInfo = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut ri_page2: RasInfo = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut ri_text: RasInfo = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut ri_title: RasInfo = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut riding: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut rp: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut rp_map: RastPort = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut rp_text: RastPort = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut rp_text2: RastPort = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut safe_flag: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut safe_r: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut safe_x: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut safe_y: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut sample: [*mut u8; 6] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut sample_mem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut sample_size: [u32; 6] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut scoremem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut secret_timer: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut sector_mem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut secx: i32 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut secy: i32 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut seg: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut seq_list: [SeqInfo; 7] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut set_file: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut shadow_mem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut shape_mem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut shapedata: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut shift: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut shp: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut sleepwait: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut sprite_data: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut stuff: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut tempshape: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut terra_mem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut tfont: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut thistask: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut track: [*mut u8; 32] = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut turtleprox: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut v: View = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut viewstatus: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut volmem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut vp: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut vp_page: ViewPort = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut vp_text: ViewPort = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut vp_title: ViewPort = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut wavmem: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut wb_bmap: *mut c_void = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut wcarry: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut wdir: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut wealth: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut witchflag: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut witchindex: i8 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut wmask: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut work_bm: BitMap = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut wt: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut xreg: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut xtype: i16 = unsafe { std::mem::zeroed() };
#[unsafe(no_mangle)]
pub static mut yreg: i16 = unsafe { std::mem::zeroed() };
