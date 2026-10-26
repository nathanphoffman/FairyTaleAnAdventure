//! Moving the hero somewhere new, and working out where the hero is (from csrc/fmain.c).
//!
//! `find_place` looks up the sector the hero stands on in a table to get the place name message
//! ("You are in a village" ...), and looks the position up in `extent_list` to learn what kind of
//! area it is (`xtype`): safe, a monster zone, a boss, a bird or turtle area, and so on.
//! Stepping into some of them starts something: monsters appear, or a carrier is loaded.

use super::actors::{proxcheck, rescue};
use super::asm_text::msg;
use super::encounters::{load_carrier, set_encounter};
use super::loading::prep;
use super::mood::setmood;
use super::regions::{gen_mini, load_all};
use super::tables::ob_list8;
use super::types::{Extent, Shape};
use std::ffi::c_char;

unsafe extern "C" {
    static mut map_x: u16;
    static mut map_y: u16;
    static mut hero_x: u16;
    static mut hero_y: u16;
    static xreg: u16;
    static yreg: u16;
    static region_num: u16;
    static mut new_region: u16;
    static mut keydir: i16;
    static mut viewstatus: i8;
    static mut encounter_number: u8;
    static mut encounter_x: u16;
    static mut encounter_y: u16;
    static mut encounter_type: u16;
    static mut hero_sector: i16;
    static mut hero_place: u16;
    static mut xtype: u16;
    static mut extn: *mut Extent;
    static mut extent_list: [Extent; 0];
    static mut anim_list: [Shape; 20];
    static mut anix: i16;
    static mut mixflag: i16;
    static mut wt: i16;
    static mut actors_loading: i8;
    static mut active_carrier: i16;
    static riding: i16;
    static actor_file: i16;
    static place_tbl: [u8; 0];
    static inside_tbl: [u8; 0];
    static place_msg: [c_char; 0];
    static inside_msg: [c_char; 0];
    #[link_name = "j"]
    static mut c_j: i16;

    fn load_actors();
    fn motor_off();
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
/// Slot of the first monster in `anim_list`, and the first slot after the monsters.
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
#[unsafe(no_mangle)]
pub extern "C" fn xfer(xtest: u16, ytest: u16, flag: u16) -> i32 {
    unsafe {
        map_x = map_x.wrapping_add(xtest.wrapping_sub(hero_x));
        map_y = map_y.wrapping_add(ytest.wrapping_sub(hero_y));
        hero_x = xtest;
        anim_list[0].abs_x = xtest;
        hero_y = ytest;
        anim_list[0].abs_y = ytest;

        encounter_number = 0;
        if flag != 0 {
            let sector_x = (map_x as u32 + 151) >> 8;
            let sector_y = (map_y as u32 + 64) >> 8;
            let column = (sector_x >> 6) & 1; // 64 sectors across
            let row = (sector_y >> 5) & 7; // 32 sectors down
            new_region = (column + row + row) as u16;
        }
        keydir = 0;
        load_all();
        gen_mini(); // to set xreg and yreg
        viewstatus = 99;
        setmood(1);
        while proxcheck(hero_x as i16, hero_y as i16, 0) != 0 {
            hero_y = hero_y.wrapping_add(1);
        }
        0
    }
}

/// Starts the monsters of the extent the hero is in: loads their pictures and places up to four
/// of them (there must be room before the carrier slots).
unsafe fn start_forced_encounter() {
    unsafe {
        encounter_type = (*extn).v3 as u16;
        mixflag = 0;
        wt = 0;
        load_actors();
        prep(ENEMY_SLOT);
        motor_off();
        actors_loading = 0;
        encounter_number = (*extn).v1;
        anix = FIRST_MONSTER;
        while encounter_number != 0 && anix < MONSTER_SLOTS_END {
            if set_encounter(anix as u16, ENCOUNTER_SPREAD) != 0 {
                anix += 1;
            }
            encounter_number -= 1;
        }
    }
}

/// Works out where the hero is: the place name (said when it changes and `flag` is set), then the
/// extent the hero stands in, and whatever that extent starts.
///
/// `flag` 0 is silent, 1 also starts monster zones, 2 only says the place name.
#[unsafe(no_mangle)]
pub extern "C" fn find_place(flag: i16) -> i32 {
    unsafe {
        let mut flag = flag;
        loop {
            hero_sector &= 255;
            c_j = hero_sector;
            let sector = hero_sector as u8;
            let (table, messages) = if region_num >= FIRST_INDOOR_REGION {
                hero_sector += INSIDE_SECTOR_OFFSET;
                (inside_tbl.as_ptr(), inside_msg.as_ptr())
            } else {
                (place_tbl.as_ptr(), place_msg.as_ptr())
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
            let hero_column = (hero_x >> 8).wrapping_sub(xreg);
            let hero_row = (hero_y >> 8).wrapping_sub(yreg);
            if new_region < NO_REGION || hero_column & 64 != 0 || hero_row & 32 != 0 {
                message = 0;
            }
            if message != 0 && message as u16 != hero_place {
                hero_place = message as u16;
                if flag != 0 {
                    msg(messages, message as i32);
                }
            }

            extn = (&raw mut extent_list).cast::<Extent>();
            for _ in 0..EXTENT_COUNT {
                let e = &*extn;
                if hero_x > e.x1 && hero_x < e.x2 && hero_y > e.y1 && hero_y < e.y2 {
                    break;
                }
                extn = extn.add(1);
            }

            if xtype != (*extn).etype as u16 {
                xtype = (*extn).etype as u16;
                let mut force = false;
                if xtype == XT_PRINCESS && ob_list8[PRINCESS_OBJECT].ob_stat != 0 {
                    rescue();
                    flag = 0;
                    continue;
                } else if xtype >= XT_SPECIAL_MIN {
                    // birds, turtles, special
                    if xtype == XT_NECROMANCER || xtype == XT_TURTLE_EGGS {
                        if anim_list[FIRST_MONSTER as usize].race != (*extn).v3 || anix < 4 {
                            encounter_x = (((*extn).x1 as u32 + (*extn).x2 as u32) / 2) as u16;
                            encounter_y = (((*extn).y1 as u32 + (*extn).y2 as u32) / 2) as u16;
                            force = true;
                        }
                    }
                } else if xtype == XT_SPIDER_PIT {
                    encounter_type = SPIDER_ENCOUNTER;
                    load_actors();
                    prep(ENEMY_SLOT);
                    motor_off();
                    actors_loading = 0;
                } else if xtype >= XT_MONSTER_ZONE_MIN && flag == 1 {
                    encounter_x = hero_x;
                    encounter_y = hero_y;
                    force = true;
                }
                if force {
                    start_forced_encounter();
                }
            }
            if xtype < XT_CARRIER_MIN {
                active_carrier = 0;
            } else if xtype == XT_CARRIER
                && (active_carrier == 0 || (riding == 0 && actor_file != (*extn).v3 as i16))
            {
                load_carrier((*extn).v3 as i16);
            }
            return 0;
        }
    }
}

