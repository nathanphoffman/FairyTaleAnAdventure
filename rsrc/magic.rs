//! The Magic menu actions (from `choose_menu_entry` in csrc/fmain.c): casting the spells the hero has
//! stones, vials and so on for. Each use takes one from the stack.

use super::asm_map::draw_overview_map;
use super::asm_text::{queue_print, show_event, speak};
use super::encounters::check_death;
use super::game_tables::{region_num, stone_ring_sectors};
use super::globals::{
    active_actor_count, actor_list, battle_flag, bm_draw, brave, carrier_slot, cheat_enabled,
    current_extent, drawing_page, freeze_timer, hero_sector, hero_x, hero_y, light_timer, map_x,
    map_y, planes, region_origin_x, region_origin_y, riding, rp_map, secret_timer, secx, secy,
    stuff, view_status,
};
use super::menus::update_menu_availability;
use super::messages::print;
use super::places::transfer_hero;
use super::random::rand8;
use super::screen_fx::{flicker_colors, show_still_screen};
use crate::amiga::RastPort;
use std::ffi::c_char;

unsafe extern "C" {
    fn SetDrMd(current_rastport: *mut RastPort, mode: u32);
    fn SetAPen(current_rastport: *mut RastPort, pen: u32);
    fn Move(current_rastport: *mut RastPort, x: i32, y: i32);
    fn Text(current_rastport: *mut RastPort, s: *const c_char, len: i32) -> i32;
}

// The menu entries: the spells are entries 5-11 and use the item slots 9-15 (`4 + entry`).
const SPELL_STONE: i16 = 5;
const SPELL_LIGHT: i16 = 6;
const SPELL_HEAL: i16 = 7;
const SPELL_SECRET: i16 = 8;
const SPELL_MAP: i16 = 9;
const SPELL_FREEZE: i16 = 10;
const SPELL_FEAR: i16 = 11;
const FIRST_SPELL: i16 = 5;
const SPELL_SLOT_BASE: usize = 4;

const LIGHT_TIME: i16 = 760;
const SECRET_TIME: i16 = 360;
const FREEZE_TIME: i16 = 100;
/// Where the spells cannot be used: the extent type with this `v3` (a quiet place).
const NO_MAGIC_ZONE: u8 = 9;
const NO_MAGIC_SPEECH: i32 = 59;
const NO_SPELL_EVENT: i32 = 21;
const BATTLE_EVENT: i32 = 34;
/// Regions above this are indoors (no overview map there).
const LAST_OUTDOOR_REGION: u16 = 7;
/// Riding something bigger than the raft stops the freeze spell.
const RIDING_BIG: i16 = 1;

// The stone circle: sector, and where in the sector the hero must stand.
const STONE_SECTOR: i16 = 144;
const STONE_COLUMN_WIDTH: u16 = 85;
const STONE_ROW_HEIGHT: u16 = 64;
const STONE_CIRCLES: usize = 11;

// The overview map and where the hero's position is marked.
const MAP_MARK_PEN: u32 = 31;
const JAM1: u32 = 0;
const MARK_X_OFFSET: i64 = 4;
const MARK_Y_OFFSET: i64 = 3;
const MARK_MAX: (u32, u32) = (320, 143);
const VIEW_MAP: i8 = 1;

const PRQ_STATS: i32 = 4;
const PRQ_MAP: i32 = 5;
/// Healing is capped at this plus a quarter of bravery.
const HEAL_CAP_BASE: i16 = 15;
const HEAL_MIN: i32 = 4;
/// The fear spell only hits enemies of races below this.
const FEAR_RACE_LIMIT: u8 = 7;
const ENEMY: i8 = 2;

/// Handles an entry of the Magic menu. Returns false if `update_menu_availability` should not follow
/// (the spell did not work, so the stone is not used up).
pub unsafe fn magic_option(chosen_entry: i16) -> bool {
    unsafe {
        if chosen_entry < FIRST_SPELL || *stuff.add(SPELL_SLOT_BASE + chosen_entry as usize) == 0 {
            show_event(NO_SPELL_EVENT);
            return true;
        }
        if (*current_extent).v3 == NO_MAGIC_ZONE {
            speak(NO_MAGIC_SPEECH);
            return true;
        }
        match chosen_entry {
            SPELL_LIGHT => light_timer += LIGHT_TIME,
            SPELL_SECRET => secret_timer += SECRET_TIME,
            SPELL_FREEZE => {
                if riding > RIDING_BIG {
                    return false;
                }
                freeze_timer += FREEZE_TIME;
            }
            SPELL_MAP => {
                if cheat_enabled == 0 && region_num > LAST_OUTDOOR_REGION {
                    return false;
                }
                show_map();
            }
            SPELL_STONE => {
                if !stone_jump() {
                    return false;
                }
                heal();
            }
            SPELL_HEAL => heal(),
            SPELL_FEAR => scare_enemies(),
            _ => {}
        }
        let slot = SPELL_SLOT_BASE + chosen_entry as usize;
        *stuff.add(slot) = (*stuff.add(slot)).wrapping_sub(1);
        if *stuff.add(slot) == 0 {
            update_menu_availability();
        }
        true
    }
}

/// Draws the overview map with a "+" where the hero is, and shows it.
unsafe fn show_map() {
    unsafe {
        bm_draw = (*(*drawing_page).ri_page).BitMap;
        planes = (*bm_draw).Planes.as_mut_ptr();
        draw_overview_map(map_x as i32, map_y as i32);

        let x = ((hero_x >> 4) as i64
            - ((secx as i64 + region_origin_x as i64) << 4)
            - MARK_X_OFFSET) as u32;
        let y = ((hero_y >> 4) as i64 - ((secy as i64 + region_origin_y as i64) << 4)
            + MARK_Y_OFFSET) as u32;
        rp_map.BitMap = bm_draw;
        SetDrMd(&raw mut rp_map, JAM1);
        SetAPen(&raw mut rp_map, MAP_MARK_PEN);
        if x > 0 && x < MARK_MAX.0 && y > 0 && y < MARK_MAX.1 {
            Move(&raw mut rp_map, x as i32, y as i32);
            Text(&raw mut rp_map, c"+".as_ptr(), 1);
        }
        view_status = VIEW_MAP;
        show_still_screen();
        queue_print(PRQ_MAP);
    }
}

/// The stone ring spell: only works standing in the middle of a stone circle. Takes the hero to
/// the next circle in the direction faced. Returns false if the spell did not work.
unsafe fn stone_jump() -> bool {
    unsafe {
        if hero_sector != STONE_SECTOR {
            return false;
        }
        if (hero_x & 255) / STONE_COLUMN_WIDTH != 1 || (hero_y & 255) / STONE_ROW_HEIGHT != 1 {
            return false;
        }
        let (sector_x, sector_y) = (hero_x >> 8, hero_y >> 8);
        for ring in 0..STONE_CIRCLES {
            if stone_ring_sectors[ring * 2] as u16 == sector_x
                && stone_ring_sectors[ring * 2 + 1] as u16 == sector_y
            {
                let mut target = ring + (actor_list[0].facing + 1) as usize;
                if target > STONE_CIRCLES - 1 {
                    target -= STONE_CIRCLES;
                }
                let x = ((stone_ring_sectors[target * 2] as u16) << 8) + (hero_x & 255);
                let y = ((stone_ring_sectors[target * 2 + 1] as u16) << 8) + (hero_y & 255);
                flicker_colors();
                transfer_hero(x, y, 1);
                if riding != 0 {
                    actor_list[carrier_slot as usize].abs_x = actor_list[0].abs_x;
                    actor_list[carrier_slot as usize].abs_y = actor_list[0].abs_y;
                }
                break;
            }
        }
        true
    }
}

/// Heals the hero a little, up to a limit set by bravery.
unsafe fn heal() {
    unsafe {
        let cap = HEAL_CAP_BASE + brave / 4;
        let hero = &raw mut actor_list[0];
        (*hero).vitality = (*hero).vitality.wrapping_add((rand8() + HEAL_MIN) as i16);
        if (*hero).vitality > cap {
            (*hero).vitality = cap;
        } else {
            print(c"That feels a lot better!".as_ptr());
        }
        queue_print(PRQ_STATS);
    }
}

/// Kills every weak enemy on the screen (and costs a little bravery for each).
unsafe fn scare_enemies() {
    unsafe {
        let mut i = 1;
        while i < active_actor_count as usize {
            let an = &raw mut actor_list[i];
            if (*an).vitality != 0 && (*an).kind == ENEMY && (*an).race < FEAR_RACE_LIMIT {
                (*an).vitality = 0;
                check_death(i as i32, 0);
                brave -= 1;
            }
            i += 1;
        }
        if battle_flag != 0 {
            show_event(BATTLE_EVENT);
        }
    }
}
