//! Moving the hero somewhere new, and working out where the hero is (from csrc/fmain.c).
//!
//! `find_place` looks up the sector the hero stands on in a table to get the place name message
//! ("You are in a village" ...), and looks the position up in `extent_list` to learn what kind of
//! area it is (`extent_type`): safe, a monster zone, a boss, a bird or turtle area, and so on.
//! Stepping into some of them starts something: monsters appear, or a carrier is loaded.

use super::actors::{blocked_at, rescue_princess};
use super::asm_text::show_message;
use super::encounters::load_enemy_sprites;
use super::encounters::{load_carrier, place_encounter};
use super::game_tables::{extent_list, region_num};
use super::globals::{
    active_actor_count, active_carrier, actor_list, actors_loading, current_extent,
    encounter_number, encounter_type, encounter_x, encounter_y, extent_type, held_direction,
    hero_place, hero_sector, hero_x, hero_y, j as c_j, loaded_actor_file, map_x, map_y, mixflag,
    new_region, region_origin_x, region_origin_y, riding, view_status, wt,
};
use super::hdrive::motor_off;
use super::loading::finish_sprite_load;
use super::mood::set_mood;
use super::regions::{load_region_now, refresh_minimap};
use super::tables::ob_list8;
use super::types::Extent;
use std::ffi::c_char;

unsafe extern "C" {
    static place_tbl: [u8; 0];
    static inside_tbl: [u8; 0];
    static place_msg: [c_char; 0];
    static inside_msg: [c_char; 0];
}

/// The first region number that is indoors.
const FIRST_INDOOR_REGION: u16 = 8;
const NO_REGION: u16 = 10;
/// The place message that stands for "the mountains", which depends on the region.
const MOUNTAINS_MESSAGE: u8 = 4;
const NORTHERN_MOUNTAINS_MESSAGE: u8 = 0;
const SOUTHERN_MOUNTAINS_MESSAGE: u8 = 5;
/// How many extents there are (the last one covers the whole world).
const EXTENT_COUNT: usize = 22;
/// Sector numbers inside buildings are offset by this in the place table.
const INSIDE_SECTOR_OFFSET: i16 = 256;

// Extent types (`etype`).
const XT_TURTLE_EGGS: u16 = 61;
const XT_NECROMANCER: u16 = 60;
const XT_SPIDER_PIT: u16 = 52;
const XT_MONSTER_ZONE_MIN: u16 = 50;
const XT_PRINCESS: u16 = 83;
const XT_SPECIAL_MIN: u16 = 60;
const XT_CARRIER_MIN: u16 = 70;
const XT_CARRIER: u16 = 70;
/// Slot in `ob_list8` of the princess object.
const PRINCESS_OBJECT: usize = 9;
/// Slot of the first monster in `actor_list`, and the first slot after the monsters.
const FIRST_MONSTER: i16 = 3;
const MONSTER_SLOTS_END: i16 = 7;
/// How far from the encounter spot monsters are placed.
const ENCOUNTER_SPREAD: u16 = 63;
const ENEMY_SLOT: i16 = 2;
/// Encounter type used in the spider pit.
const SPIDER_ENCOUNTER: u16 = 8;

/// Moves the hero to the world position (x, y) and reloads what is needed there. With `flag`
/// set the region is worked out from the position (like walking through a door); otherwise
/// `new_region` must already be set. If the spot is blocked the hero is moved down until it is free.
#[unsafe(export_name = "xfer")]
pub extern "C" fn transfer_hero(xtest: u16, ytest: u16, flag: u16) -> i32 {
    unsafe {
        map_x = map_x.wrapping_add(xtest.wrapping_sub(hero_x));
        map_y = map_y.wrapping_add(ytest.wrapping_sub(hero_y));
        hero_x = xtest;
        actor_list[0].abs_x = xtest;
        hero_y = ytest;
        actor_list[0].abs_y = ytest;

        encounter_number = 0;
        if flag != 0 {
            let sector_x = (map_x as u32 + 151) >> 8;
            let sector_y = (map_y as u32 + 64) >> 8;
            let column = (sector_x >> 6) & 1; // 64 sectors across
            let row = (sector_y >> 5) & 7; // 32 sectors down
            new_region = (column + row + row) as u16;
        }
        held_direction = 0;
        load_region_now();
        refresh_minimap(); // to set region_origin_x and region_origin_y
        view_status = 99;
        set_mood(1);
        while blocked_at(hero_x as i16, hero_y as i16, 0) != 0 {
            hero_y = hero_y.wrapping_add(1);
        }
        0
    }
}

/// Starts the monsters of the extent the hero is in: loads their pictures and places up to four
/// of them (there must be room before the carrier slots).
unsafe fn start_forced_encounter() {
    unsafe {
        encounter_type = (*current_extent).v3 as u16;
        mixflag = 0;
        wt = 0;
        load_enemy_sprites();
        finish_sprite_load(ENEMY_SLOT);
        motor_off();
        actors_loading = 0;
        encounter_number = (*current_extent).v1;
        active_actor_count = FIRST_MONSTER;
        while encounter_number != 0 && active_actor_count < MONSTER_SLOTS_END {
            if place_encounter(active_actor_count as u16, ENCOUNTER_SPREAD) != 0 {
                active_actor_count += 1;
            }
            encounter_number -= 1;
        }
    }
}

/// Works out where the hero is: the place name (said when it changes and `flag` is set), then the
/// extent the hero stands in, and whatever that extent starts.
///
/// `flag` 0 is silent, 1 also starts monster zones, 2 only says the place name.
pub fn find_place(flag: i16) -> i32 {
    unsafe {
        let mut flag = flag;
        loop {
            announce_place(flag);
            locate_extent();
            if !enter_extent(flag) {
                // the princess was rescued: look again from the top, without the messages
                flag = 0;
                continue;
            }
            update_carrier();
            return 0;
        }
    }
}

/// The place-name message number for the hero's sector, or 0 for none. Mountains get the name of
/// the part of the map they are in, and nothing is said while the next region is still loading or
/// when the hero is on the edge of the loaded one.
unsafe fn place_message_number() -> u8 {
    unsafe {
        hero_sector &= 255;
        c_j = hero_sector;
        let sector = hero_sector as u8;
        let table = if region_num >= FIRST_INDOOR_REGION {
            hero_sector += INSIDE_SECTOR_OFFSET;
            inside_tbl.as_ptr()
        } else {
            place_tbl.as_ptr()
        };

        // find the table row (three bytes: first sector, last sector, message) that holds this sector
        let mut row = table;
        for _ in 0..256 {
            if sector >= *row && sector <= *row.add(1) {
                break;
            }
            row = row.add(3);
        }
        let mut message = *row.add(2);
        if message == MOUNTAINS_MESSAGE {
            if region_num >= FIRST_INDOOR_REGION {
            } else if region_num & 1 != 0 {
                message = NORTHERN_MOUNTAINS_MESSAGE;
            } else if region_num > 3 {
                message = SOUTHERN_MOUNTAINS_MESSAGE;
            }
        }
        let hero_column = (hero_x >> 8).wrapping_sub(region_origin_x);
        let hero_row = (hero_y >> 8).wrapping_sub(region_origin_y);
        if new_region < NO_REGION || hero_column & 64 != 0 || hero_row & 32 != 0 {
            message = 0;
        }
        message
    }
}

/// Remembers the place the hero is in, and says its name when it changes (if `flag` is set).
unsafe fn announce_place(flag: i16) {
    unsafe {
        let message = place_message_number();
        if message != 0 && message as u16 != hero_place {
            hero_place = message as u16;
            if flag != 0 {
                let messages = if region_num >= FIRST_INDOOR_REGION {
                    inside_msg.as_ptr()
                } else {
                    place_msg.as_ptr()
                };
                show_message(messages, message as i32);
            }
        }
    }
}

/// Finds the first extent (special rectangle of the world) the hero is in.
unsafe fn locate_extent() {
    unsafe {
        current_extent = (&raw mut extent_list).cast::<Extent>();
        for _ in 0..EXTENT_COUNT {
            let e = &*current_extent;
            if hero_x > e.x1 && hero_x < e.x2 && hero_y > e.y1 && hero_y < e.y2 {
                break;
            }
            current_extent = current_extent.add(1);
        }
    }
}

/// What happens when the hero walks into a different kind of extent: the princess is rescued, a
/// boss or the turtle eggs call up their monsters, the spider pit loads the spiders, and monster
/// zones may send a fight. Returns false if the princess was rescued (the caller starts over).
unsafe fn enter_extent(flag: i16) -> bool {
    unsafe {
        if extent_type == (*current_extent).etype as u16 {
            return true;
        }
        extent_type = (*current_extent).etype as u16;
        let mut force = false;
        if extent_type == XT_PRINCESS && ob_list8[PRINCESS_OBJECT].ob_stat != 0 {
            rescue_princess();
            return false;
        } else if extent_type >= XT_SPECIAL_MIN {
            // birds, turtles, special
            if (extent_type == XT_NECROMANCER || extent_type == XT_TURTLE_EGGS)
                && (actor_list[FIRST_MONSTER as usize].race != (*current_extent).v3
                    || active_actor_count < 4)
            {
                encounter_x =
                    (((*current_extent).x1 as u32 + (*current_extent).x2 as u32) / 2) as u16;
                encounter_y =
                    (((*current_extent).y1 as u32 + (*current_extent).y2 as u32) / 2) as u16;
                force = true;
            }
        } else if extent_type == XT_SPIDER_PIT {
            encounter_type = SPIDER_ENCOUNTER;
            load_enemy_sprites();
            finish_sprite_load(ENEMY_SLOT);
            motor_off();
            actors_loading = 0;
        } else if extent_type >= XT_MONSTER_ZONE_MIN && flag == 1 {
            encounter_x = hero_x;
            encounter_y = hero_y;
            force = true;
        }
        if force {
            start_forced_encounter();
        }
        true
    }
}

/// Drops the carrier when the hero is outside the carrier extents, and loads the swan, turtle or
/// dragon inside them.
unsafe fn update_carrier() {
    unsafe {
        if extent_type < XT_CARRIER_MIN {
            active_carrier = 0;
        } else if extent_type == XT_CARRIER
            && (active_carrier == 0
                || (riding == 0 && loaded_actor_file != (*current_extent).v3 as i16))
        {
            load_carrier((*current_extent).v3 as i16);
        }
    }
}
