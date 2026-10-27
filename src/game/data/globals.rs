//! The game's other variables (from the top of csrc/fmain.c): the sprite blitter's scratch values,
//! and some saved-game fields. They start out zero. The hero, world, encounter and actor variables
//! are in `state`.
#![allow(non_upper_case_globals)]

use crate::game::render::blit::SShape;

pub static mut shadow_mask_offset: i16 = unsafe { std::mem::zeroed() };

pub static mut sprite_mask_modulo: i16 = unsafe { std::mem::zeroed() };

pub static mut sprite_mask_offset: i16 = unsafe { std::mem::zeroed() };

pub static mut save_padding_1: u16 = unsafe { std::mem::zeroed() };

pub static mut save_padding_2: u16 = unsafe { std::mem::zeroed() };

pub static mut save_padding_3: u16 = unsafe { std::mem::zeroed() };

pub static mut princesses_rescued: i16 = unsafe { std::mem::zeroed() };

pub static mut sprite_x_shift: i16 = unsafe { std::mem::zeroed() };

pub static mut current_sshape: *mut SShape = unsafe { std::mem::zeroed() };

pub static mut last_word_mask: i16 = unsafe { std::mem::zeroed() };
