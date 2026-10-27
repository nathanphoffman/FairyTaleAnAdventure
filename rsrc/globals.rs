//! The game's other variables (from the top of csrc/fmain.c): the screen structures, memory areas
//! and flags. They start out zero. The hero, world, encounter and actor variables are in `state`.
#![allow(non_upper_case_globals)]

use super::asm_blit::SShape;
use super::types::IOAudio;
use crate::amiga::{DiskFontHeader, GfxBase, Process, TextFont, View};
use std::ffi::c_void;

pub static mut GfxBase: *mut GfxBase = unsafe { std::mem::zeroed() };
pub static mut LayersBase: *mut c_void = unsafe { std::mem::zeroed() };
pub static mut story_font: *mut TextFont = unsafe { std::mem::zeroed() };
pub static mut aoff: i16 = unsafe { std::mem::zeroed() };
pub static mut audio_open: i16 = unsafe { std::mem::zeroed() };
pub static mut audioport: *mut c_void = unsafe { std::mem::zeroed() };
pub static mut bmod: i16 = unsafe { std::mem::zeroed() };
pub static mut boff: i16 = unsafe { std::mem::zeroed() };
pub static mut font: *mut DiskFontHeader = unsafe { std::mem::zeroed() };
pub static mut i: i32 = unsafe { std::mem::zeroed() };
pub static mut ioaudio: *mut IOAudio = unsafe { std::mem::zeroed() };
pub static mut j: i16 = unsafe { std::mem::zeroed() };
pub static mut li: *mut c_void = unsafe { std::mem::zeroed() };
pub static mut openflags: u16 = unsafe { std::mem::zeroed() };
pub static mut origDir: isize = unsafe { std::mem::zeroed() };
pub static mut pad1: u16 = unsafe { std::mem::zeroed() };
pub static mut pad2: u16 = unsafe { std::mem::zeroed() };
pub static mut pad3: u16 = unsafe { std::mem::zeroed() };
pub static mut princess: i16 = unsafe { std::mem::zeroed() };
pub static mut secx: i32 = unsafe { std::mem::zeroed() };
pub static mut secy: i32 = unsafe { std::mem::zeroed() };
pub static mut seg: isize = unsafe { std::mem::zeroed() };
pub static mut shift: i16 = unsafe { std::mem::zeroed() };
pub static mut shp: *mut SShape = unsafe { std::mem::zeroed() };
pub static mut sleepwait: i16 = unsafe { std::mem::zeroed() };
pub static mut panel_font: *mut TextFont = unsafe { std::mem::zeroed() };
pub static mut thistask: *mut Process = unsafe { std::mem::zeroed() };
pub static mut v: View = unsafe { std::mem::zeroed() };
pub static mut wmask: i16 = unsafe { std::mem::zeroed() };
