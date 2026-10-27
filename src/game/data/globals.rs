//! The game's other variables (from the top of csrc/fmain.c): the Amiga library bases, fonts, the blitter
//! scratch values and open flags. They start out zero. The hero, world, encounter and actor variables
//! are in `state`. (`GfxBase` and `LayersBase` keep the Amiga library names.)
#![allow(non_upper_case_globals)]

use crate::amiga::{DiskFontHeader, GfxBase, Process, TextFont, View};
use crate::game::data::types::IOAudio;
use crate::game::render::blit::SShape;
use std::ffi::c_void;

pub static mut GfxBase: *mut GfxBase = unsafe { std::mem::zeroed() };

pub static mut LayersBase: *mut c_void = unsafe { std::mem::zeroed() };

pub static mut story_font: *mut TextFont = unsafe { std::mem::zeroed() };

pub static mut shadow_mask_offset: i16 = unsafe { std::mem::zeroed() };

pub static mut audio_is_open: i16 = unsafe { std::mem::zeroed() };

pub static mut audio_port: *mut c_void = unsafe { std::mem::zeroed() };

pub static mut sprite_mask_modulo: i16 = unsafe { std::mem::zeroed() };

pub static mut sprite_mask_offset: i16 = unsafe { std::mem::zeroed() };

pub static mut font_header: *mut DiskFontHeader = unsafe { std::mem::zeroed() };

pub static mut audio_request: *mut IOAudio = unsafe { std::mem::zeroed() };

pub static mut layer_info: *mut c_void = unsafe { std::mem::zeroed() };

pub static mut open_flags: u16 = unsafe { std::mem::zeroed() };

pub static mut original_dir: isize = unsafe { std::mem::zeroed() };

pub static mut save_padding_1: u16 = unsafe { std::mem::zeroed() };

pub static mut save_padding_2: u16 = unsafe { std::mem::zeroed() };

pub static mut save_padding_3: u16 = unsafe { std::mem::zeroed() };

pub static mut princesses_rescued: i16 = unsafe { std::mem::zeroed() };

pub static mut font_segment: isize = unsafe { std::mem::zeroed() };

pub static mut sprite_x_shift: i16 = unsafe { std::mem::zeroed() };

pub static mut current_sshape: *mut SShape = unsafe { std::mem::zeroed() };

pub static mut panel_font: *mut TextFont = unsafe { std::mem::zeroed() };

pub static mut this_task: *mut Process = unsafe { std::mem::zeroed() };

pub static mut main_view: View = unsafe { std::mem::zeroed() };

pub static mut last_word_mask: i16 = unsafe { std::mem::zeroed() };
