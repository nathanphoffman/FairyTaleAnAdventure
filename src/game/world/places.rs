//! Moving the hero somewhere new, and working out where the hero is (from csrc/fmain.c).
//!
//! `find_place` looks up the sector the hero stands on in a table to get the place name message
//! ("You are in a village" ...), and looks the position up in `extent_list` to learn what kind of
//! area it is (`extent_type`): safe, a monster zone, a boss, a bird or turtle area, and so on.
//! Stepping into some of them starts something: monsters appear, or a carrier is loaded.

use crate::game::actors::encounters::load_enemy_sprites;
use crate::game::actors::encounters::{load_carrier, place_encounter};
use crate::game::actors::interaction::{blocked_at, rescue_princess};
use crate::game::data::object_tables::ob_list8;
use crate::game::data::state::{
    the_actors, the_encounter, the_hero, the_screen, the_ui, the_world,
};

use crate::game::data::tables::extent_list;
use crate::game::data::types::Extent;
use crate::game::files::disk::motor_off;
use crate::game::files::loading::finish_sprite_load;
use crate::game::ui::music::set_mood;
use crate::game::ui::text::show_message;
use crate::game::world::regions::{
    REGION_HEIGHT_SHIFT, REGION_WIDTH_SHIFT, center_sector, load_region_now, refresh_minimap,
};
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

/// Moving the hero picks the region from the centre sector: the row there is 3 bits (8 rows).
const TRANSFER_ROW_MASK: u32 = 7;

/// `view_status` that makes the whole screen be rebuilt.
const VIEW_REBUILD_ALL: i8 = 99;

/// The place tables have a row of three bytes (first sector, last sector, message) per entry.
const PLACE_TABLE_ROWS: usize = 256;
const PLACE_ROW_BYTES: usize = 3;
const SECTOR_BYTE_MASK: i16 = 255;

/// Regions up to this number are in the north half of the map.
const LAST_NORTHERN_REGION: u16 = 3;

/// A sector is 256 pixels (a shift of 8); a sector column with bit 6 set, or a row with bit 5 set,
/// is outside the loaded region.
const SECTOR_SHIFT: u32 = 8;
const OUTSIDE_COLUMN_BIT: u16 = 64;
const OUTSIDE_ROW_BIT: u16 = 32;

/// The necromancer and the turtle eggs appear again when fewer than this many actors are active.
const MIN_BOSS_ACTORS: i16 = 4;

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
pub extern "C" fn transfer_hero(xtest: u16, ytest: u16, flag: u16) -> i32 {
    unsafe {
        the_world.map_x = the_world.map_x.wrapping_add(xtest.wrapping_sub(the_hero.x));
        the_world.map_y = the_world.map_y.wrapping_add(ytest.wrapping_sub(the_hero.y));
        the_hero.x = xtest;
        the_actors.list[0].abs_x = xtest;
        the_hero.y = ytest;
        the_actors.list[0].abs_y = ytest;

        the_encounter.number = 0;

        if flag != 0 {
            let (sector_x, sector_y) = center_sector();
            // Shifts by REGION_WIDTH_SHIFT, converting between pixels, tiles, sectors or bytes.
            let column = (sector_x >> REGION_WIDTH_SHIFT) & 1;
            // Shifts by REGION_HEIGHT_SHIFT, converting between pixels, tiles, sectors or bytes.
            let row = (sector_y >> REGION_HEIGHT_SHIFT) & TRANSFER_ROW_MASK;
            the_world.new_region = (column + row + row) as u16;
        }

        the_ui.held_direction = 0;
        load_region_now();
        refresh_minimap(); // to set region_origin_x and region_origin_y
        the_screen.view_status = VIEW_REBUILD_ALL;
        set_mood(1);

        while blocked_at(the_hero.x as i16, the_hero.y as i16, 0) != 0 {
            the_hero.y = the_hero.y.wrapping_add(1);
        }

        0
    }
}

/// Starts the monsters of the extent the hero is in: loads their pictures and places up to four
/// of them (there must be room before the carrier slots).
unsafe fn start_forced_encounter() {
    unsafe {
        the_encounter.kind = (*the_world.extent).v3 as u16;
        the_encounter.mix_flag = 0;
        the_encounter.weapon_type = 0;
        load_enemy_sprites();
        finish_sprite_load(ENEMY_SLOT);
        motor_off();
        the_actors.loading = 0;
        the_encounter.number = (*the_world.extent).v1;
        the_actors.active = FIRST_MONSTER;

        while the_encounter.number != 0 && the_actors.active < MONSTER_SLOTS_END {
            if place_encounter(the_actors.active as u16, ENCOUNTER_SPREAD) != 0 {
                the_actors.active += 1;
            }

            the_encounter.number -= 1;
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
        the_hero.sector &= SECTOR_BYTE_MASK;
        let sector = the_hero.sector as u8;
        let table = if the_world.region >= FIRST_INDOOR_REGION {
            the_hero.sector += INSIDE_SECTOR_OFFSET;

            inside_tbl.as_ptr()
        } else {
            place_tbl.as_ptr()
        };

        let message = mountain_name(find_table_message(table, sector));

        if hero_on_loaded_edge() { 0 } else { message }
    }
}

/// The message of the table row (three bytes: first sector, last sector, message) that holds
/// `sector`.
unsafe fn find_table_message(table: *const u8, sector: u8) -> u8 {
    unsafe {
        let mut row = table;

        for _ in 0..PLACE_TABLE_ROWS {
            if sector >= *row && sector <= *row.add(1) {
                break;
            }

            row = row.add(PLACE_ROW_BYTES);
        }

        *row.add(2)
    }
}

/// Mountains are named after the part of the map they are in (outdoors only).
unsafe fn mountain_name(message: u8) -> u8 {
    unsafe {
        if message != MOUNTAINS_MESSAGE || the_world.region >= FIRST_INDOOR_REGION {
            message
        // Masks off the low bits (a cheap remainder or a flag test).
        } else if the_world.region & 1 != 0 {
            NORTHERN_MOUNTAINS_MESSAGE
        } else if the_world.region > LAST_NORTHERN_REGION {
            SOUTHERN_MOUNTAINS_MESSAGE
        } else {
            message
        }
    }
}

/// True while the next region is still loading, or when the hero is outside the loaded one.
unsafe fn hero_on_loaded_edge() -> bool {
    unsafe {
        // The hero's sector column and row: 256 pixels to a sector.
        let hero_column = (the_hero.x >> SECTOR_SHIFT).wrapping_sub(the_world.origin_x);
        // Shifts by SECTOR_SHIFT, converting between pixels, tiles, sectors or bytes.
        let hero_row = (the_hero.y >> SECTOR_SHIFT).wrapping_sub(the_world.origin_y);

        the_world.new_region < NO_REGION
            // Masks with OUTSIDE_COLUMN_BIT, keeping only the bits that matter here.
            || hero_column & OUTSIDE_COLUMN_BIT != 0
            // Masks with OUTSIDE_ROW_BIT, keeping only the bits that matter here.
            || hero_row & OUTSIDE_ROW_BIT != 0
    }
}

/// Remembers the place the hero is in, and says its name when it changes (if `flag` is set).
unsafe fn announce_place(flag: i16) {
    unsafe {
        let message = place_message_number();

        if message != 0 && message as u16 != the_hero.place {
            the_hero.place = message as u16;

            if flag != 0 {
                let messages = if the_world.region >= FIRST_INDOOR_REGION {
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
        the_world.extent = (&raw mut extent_list).cast::<Extent>();

        for _ in 0..EXTENT_COUNT {
            let e = &*the_world.extent;

            if the_hero.x > e.x1 && the_hero.x < e.x2 && the_hero.y > e.y1 && the_hero.y < e.y2 {
                break;
            }

            the_world.extent = the_world.extent.add(1);
        }
    }
}

/// What happens when the hero walks into a different kind of extent: the princess is rescued, a
/// boss or the turtle eggs call up their monsters, the spider pit loads the spiders, and monster
/// zones may send a fight. Returns false if the princess was rescued (the caller starts over).
unsafe fn enter_extent(flag: i16) -> bool {
    unsafe {
        if the_world.extent_type == (*the_world.extent).etype as u16 {
            return true;
        }

        the_world.extent_type = (*the_world.extent).etype as u16;
        let mut force = false;

        if the_world.extent_type == XT_PRINCESS && ob_list8[PRINCESS_OBJECT].ob_stat != 0 {
            rescue_princess();

            return false;
        } else if the_world.extent_type >= XT_SPECIAL_MIN {
            // birds, turtles, special
            if (the_world.extent_type == XT_NECROMANCER || the_world.extent_type == XT_TURTLE_EGGS)
                && (the_actors.list[FIRST_MONSTER as usize].race != (*the_world.extent).v3
                    || the_actors.active < MIN_BOSS_ACTORS)
            {
                the_encounter.x =
                    (((*the_world.extent).x1 as u32 + (*the_world.extent).x2 as u32) / 2) as u16;
                the_encounter.y =
                    (((*the_world.extent).y1 as u32 + (*the_world.extent).y2 as u32) / 2) as u16;
                force = true;
            }
        } else if the_world.extent_type == XT_SPIDER_PIT {
            the_encounter.kind = SPIDER_ENCOUNTER;
            load_enemy_sprites();
            finish_sprite_load(ENEMY_SLOT);
            motor_off();
            the_actors.loading = 0;
        } else if the_world.extent_type >= XT_MONSTER_ZONE_MIN && flag == 1 {
            the_encounter.x = the_hero.x;
            the_encounter.y = the_hero.y;
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
        if the_world.extent_type < XT_CARRIER_MIN {
            the_actors.active_carrier = 0;
        } else if the_world.extent_type == XT_CARRIER
            && (the_actors.active_carrier == 0
                || (the_hero.riding == 0
                    && the_actors.loaded_actor_file != (*the_world.extent).v3 as i16))
        {
            load_carrier((*the_world.extent).v3 as i16);
        }
    }
}
