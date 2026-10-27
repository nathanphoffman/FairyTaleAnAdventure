//! The tables that describe the game's kinds of things (from the top of csrc/fmain.c): the people,
//! monsters, doors, animation frames, items, menus and map regions. (What lies where in each region,
//! the colours and the loot odds are in `object_tables`.) They are `static mut` because some of them are
//! changed while the game runs (`extent_list`, the menu `enabled` flags, ...). Each keeps the C name
//! and layout, since the rest of the game finds them by that name.
#![allow(non_upper_case_globals)]

use crate::game::actors::encounters::Encounter;
use crate::game::world::objects::SetFigEntry;
use crate::game::data::types::{Bro, Door, Extent, InvItem, Letter, Menu, Need, StateEntry};
use std::ffi::c_char;
use crate::game::data::state::{the_hero};

/// The kinds of people (setfigs): which sprite file, the first image, and whether they can talk.
pub static mut setfig_table: [SetFigEntry; 14] = [
    SetFigEntry { cfile_entry: 13, image_base: 0, can_talk: 1 },
    SetFigEntry { cfile_entry: 13, image_base: 4, can_talk: 1 },
    SetFigEntry { cfile_entry: 14, image_base: 0, can_talk: 0 },
    SetFigEntry { cfile_entry: 14, image_base: 1, can_talk: 0 },
    SetFigEntry { cfile_entry: 14, image_base: 2, can_talk: 0 },
    SetFigEntry { cfile_entry: 14, image_base: 4, can_talk: 1 },
    SetFigEntry { cfile_entry: 14, image_base: 6, can_talk: 0 },
    SetFigEntry { cfile_entry: 14, image_base: 7, can_talk: 0 },
    SetFigEntry { cfile_entry: 15, image_base: 0, can_talk: 0 },
    SetFigEntry { cfile_entry: 16, image_base: 0, can_talk: 0 },
    SetFigEntry { cfile_entry: 16, image_base: 6, can_talk: 0 },
    SetFigEntry { cfile_entry: 16, image_base: 7, can_talk: 0 },
    SetFigEntry { cfile_entry: 17, image_base: 0, can_talk: 1 },
    SetFigEntry { cfile_entry: 17, image_base: 4, can_talk: 1 },
];

/// The monsters: hit points, aggression, weapon row, cleverness, treasure and sprite file.
pub static mut encounter_chart: [Encounter; 11] = [
    Encounter { hitpoints: 18, agressive: 1, arms: 2, cleverness: 0, treasure: 2, file_id: 6 },
    Encounter { hitpoints: 12, agressive: 1, arms: 4, cleverness: 1, treasure: 1, file_id: 6 },
    Encounter { hitpoints: 16, agressive: 1, arms: 6, cleverness: 1, treasure: 4, file_id: 7 },
    Encounter { hitpoints: 8, agressive: 1, arms: 3, cleverness: 0, treasure: 3, file_id: 7 },
    Encounter { hitpoints: 16, agressive: 1, arms: 6, cleverness: 1, treasure: 0, file_id: 8 },
    Encounter { hitpoints: 9, agressive: 1, arms: 3, cleverness: 0, treasure: 0, file_id: 7 },
    Encounter { hitpoints: 10, agressive: 1, arms: 6, cleverness: 1, treasure: 0, file_id: 8 },
    Encounter { hitpoints: 40, agressive: 1, arms: 7, cleverness: 1, treasure: 0, file_id: 8 },
    Encounter { hitpoints: 12, agressive: 1, arms: 6, cleverness: 1, treasure: 0, file_id: 9 },
    Encounter { hitpoints: 50, agressive: 1, arms: 5, cleverness: 0, treasure: 0, file_id: 9 },
    Encounter { hitpoints: 4, agressive: 0, arms: 0, cleverness: 0, treasure: 0, file_id: 9 },
];

/// Next state of a weapon swing, for each state and each of 4 random picks.
pub static mut swing_transitions: [[i8; 4]; 9] = [
    [1, 8, 0, 1],
    [2, 0, 1, 0],
    [3, 1, 2, 8],
    [4, 2, 3, 7],
    [5, 3, 4, 6],
    [6, 4, 5, 5],
    [8, 5, 6, 4],
    [8, 6, 7, 3],
    [0, 6, 8, 2],
];

/// What to draw for each animation state: the figure image, the weapon image, and where the weapon is.
pub static mut animation_frames: [StateEntry; 87] = [
    StateEntry { figure: 0, wpn_no: 11, wpn_x: -2, wpn_y: 11 },
    StateEntry { figure: 1, wpn_no: 11, wpn_x: -3, wpn_y: 11 },
    StateEntry { figure: 2, wpn_no: 11, wpn_x: -3, wpn_y: 10 },
    StateEntry { figure: 3, wpn_no: 11, wpn_x: -3, wpn_y: 9 },
    StateEntry { figure: 4, wpn_no: 11, wpn_x: -3, wpn_y: 10 },
    StateEntry { figure: 5, wpn_no: 11, wpn_x: -3, wpn_y: 11 },
    StateEntry { figure: 6, wpn_no: 11, wpn_x: -2, wpn_y: 11 },
    StateEntry { figure: 7, wpn_no: 11, wpn_x: -1, wpn_y: 11 },
    StateEntry { figure: 8, wpn_no: 9, wpn_x: -12, wpn_y: 11 },
    StateEntry { figure: 9, wpn_no: 9, wpn_x: -11, wpn_y: 12 },
    StateEntry { figure: 10, wpn_no: 9, wpn_x: -8, wpn_y: 13 },
    StateEntry { figure: 11, wpn_no: 9, wpn_x: -4, wpn_y: 13 },
    StateEntry { figure: 12, wpn_no: 9, wpn_x: 0, wpn_y: 13 },
    StateEntry { figure: 13, wpn_no: 9, wpn_x: -4, wpn_y: 13 },
    StateEntry { figure: 14, wpn_no: 9, wpn_x: -8, wpn_y: 13 },
    StateEntry { figure: 15, wpn_no: 9, wpn_x: -11, wpn_y: 12 },
    StateEntry { figure: 16, wpn_no: 14, wpn_x: -1, wpn_y: 1 },
    StateEntry { figure: 17, wpn_no: 14, wpn_x: -1, wpn_y: 2 },
    StateEntry { figure: 18, wpn_no: 14, wpn_x: -1, wpn_y: 3 },
    StateEntry { figure: 19, wpn_no: 14, wpn_x: -1, wpn_y: 4 },
    StateEntry { figure: 20, wpn_no: 14, wpn_x: -1, wpn_y: 3 },
    StateEntry { figure: 21, wpn_no: 14, wpn_x: -1, wpn_y: 2 },
    StateEntry { figure: 22, wpn_no: 14, wpn_x: -1, wpn_y: 1 },
    StateEntry { figure: 23, wpn_no: 14, wpn_x: -1, wpn_y: 1 },
    StateEntry { figure: 24, wpn_no: 10, wpn_x: 5, wpn_y: 12 },
    StateEntry { figure: 25, wpn_no: 10, wpn_x: 3, wpn_y: 12 },
    StateEntry { figure: 26, wpn_no: 10, wpn_x: 2, wpn_y: 12 },
    StateEntry { figure: 27, wpn_no: 10, wpn_x: 3, wpn_y: 12 },
    StateEntry { figure: 28, wpn_no: 10, wpn_x: 5, wpn_y: 12 },
    StateEntry { figure: 29, wpn_no: 10, wpn_x: 6, wpn_y: 12 },
    StateEntry { figure: 30, wpn_no: 10, wpn_x: 6, wpn_y: 11 },
    StateEntry { figure: 31, wpn_no: 10, wpn_x: 6, wpn_y: 12 },
    StateEntry { figure: 32, wpn_no: 11, wpn_x: -2, wpn_y: 12 },
    StateEntry { figure: 32, wpn_no: 10, wpn_x: 0, wpn_y: 12 },
    StateEntry { figure: 33, wpn_no: 0, wpn_x: 2, wpn_y: 10 },
    StateEntry { figure: 34, wpn_no: 1, wpn_x: 4, wpn_y: 6 },
    StateEntry { figure: 34, wpn_no: 2, wpn_x: 1, wpn_y: 4 },
    StateEntry { figure: 34, wpn_no: 3, wpn_x: 0, wpn_y: 4 },
    StateEntry { figure: 36, wpn_no: 4, wpn_x: -5, wpn_y: 0 },
    StateEntry { figure: 36, wpn_no: 5, wpn_x: -10, wpn_y: 1 },
    StateEntry { figure: 35, wpn_no: 12, wpn_x: -5, wpn_y: 5 },
    StateEntry { figure: 36, wpn_no: 0, wpn_x: 0, wpn_y: 6 },
    StateEntry { figure: 38, wpn_no: 85, wpn_x: -6, wpn_y: 5 },
    StateEntry { figure: 37, wpn_no: 81, wpn_x: -6, wpn_y: 5 },
    StateEntry { figure: 40, wpn_no: 9, wpn_x: -7, wpn_y: 12 },
    StateEntry { figure: 40, wpn_no: 8, wpn_x: -9, wpn_y: 9 },
    StateEntry { figure: 41, wpn_no: 7, wpn_x: -10, wpn_y: 5 },
    StateEntry { figure: 42, wpn_no: 7, wpn_x: -12, wpn_y: 4 },
    StateEntry { figure: 42, wpn_no: 6, wpn_x: -12, wpn_y: 3 },
    StateEntry { figure: 42, wpn_no: 5, wpn_x: -12, wpn_y: 3 },
    StateEntry { figure: 44, wpn_no: 5, wpn_x: -8, wpn_y: 3 },
    StateEntry { figure: 44, wpn_no: 14, wpn_x: -7, wpn_y: 6 },
    StateEntry { figure: 43, wpn_no: 13, wpn_x: -7, wpn_y: 8 },
    StateEntry { figure: 42, wpn_no: 5, wpn_x: -12, wpn_y: 3 },
    StateEntry { figure: 46, wpn_no: 86, wpn_x: -3, wpn_y: 0 },
    StateEntry { figure: 45, wpn_no: 82, wpn_x: -3, wpn_y: 0 },
    StateEntry { figure: 48, wpn_no: 14, wpn_x: -3, wpn_y: 0 },
    StateEntry { figure: 48, wpn_no: 6, wpn_x: -3, wpn_y: -1 },
    StateEntry { figure: 49, wpn_no: 5, wpn_x: -2, wpn_y: -3 },
    StateEntry { figure: 50, wpn_no: 5, wpn_x: -3, wpn_y: -4 },
    StateEntry { figure: 50, wpn_no: 4, wpn_x: 0, wpn_y: 0 },
    StateEntry { figure: 50, wpn_no: 3, wpn_x: 3, wpn_y: 0 },
    StateEntry { figure: 52, wpn_no: 4, wpn_x: 6, wpn_y: 1 },
    StateEntry { figure: 52, wpn_no: 15, wpn_x: 7, wpn_y: 3 },
    StateEntry { figure: 51, wpn_no: 14, wpn_x: 1, wpn_y: 6 },
    StateEntry { figure: 50, wpn_no: 4, wpn_x: 0, wpn_y: 0 },
    StateEntry { figure: 54, wpn_no: 87, wpn_x: 3, wpn_y: 0 },
    StateEntry { figure: 53, wpn_no: 83, wpn_x: 3, wpn_y: 0 },
    StateEntry { figure: 56, wpn_no: 10, wpn_x: 5, wpn_y: 11 },
    StateEntry { figure: 56, wpn_no: 0, wpn_x: 6, wpn_y: 9 },
    StateEntry { figure: 57, wpn_no: 1, wpn_x: 10, wpn_y: 6 },
    StateEntry { figure: 58, wpn_no: 1, wpn_x: 10, wpn_y: 5 },
    StateEntry { figure: 58, wpn_no: 2, wpn_x: 7, wpn_y: 3 },
    StateEntry { figure: 58, wpn_no: 3, wpn_x: 6, wpn_y: 3 },
    StateEntry { figure: 60, wpn_no: 4, wpn_x: 1, wpn_y: 0 },
    StateEntry { figure: 60, wpn_no: 3, wpn_x: 3, wpn_y: 2 },
    StateEntry { figure: 59, wpn_no: 15, wpn_x: 4, wpn_y: 1 },
    StateEntry { figure: 58, wpn_no: 4, wpn_x: 5, wpn_y: 1 },
    StateEntry { figure: 62, wpn_no: 84, wpn_x: 3, wpn_y: 0 },
    StateEntry { figure: 61, wpn_no: 80, wpn_x: 3, wpn_y: 0 },
    StateEntry { figure: 47, wpn_no: 0, wpn_x: 5, wpn_y: 11 },
    StateEntry { figure: 63, wpn_no: 0, wpn_x: 6, wpn_y: 9 },
    StateEntry { figure: 39, wpn_no: 0, wpn_x: 6, wpn_y: 9 },
    StateEntry { figure: 55, wpn_no: 10, wpn_x: 5, wpn_y: 11 },
    StateEntry { figure: 64, wpn_no: 10, wpn_x: 5, wpn_y: 11 },
    StateEntry { figure: 65, wpn_no: 10, wpn_x: 5, wpn_y: 11 },
    StateEntry { figure: 66, wpn_no: 10, wpn_x: 5, wpn_y: 11 },
];

/// Every door: where it is outside, where it leads inside, its kind and the sectors it joins.
pub static mut door_list: [Door; 86] = [
    Door { xc1: 4464, yc1: 20576, xc2: 10352, yc2: 35680, kind: 1, secs: 1 },
    Door { xc1: 4464, yc1: 20576, xc2: 10352, yc2: 35680, kind: 1, secs: 1 },
    Door { xc1: 4464, yc1: 20576, xc2: 10352, yc2: 35680, kind: 1, secs: 1 },
    Door { xc1: 4464, yc1: 20576, xc2: 10352, yc2: 35680, kind: 1, secs: 1 },
    Door { xc1: 5008, yc1: 7008, xc2: 6528, yc2: 35936, kind: 18, secs: 2 },
    Door { xc1: 6000, yc1: 27296, xc2: 8816, yc2: 38560, kind: 9, secs: 1 },
    Door { xc1: 6512, yc1: 25248, xc2: 8048, yc2: 38560, kind: 9, secs: 1 },
    Door { xc1: 6816, yc1: 19360, xc2: 5024, yc2: 38304, kind: 17, secs: 1 },
    Door { xc1: 6816, yc1: 19552, xc2: 5024, yc2: 38752, kind: 17, secs: 1 },
    Door { xc1: 6944, yc1: 19296, xc2: 5920, yc2: 38240, kind: 17, secs: 1 },
    Door { xc1: 7040, yc1: 19328, xc2: 5504, yc2: 38272, kind: 17, secs: 1 },
    Door { xc1: 7040, yc1: 19520, xc2: 5504, yc2: 38720, kind: 17, secs: 1 },
    Door { xc1: 7792, yc1: 15200, xc2: 10368, yc2: 40032, kind: 3, secs: 1 },
    Door { xc1: 9344, yc1: 13216, xc2: 11904, yc2: 36256, kind: 1, secs: 1 },
    Door { xc1: 10592, yc1: 34656, xc2: 11008, yc2: 37568, kind: 15, secs: 1 },
    Door { xc1: 11008, yc1: 37568, xc2: 10592, yc2: 34688, kind: 15, secs: 2 },
    Door { xc1: 11264, yc1: 29024, xc2: 10992, yc2: 37728, kind: 9, secs: 1 },
    Door { xc1: 12144, yc1: 11872, xc2: 12672, yc2: 39520, kind: 3, secs: 1 },
    Door { xc1: 12144, yc1: 25504, xc2: 7280, yc2: 38560, kind: 9, secs: 1 },
    Door { xc1: 12672, yc1: 14528, xc2: 10112, yc2: 39104, kind: 1, secs: 1 },
    Door { xc1: 13424, yc1: 19296, xc2: 1136, yc2: 36576, kind: 15, secs: 2 },
    Door { xc1: 15840, yc1: 7104, xc2: 12000, yc2: 37824, kind: 7, secs: 1 },
    Door { xc1: 15872, yc1: 7104, xc2: 12032, yc2: 37824, kind: 7, secs: 1 },
    Door { xc1: 17008, yc1: 9568, xc2: 11904, yc2: 39520, kind: 3, secs: 1 },
    Door { xc1: 17024, yc1: 15296, xc2: 10624, yc2: 39104, kind: 1, secs: 1 },
    Door { xc1: 17888, yc1: 21376, xc2: 9680, yc2: 38528, kind: 10, secs: 1 },
    Door { xc1: 18304, yc1: 12224, xc2: 9600, yc2: 39104, kind: 1, secs: 1 },
    Door { xc1: 18528, yc1: 26176, xc2: 7264, yc2: 39488, kind: 18, secs: 1 },
    Door { xc1: 18576, yc1: 26272, xc2: 7312, yc2: 39584, kind: 11, secs: 1 },
    Door { xc1: 18784, yc1: 23360, xc2: 8800, yc2: 39488, kind: 18, secs: 1 },
    Door { xc1: 18832, yc1: 23456, xc2: 8848, yc2: 39584, kind: 11, secs: 1 },
    Door { xc1: 18848, yc1: 15552, xc2: 2976, yc2: 33472, kind: 2, secs: 1 },
    Door { xc1: 18896, yc1: 15808, xc2: 3024, yc2: 33984, kind: 2, secs: 1 },
    Door { xc1: 18896, yc1: 15872, xc2: 3024, yc2: 34048, kind: 2, secs: 1 },
    Door { xc1: 18960, yc1: 15488, xc2: 3344, yc2: 33408, kind: 1, secs: 1 },
    Door { xc1: 18960, yc1: 15680, xc2: 3856, yc2: 33600, kind: 1, secs: 1 },
    Door { xc1: 18992, yc1: 15808, xc2: 3632, yc2: 34240, kind: 1, secs: 1 },
    Door { xc1: 19040, yc1: 16000, xc2: 4192, yc2: 34176, kind: 1, secs: 1 },
    Door { xc1: 19056, yc1: 15488, xc2: 4976, yc2: 33408, kind: 1, secs: 1 },
    Door { xc1: 19072, yc1: 15680, xc2: 4496, yc2: 33600, kind: 1, secs: 1 },
    Door { xc1: 19568, yc1: 12896, xc2: 9600, yc2: 40032, kind: 3, secs: 1 },
    Door { xc1: 19808, yc1: 21568, xc2: 8032, yc2: 40000, kind: 18, secs: 1 },
    Door { xc1: 19856, yc1: 17280, xc2: 12416, yc2: 36224, kind: 13, secs: 1 },
    Door { xc1: 19856, yc1: 21664, xc2: 8080, yc2: 40096, kind: 11, secs: 1 },
    Door { xc1: 19936, yc1: 27520, xc2: 10704, yc2: 38528, kind: 10, secs: 1 },
    Door { xc1: 21344, yc1: 22592, xc2: 8800, yc2: 38976, kind: 18, secs: 1 },
    Door { xc1: 21392, yc1: 22688, xc2: 8848, yc2: 39072, kind: 11, secs: 1 },
    Door { xc1: 21600, yc1: 17728, xc2: 7264, yc2: 38976, kind: 18, secs: 1 },
    Door { xc1: 21616, yc1: 25728, xc2: 11392, yc2: 36224, kind: 3, secs: 1 },
    Door { xc1: 21648, yc1: 17824, xc2: 7312, yc2: 39072, kind: 11, secs: 1 },
    Door { xc1: 22000, yc1: 21216, xc2: 5856, yc2: 33760, kind: 10, secs: 1 },
    Door { xc1: 22208, yc1: 21440, xc2: 7104, yc2: 33984, kind: 13, secs: 1 },
    Door { xc1: 22208, yc1: 21568, xc2: 6592, yc2: 34112, kind: 13, secs: 1 },
    Door { xc1: 22256, yc1: 20896, xc2: 6640, yc2: 33440, kind: 13, secs: 1 },
    Door { xc1: 22272, yc1: 21056, xc2: 7664, yc2: 33600, kind: 14, secs: 1 },
    Door { xc1: 22288, yc1: 21568, xc2: 7184, yc2: 34368, kind: 13, secs: 1 },
    Door { xc1: 22320, yc1: 21248, xc2: 6736, yc2: 33792, kind: 13, secs: 1 },
    Door { xc1: 22320, yc1: 21376, xc2: 7216, yc2: 33920, kind: 14, secs: 1 },
    Door { xc1: 22352, yc1: 20896, xc2: 7264, yc2: 33440, kind: 13, secs: 1 },
    Door { xc1: 22352, yc1: 21088, xc2: 8272, yc2: 33632, kind: 13, secs: 1 },
    Door { xc1: 22368, yc1: 21440, xc2: 8288, yc2: 33984, kind: 13, secs: 1 },
    Door { xc1: 22368, yc1: 21568, xc2: 7776, yc2: 34112, kind: 13, secs: 1 },
    Door { xc1: 22624, yc1: 23872, xc2: 7264, yc2: 39488, kind: 18, secs: 1 },
    Door { xc1: 22672, yc1: 23968, xc2: 7312, yc2: 40096, kind: 11, secs: 1 },
    Door { xc1: 22720, yc1: 11872, xc2: 2752, yc2: 34912, kind: 18, secs: 2 },
    Door { xc1: 22880, yc1: 28480, xc2: 8800, yc2: 39488, kind: 18, secs: 1 },
    Door { xc1: 22928, yc1: 28576, xc2: 8848, yc2: 40096, kind: 11, secs: 1 },
    Door { xc1: 22944, yc1: 26464, xc2: 10912, yc2: 35680, kind: 15, secs: 1 },
    Door { xc1: 23008, yc1: 22656, xc2: 10192, yc2: 38528, kind: 10, secs: 1 },
    Door { xc1: 24176, yc1: 6752, xc2: 9600, yc2: 39520, kind: 3, secs: 1 },
    Door { xc1: 24256, yc1: 10592, xc2: 4544, yc2: 35680, kind: 18, secs: 2 },
    Door { xc1: 24672, yc1: 29248, xc2: 6496, yc2: 40000, kind: 18, secs: 1 },
    Door { xc1: 24720, yc1: 29344, xc2: 6544, yc2: 40096, kind: 11, secs: 1 },
    Door { xc1: 24816, yc1: 12992, xc2: 9712, yc2: 35776, kind: 3, secs: 1 },
    Door { xc1: 25792, yc1: 6240, xc2: 960, yc2: 34400, kind: 18, secs: 2 },
    Door { xc1: 25952, yc1: 23872, xc2: 8032, yc2: 39488, kind: 18, secs: 1 },
    Door { xc1: 26000, yc1: 23968, xc2: 8080, yc2: 39072, kind: 11, secs: 1 },
    Door { xc1: 26048, yc1: 6688, xc2: 1200, yc2: 34880, kind: 9, secs: 2 },
    Door { xc1: 26224, yc1: 10848, xc2: 11136, yc2: 39520, kind: 3, secs: 1 },
    Door { xc1: 26624, yc1: 7008, xc2: 10992, yc2: 36960, kind: 9, secs: 1 },
    Door { xc1: 27472, yc1: 17280, xc2: 10320, yc2: 36224, kind: 13, secs: 1 },
    Door { xc1: 27616, yc1: 31872, xc2: 11216, yc2: 38528, kind: 10, secs: 1 },
    Door { xc1: 27760, yc1: 11872, xc2: 10368, yc2: 39520, kind: 3, secs: 1 },
    Door { xc1: 28000, yc1: 26688, xc2: 8032, yc2: 39488, kind: 18, secs: 1 },
    Door { xc1: 28048, yc1: 26784, xc2: 8080, yc2: 39584, kind: 11, secs: 1 },
    Door { xc1: 28384, yc1: 21120, xc2: 12752, yc2: 38528, kind: 10, secs: 1 },
];

/// Rectangles of the world with a meaning: where monsters appear, peaceful places, the carriers.
pub static mut extent_list: [Extent; 23] = [
    Extent { x1: 2118, y1: 27237, x2: 2618, y2: 27637, etype: 70, v1: 0, v2: 1, v3: 11 },
    Extent { x1: 0, y1: 0, x2: 0, y2: 0, etype: 70, v1: 0, v2: 1, v3: 5 },
    Extent { x1: 6749, y1: 34951, x2: 7249, y2: 35351, etype: 70, v1: 0, v2: 1, v3: 10 },
    Extent { x1: 4063, y1: 34819, x2: 4909, y2: 35125, etype: 53, v1: 4, v2: 1, v3: 6 },
    Extent { x1: 9563, y1: 33883, x2: 10144, y2: 34462, etype: 60, v1: 1, v2: 1, v3: 9 },
    Extent { x1: 22945, y1: 5597, x2: 23225, y2: 5747, etype: 61, v1: 3, v2: 2, v3: 4 },
    Extent { x1: 10820, y1: 35646, x2: 10877, y2: 35670, etype: 83, v1: 1, v2: 1, v3: 0 },
    Extent { x1: 19596, y1: 17123, x2: 19974, y2: 17401, etype: 48, v1: 8, v2: 8, v3: 2 },
    Extent { x1: 19400, y1: 17034, x2: 20240, y2: 17484, etype: 80, v1: 4, v2: 20, v3: 0 },
    Extent { x1: 9216, y1: 33280, x2: 12544, y2: 35328, etype: 52, v1: 3, v2: 1, v3: 8 },
    Extent { x1: 5272, y1: 33300, x2: 6112, y2: 34200, etype: 81, v1: 0, v2: 1, v3: 0 },
    Extent { x1: 11712, y1: 37350, x2: 12416, y2: 38020, etype: 82, v1: 0, v2: 1, v3: 0 },
    Extent { x1: 2752, y1: 33300, x2: 8632, y2: 35400, etype: 80, v1: 0, v2: 1, v3: 0 },
    Extent { x1: 10032, y1: 35550, x2: 12976, y2: 40270, etype: 80, v1: 0, v2: 1, v3: 0 },
    Extent { x1: 4712, y1: 38100, x2: 10032, y2: 40350, etype: 80, v1: 0, v2: 1, v3: 0 },
    Extent { x1: 21405, y1: 25583, x2: 21827, y2: 26028, etype: 60, v1: 1, v2: 1, v3: 7 },
    Extent { x1: 6156, y1: 12755, x2: 12316, y2: 15905, etype: 7, v1: 1, v2: 8, v3: 0 },
    Extent { x1: 5140, y1: 34860, x2: 6260, y2: 37260, etype: 8, v1: 1, v2: 8, v3: 0 },
    Extent { x1: 660, y1: 33510, x2: 2060, y2: 34560, etype: 8, v1: 1, v2: 8, v3: 0 },
    Extent { x1: 18687, y1: 15338, x2: 19211, y2: 16136, etype: 80, v1: 0, v2: 1, v3: 0 },
    Extent { x1: 16953, y1: 18719, x2: 20240, y2: 17484, etype: 3, v1: 1, v2: 3, v3: 0 },
    Extent { x1: 20593, y1: 18719, x2: 23113, y2: 22769, etype: 3, v1: 1, v2: 3, v3: 0 },
    Extent { x1: 0, y1: 0, x2: 32767, y2: 40959, etype: 3, v1: 1, v2: 8, v3: 0 },
];

/// The stone ring positions, as sector x, y pairs.
pub static mut stone_ring_sectors: [u8; 22] = [54, 43, 71, 77, 78, 102, 66, 121, 12, 85, 79, 40, 107, 38, 73, 21, 12, 26, 26, 53, 84, 60];

/// Every item: its picture, where it sits on the item list, how many show, and its name.
pub static mut item_list: [InvItem; 36] = [
    InvItem { image_number: 12, xoff: 10, yoff: 0, ydelta: 0, img_off: 0, img_height: 8, maxshown: 1, name: c"Dirk".as_ptr() },
    InvItem { image_number: 9, xoff: 10, yoff: 10, ydelta: 0, img_off: 0, img_height: 8, maxshown: 1, name: c"Mace".as_ptr() },
    InvItem { image_number: 8, xoff: 10, yoff: 20, ydelta: 0, img_off: 0, img_height: 8, maxshown: 1, name: c"Sword".as_ptr() },
    InvItem { image_number: 10, xoff: 10, yoff: 30, ydelta: 0, img_off: 0, img_height: 8, maxshown: 1, name: c"Bow".as_ptr() },
    InvItem { image_number: 17, xoff: 10, yoff: 40, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Magic Wand".as_ptr() },
    InvItem { image_number: 27, xoff: 10, yoff: 50, ydelta: 0, img_off: 0, img_height: 8, maxshown: 1, name: c"Golden Lasso".as_ptr() },
    InvItem { image_number: 23, xoff: 10, yoff: 60, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Sea Shell".as_ptr() },
    InvItem { image_number: 27, xoff: 10, yoff: 70, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Sun Stone".as_ptr() },
    InvItem { image_number: 3, xoff: 30, yoff: 0, ydelta: 3, img_off: 7, img_height: 1, maxshown: 45, name: c"Arrows".as_ptr() },
    InvItem { image_number: 18, xoff: 50, yoff: 0, ydelta: 9, img_off: 0, img_height: 8, maxshown: 15, name: c"Blue Stone".as_ptr() },
    InvItem { image_number: 19, xoff: 65, yoff: 0, ydelta: 6, img_off: 0, img_height: 5, maxshown: 23, name: c"Green Jewel".as_ptr() },
    InvItem { image_number: 22, xoff: 80, yoff: 0, ydelta: 8, img_off: 0, img_height: 7, maxshown: 17, name: c"Glass Vial".as_ptr() },
    InvItem { image_number: 21, xoff: 95, yoff: 0, ydelta: 7, img_off: 0, img_height: 6, maxshown: 20, name: c"Crystal Orb".as_ptr() },
    InvItem { image_number: 23, xoff: 110, yoff: 0, ydelta: 10, img_off: 0, img_height: 9, maxshown: 14, name: c"Bird Totem".as_ptr() },
    InvItem { image_number: 17, xoff: 125, yoff: 0, ydelta: 6, img_off: 0, img_height: 5, maxshown: 23, name: c"Gold Ring".as_ptr() },
    InvItem { image_number: 24, xoff: 140, yoff: 0, ydelta: 10, img_off: 0, img_height: 9, maxshown: 14, name: c"Jade Skull".as_ptr() },
    InvItem { image_number: 25, xoff: 160, yoff: 0, ydelta: 5, img_off: 0, img_height: 5, maxshown: 25, name: c"Gold Key".as_ptr() },
    InvItem { image_number: 25, xoff: 172, yoff: 0, ydelta: 5, img_off: 8, img_height: 5, maxshown: 25, name: c"Green Key".as_ptr() },
    InvItem { image_number: 114, xoff: 184, yoff: 0, ydelta: 5, img_off: 0, img_height: 5, maxshown: 25, name: c"Blue Key".as_ptr() },
    InvItem { image_number: 114, xoff: 196, yoff: 0, ydelta: 5, img_off: 8, img_height: 5, maxshown: 25, name: c"Red Key".as_ptr() },
    InvItem { image_number: 26, xoff: 208, yoff: 0, ydelta: 5, img_off: 0, img_height: 5, maxshown: 25, name: c"Grey Key".as_ptr() },
    InvItem { image_number: 26, xoff: 220, yoff: 0, ydelta: 5, img_off: 8, img_height: 5, maxshown: 25, name: c"White Key".as_ptr() },
    InvItem { image_number: 11, xoff: 0, yoff: 80, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Talisman".as_ptr() },
    InvItem { image_number: 19, xoff: 0, yoff: 90, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Rose".as_ptr() },
    InvItem { image_number: 20, xoff: 0, yoff: 100, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Fruit".as_ptr() },
    InvItem { image_number: 21, xoff: 232, yoff: 0, ydelta: 10, img_off: 8, img_height: 8, maxshown: 5, name: c"Gold Statue".as_ptr() },
    InvItem { image_number: 22, xoff: 0, yoff: 110, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Book".as_ptr() },
    InvItem { image_number: 8, xoff: 14, yoff: 80, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Herb".as_ptr() },
    InvItem { image_number: 9, xoff: 14, yoff: 90, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Writ".as_ptr() },
    InvItem { image_number: 10, xoff: 14, yoff: 100, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Bone".as_ptr() },
    InvItem { image_number: 12, xoff: 14, yoff: 110, ydelta: 0, img_off: 8, img_height: 8, maxshown: 1, name: c"Shard".as_ptr() },
    InvItem { image_number: 0, xoff: 0, yoff: 0, ydelta: 0, img_off: 0, img_height: 0, maxshown: 2, name: c"2 Gold Pieces".as_ptr() },
    InvItem { image_number: 0, xoff: 0, yoff: 0, ydelta: 0, img_off: 0, img_height: 0, maxshown: 5, name: c"5 Gold Pieces".as_ptr() },
    InvItem { image_number: 0, xoff: 0, yoff: 0, ydelta: 0, img_off: 0, img_height: 0, maxshown: 10, name: c"10 Gold Pieces".as_ptr() },
    InvItem { image_number: 0, xoff: 0, yoff: 0, ydelta: 0, img_off: 0, img_height: 0, maxshown: 100, name: c"100 Gold Pieces".as_ptr() },
    InvItem { image_number: 0, xoff: 0, yoff: 0, ydelta: 0, img_off: 0, img_height: 0, maxshown: 0, name: c"quiver of arrows".as_ptr() },
];

/// The mouse pointer sprite.
/// The pointer sprite's image data (20 words, packed as 32 bit values).
pub static pointer_sprite_data: [u32; 20] = [0x0, 0x80060000, 0x60010002, 0x40012000, 0x10030000, 0x8FE0001, 0x581007F, 0x3000081, 0x6010101, 0xC800201, 0x8410401, 0x8200401, 0x8110401, 0x80C0401, 0x880F0401, 0x98074400, 0x75560FFC, 0x0, 0x0, 0x0];

/// The text panel colours.
pub static text_colors: [u16; 20] = [0x000, 0xFFF, 0xC00, 0xF60, 0x00F, 0xC0F, 0x090, 0xFF0, 0xF90, 0xF0C, 0xA50, 0xFDB, 0xEB7, 0xCCC, 0x888, 0x444, 0x000, 0xDB0, 0x740, 0xC70];

/// The intro page colours.
pub static intro_colors: [u16; 32] = [0x000, 0xFFF, 0xE00, 0xA00, 0xD80, 0xEC0, 0x390, 0x021, 0xEEB, 0xEDA, 0xEEA, 0xCB8, 0xA95, 0x973, 0x840, 0x620, 0xA52, 0xC74, 0xD96, 0xFCA, 0x449, 0x444, 0xDC9, 0x668, 0x33F, 0x888, 0xA60, 0xAAF, 0xBBB, 0xCCF, 0xDDD, 0xEEE];

/// All black, for fading out.
pub static black_colors: [u16; 32] = [0; 32];

/// Scratch space for printing numbers (ends with a space).
pub static mut number_buffer: [i8; 11] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32];

// The menu labels: fixed width fields of 5 characters, one after another.
pub static mut label1: [u8; 26] = *b"ItemsMagicTalk Buy  Game \0";
pub static mut label2: [u8; 26] = *b"List Take Look Use  Give \0";
pub static mut label3: [u8; 16] = *b"Yell Say  Ask  \0";
pub static mut label4: [u8; 26] = *b"PauseMusicSoundQuit Load \0";
pub static mut label5: [u8; 36] = *b"Food ArrowVial Mace SwordBow  Totem\0";
pub static mut label6: [u8; 36] = *b"StoneJewelVial Orb  TotemRing Skull\0";
pub static mut label7: [u8; 51] = *b"Dirk Mace SwordBow  Wand LassoShellKey  Sun  Book \0";
pub static mut label8: [u8; 11] = *b"Save Exit \0";
pub static mut label9: [u8; 31] = *b"Gold GreenBlue Red  Grey White\0";
pub static mut labelA: [u8; 21] = *b"Gold Book Writ Bone \0";
pub static mut labelB: [u8; 41] = *b"  A    B    C    D    E    F    G    H  \0";

/// The colour of each key in the key menu.
pub static mut key_colors: [i8; 6] = [8, 6, 4, 2, 14, 1];

/// The ten menus: labels, entry count, colour, and each entry's state (see `Menu`).
pub static mut menus: [Menu; 10] = [
    Menu { label_list: &raw const label2 as *const c_char, num: 10, color: 6, enabled: [3, 2, 2, 2, 2, 10, 10, 10, 10, 10, 0, 0] },
    Menu { label_list: &raw const label6 as *const c_char, num: 12, color: 5, enabled: [2, 3, 2, 2, 2, 8, 8, 8, 8, 8, 8, 8] },
    Menu { label_list: &raw const label3 as *const c_char, num: 8, color: 9, enabled: [2, 2, 3, 2, 2, 10, 10, 10, 0, 0, 0, 0] },
    Menu { label_list: &raw const label5 as *const c_char, num: 12, color: 10, enabled: [2, 2, 2, 3, 2, 10, 10, 10, 10, 10, 10, 10] },
    Menu { label_list: &raw const label4 as *const c_char, num: 10, color: 2, enabled: [2, 2, 2, 2, 3, 6, 7, 7, 10, 10, 0, 0] },
    Menu { label_list: &raw const label8 as *const c_char, num: 7, color: 0, enabled: [2, 2, 2, 2, 2, 10, 10, 0, 0, 0, 0, 0] },
    Menu { label_list: &raw const label9 as *const c_char, num: 11, color: 8, enabled: [2, 2, 2, 2, 2, 10, 10, 10, 10, 10, 10, 0] },
    Menu { label_list: &raw const labelA as *const c_char, num: 9, color: 10, enabled: [2, 2, 2, 2, 2, 10, 0, 0, 0, 0, 0, 0] },
    Menu { label_list: &raw const label7 as *const c_char, num: 10, color: 8, enabled: [10, 10, 10, 10, 10, 10, 10, 10, 10, 0, 10, 10] },
    Menu { label_list: &raw const labelB as *const c_char, num: 10, color: 5, enabled: [10, 10, 10, 10, 10, 10, 10, 10, 0, 0, 0, 0] },
];

/// Keyboard shortcuts: the key, the menu and the entry it picks.
pub static mut menu_shortcuts: [Letter; 37] = [
    Letter { letter: 73, menu: 0, choice: 5 },
    Letter { letter: 84, menu: 0, choice: 6 },
    Letter { letter: 63, menu: 0, choice: 7 },
    Letter { letter: 85, menu: 0, choice: 8 },
    Letter { letter: 71, menu: 0, choice: 9 },
    Letter { letter: 89, menu: 2, choice: 5 },
    Letter { letter: 83, menu: 2, choice: 6 },
    Letter { letter: 65, menu: 2, choice: 7 },
    Letter { letter: 32, menu: 4, choice: 5 },
    Letter { letter: 77, menu: 4, choice: 6 },
    Letter { letter: 70, menu: 4, choice: 7 },
    Letter { letter: 81, menu: 4, choice: 8 },
    Letter { letter: 76, menu: 4, choice: 9 },
    Letter { letter: 79, menu: 3, choice: 5 },
    Letter { letter: 82, menu: 3, choice: 6 },
    Letter { letter: 56, menu: 3, choice: 7 },
    Letter { letter: 67, menu: 3, choice: 8 },
    Letter { letter: 87, menu: 3, choice: 9 },
    Letter { letter: 66, menu: 3, choice: 10 },
    Letter { letter: 69, menu: 3, choice: 11 },
    Letter { letter: 86, menu: 5, choice: 5 },
    Letter { letter: 88, menu: 5, choice: 6 },
    Letter { letter: 10, menu: 1, choice: 5 },
    Letter { letter: 11, menu: 1, choice: 6 },
    Letter { letter: 12, menu: 1, choice: 7 },
    Letter { letter: 13, menu: 1, choice: 8 },
    Letter { letter: 14, menu: 1, choice: 9 },
    Letter { letter: 15, menu: 1, choice: 10 },
    Letter { letter: 16, menu: 1, choice: 11 },
    Letter { letter: 49, menu: 8, choice: 0 },
    Letter { letter: 50, menu: 8, choice: 1 },
    Letter { letter: 51, menu: 8, choice: 2 },
    Letter { letter: 52, menu: 8, choice: 3 },
    Letter { letter: 53, menu: 8, choice: 4 },
    Letter { letter: 54, menu: 8, choice: 5 },
    Letter { letter: 55, menu: 8, choice: 6 },
    Letter { letter: 75, menu: 8, choice: 7 },
];

/// The brothers' names.
pub static mut brother_names: [*const c_char; 3] = [c"Julian".as_ptr(), c"Phillip".as_ptr(), c"Kevin".as_ptr()];

/// The disk blocks each of the ten map regions needs.
pub static mut region_disk_blocks: [Need; 10] = [
    Need { image: [320, 480, 520, 560], terra1: 0, terra2: 1, sector: 32, region: 160, setchar: 22 },
    Need { image: [320, 360, 400, 440], terra1: 2, terra2: 3, sector: 32, region: 160, setchar: 21 },
    Need { image: [320, 360, 520, 560], terra1: 2, terra2: 1, sector: 32, region: 168, setchar: 22 },
    Need { image: [320, 360, 400, 440], terra1: 2, terra2: 3, sector: 32, region: 168, setchar: 21 },
    Need { image: [320, 480, 520, 600], terra1: 0, terra2: 4, sector: 32, region: 176, setchar: 0 },
    Need { image: [320, 280, 240, 200], terra1: 5, terra2: 6, sector: 32, region: 176, setchar: 23 },
    Need { image: [320, 640, 520, 600], terra1: 7, terra2: 4, sector: 32, region: 184, setchar: 0 },
    Need { image: [320, 280, 240, 200], terra1: 5, terra2: 6, sector: 32, region: 184, setchar: 24 },
    Need { image: [680, 720, 800, 840], terra1: 8, terra2: 9, sector: 96, region: 192, setchar: 0 },
    Need { image: [680, 760, 800, 840], terra1: 10, terra2: 9, sector: 96, region: 192, setchar: 0 },
];

/// What the region loader has loaded so far (starts as "nothing useful").
pub static mut loaded_blocks: Need = Need { image: [0, 0, 0, 0], terra1: 1, terra2: 2, sector: 0, region: 0, setchar: 0 };

/// The font the text panel asks for.
/// The instrument waveforms for the music, as pairs of values.
pub static mut instrument_waves: [i16; 12] = [0x0000, 0x0000, 0x0000, 0x0000, 0x0005, 0x0202, 0x0101, 0x0103, 0x0004, 0x0504, 0x0100, 0x0500];

/// The random number seeds the game starts with.
pub static mut seed1: i32 = 19837325;
pub static mut seed2: i32 = 23098324;

/// The direction the compass shows (9 = none).
pub static mut compass_lit_direction: i16 = 9;

/// The first sprite image of each facing (16 entries: walking, then fighting).
pub static mut facing_frame_offsets: [i8; 16] = [16, 16, 24, 24, 0, 0, 8, 8, 56, 56, 68, 68, 32, 32, 44, 44];

/// The brothers' starting stats and their item lists.
pub static mut brother_stats: [Bro; 3] = unsafe {
    [
    Bro { brave: 35, luck: 20, kind: 15, wealth: 20, stuff: &raw mut the_hero.julstuff as *mut u8 },
    Bro { brave: 20, luck: 35, kind: 15, wealth: 15, stuff: &raw mut the_hero.philstuff as *mut u8 },
    Bro { brave: 15, luck: 20, kind: 35, wealth: 10, stuff: &raw mut the_hero.kevstuff as *mut u8 },
    ]
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::collections::HashSet;

    #[test]
    fn doors_are_sorted_by_x_for_the_binary_search() {
        let doors = unsafe { &*(&raw const door_list) };

        for pair in doors.windows(2) {
            assert!(pair[0].xc1 <= pair[1].xc1, "door at x {} is out of order", pair[1].xc1);
        }
    }

    /// The "around village" extent (20) has its top below its bottom in the original data, so no
    /// point is ever inside it. The port keeps that as it was.
    const INVERTED_EXTENT: usize = 20;

    #[test]
    fn extents_are_proper_rectangles() {
        let extents = unsafe { &*(&raw const extent_list) };

        for (n, e) in extents.iter().enumerate() {
            if n == INVERTED_EXTENT {
                assert!(e.y1 > e.y2, "the quirk of extent {n} is gone; update this test");
            } else {
                assert!(e.x1 <= e.x2 && e.y1 <= e.y2, "extent {n}");
            }
        }
    }

    #[test]
    fn menu_labels_cover_all_the_entries() {
        // Entries 0-4 are the menu names; every entry after that has a 5 character label.
        let all_menus = unsafe { &*(&raw const menus) };

        for (n, menu) in all_menus.iter().enumerate() {
            let label = unsafe { CStr::from_ptr(menu.label_list) };
            let needed = (menu.num as usize).saturating_sub(5) * 5;
            assert!(label.to_bytes().len() >= needed, "menu {n}");
            assert!(menu.num as usize <= menu.enabled.len(), "menu {n}");
        }
    }

    #[test]
    fn every_shortcut_key_picks_a_real_entry() {
        let all_menus = unsafe { &*(&raw const menus) };

        let shortcuts = unsafe { &*(&raw const menu_shortcuts) };
        let mut seen = HashSet::new();

        for shortcut in shortcuts {
            let menu = &all_menus[shortcut.menu as usize];
            assert!(shortcut.choice < menu.num, "key {}", shortcut.letter);
            assert!(seen.insert((shortcut.letter, shortcut.menu)), "key {} twice", shortcut.letter);
        }
    }

    #[test]
    fn swing_transitions_lead_to_swing_states() {
        let transitions = unsafe { &*(&raw const swing_transitions) };

        for (state, next) in transitions.iter().enumerate() {
            for &to in next {
                assert!((0..=8).contains(&to), "state {state} goes to {to}");
            }
        }
    }

    #[test]
    fn every_item_has_a_name() {
        let items = unsafe { &*(&raw const item_list) };
        assert_eq!(items.len(), 36);

        for (n, item) in items.iter().enumerate() {
            let name = unsafe { CStr::from_ptr(item.name) };
            assert!(!name.to_bytes().is_empty(), "item {n}");
        }

        assert_eq!(unsafe { CStr::from_ptr(items[0].name) }.to_str(), Ok("Dirk"));
    }

    #[test]
    fn the_brothers_have_their_own_item_lists() {
        let brothers = unsafe { &*(&raw const brother_stats) };
        let lists: HashSet<_> = brothers.iter().map(|b| b.stuff as usize).collect();
        assert_eq!(lists.len(), 3);

        let names = unsafe { &*(&raw const brother_names) };
        assert_eq!(unsafe { CStr::from_ptr(names[0]) }.to_str(), Ok("Julian"));
    }

    #[test]
    fn monsters_use_the_enemy_sprite_files() {
        let chart = unsafe { &*(&raw const encounter_chart) };

        for (n, monster) in chart.iter().enumerate() {
            assert!((6..=9).contains(&monster.file_id), "monster {n}");
        }
    }
}
