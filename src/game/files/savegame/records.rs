//! Reading or writing each part of the game state in the save file's fixed order.

use super::{IoErr, Read, SVERR, SVFILE, Write, is_saving};
use crate::amiga::BPTR;
use crate::game::data::globals::{
    princesses_rescued, save_padding_1, save_padding_2, save_padding_3,
};
use crate::game::data::state::{the_actors, the_encounter, the_hero, the_ui, the_world};
use crate::game::data::tables::brother_stats;
use crate::game::data::types::{Extent, Missile, Shape, WorldObject};
use std::ffi::c_void;

const MISC_VARIABLES: usize = 40;

/// Writes or reads `length` bytes of the save file, remembering an error.
pub fn transfer_bytes(buffer: *mut c_void, length: i32) {
    unsafe {
        let result = if is_saving != 0 {
            Write(SVFILE as BPTR, buffer, length)
        } else {
            Read(SVFILE as BPTR, buffer.cast(), length)
        };

        // the original kept the result in a 16 bit variable
        if (result as i16) < 0 {
            SVERR = IoErr();
        }
    }
}

/// Saves or loads one byte.
pub fn transfer_u8(v: *mut u8) {
    transfer_bytes(v.cast(), 1);
}

/// Saves or loads a 16-bit number, always stored big-endian (as the Amiga's 68000 kept it).
pub fn transfer_u16(v: *mut u16) {
    unsafe {
        let mut bytes = [0u8; 2];

        if is_saving != 0 {
            bytes = (*v).to_be_bytes();
            transfer_bytes(bytes.as_mut_ptr().cast(), 2);
        } else {
            transfer_bytes(bytes.as_mut_ptr().cast(), 2);
            *v = u16::from_be_bytes(bytes);
        }
    }
}

/// The 40 variables the original saved as one 80 byte block, in that order: the map and hero
/// positions, the stats (bravery, luck, kindness, wealth, hunger, fatigue), which brother is
/// playing, the day/night clock, the magic timers and so on. The order is the file format.
pub(super) fn save_variables() {
    let variables: [*mut u16; MISC_VARIABLES] = unsafe {
        [
            &raw mut the_world.map_x,
            &raw mut the_world.map_y,
            &raw mut the_hero.x,
            &raw mut the_hero.y,
            &raw mut the_hero.safe_x,
            &raw mut the_hero.safe_y,
            &raw mut the_hero.safe_r,
            &raw mut the_world.tile_x,
            &raw mut the_world.tile_y,
            (&raw mut the_ui.cheat_enabled).cast(),
            (&raw mut the_hero.riding).cast(),
            (&raw mut the_hero.flying).cast(),
            (&raw mut the_actors.carrier_slot).cast(),
            (&raw mut the_actors.turtle_nearness).cast(),
            (&raw mut the_actors.raft_nearness).cast(),
            (&raw mut the_hero.brave).cast(),
            (&raw mut the_hero.luck).cast(),
            (&raw mut the_hero.kindness).cast(),
            (&raw mut the_hero.wealth).cast(),
            (&raw mut the_hero.hunger).cast(),
            (&raw mut the_hero.fatigue).cast(),
            (&raw mut the_hero.brother).cast(),
            (&raw mut princesses_rescued).cast(),
            (&raw mut the_hero.sector).cast(),
            &raw mut the_hero.place,
            &raw mut the_world.day_clock,
            &raw mut the_world.light_level,
            (&raw mut the_actors.loaded_actor_file).cast(),
            (&raw mut the_actors.loaded_setfig_file).cast(),
            (&raw mut the_actors.active_carrier).cast(),
            &raw mut the_world.extent_type,
            (&raw mut the_actors.leader).cast(),
            (&raw mut the_world.secret_timer).cast(),
            (&raw mut the_world.light_timer).cast(),
            (&raw mut the_world.freeze_timer).cast(),
            (&raw mut the_ui.current_menu).cast(),
            (&raw mut the_encounter.kind).cast(),
            &raw mut save_padding_1,
            &raw mut save_padding_2,
            &raw mut save_padding_3,
        ]
    };

    for v in variables {
        transfer_u16(v);
    }
}

/// Saves or loads one actor.
pub fn transfer_actor(a: *mut Shape) {
    unsafe {
        let a = &mut *a;
        transfer_u16(&mut a.abs_x);
        transfer_u16(&mut a.abs_y);
        transfer_u16(&mut a.rel_x);
        transfer_u16(&mut a.rel_y);
        transfer_u8((&mut a.kind as *mut i8).cast());
        transfer_u8(&mut a.race);

        for byte in [
            &mut a.index,
            &mut a.visible,
            &mut a.weapon,
            &mut a.environ,
            &mut a.goal,
            &mut a.tactic,
            &mut a.state,
            &mut a.facing,
        ] {
            transfer_u8((byte as *mut i8).cast());
        }

        transfer_u16((&mut a.vitality as *mut i16).cast());
        transfer_u8((&mut a.vel_x as *mut i8).cast());
        transfer_u8((&mut a.vel_y as *mut i8).cast());
    }
}

/// Saves or loads one extent (a special rectangle of the world).
pub fn transfer_extent(e: *mut Extent) {
    unsafe {
        let e = &mut *e;
        transfer_u16(&mut e.x1);
        transfer_u16(&mut e.y1);
        transfer_u16(&mut e.x2);
        transfer_u16(&mut e.y2);
        transfer_u8(&mut e.etype);
        transfer_u8(&mut e.v1);
        transfer_u8(&mut e.v2);
        transfer_u8(&mut e.v3);
    }
}

/// Saves or loads one object (an item or person in the world).
pub fn transfer_object(o: *mut WorldObject) {
    unsafe {
        let o = &mut *o;
        transfer_u16(&mut o.xc);
        transfer_u16(&mut o.yc);
        transfer_u8((&mut o.ob_id as *mut i8).cast());
        transfer_u8((&mut o.ob_stat as *mut i8).cast());
    }
}

/// Saves or loads one missile (arrow, rock, fireball). Called from `transfer_brothers_and_missiles` in csrc/fmain.c.
pub fn transfer_missile(m: *mut Missile) {
    unsafe {
        let m = &mut *m;
        transfer_u16(&mut m.abs_x);
        transfer_u16(&mut m.abs_y);

        for byte in [
            &mut m.missile_type,
            &mut m.time_of_flight,
            &mut m.speed,
            &mut m.direction,
            &mut m.archer,
        ] {
            transfer_u8((byte as *mut i8).cast());
        }
    }
}

/// Size of one brother's item list.
const STUFF_SIZE: i32 = 35;

/// Saves or loads the three brothers' item lists and the 6 missiles (the count `next_missile` is saved
/// elsewhere). After loading, `stuff` points at the current brother's list.
pub fn transfer_brothers_and_missiles() {
    unsafe {
        transfer_bytes((&raw mut the_hero.julstuff).cast(), STUFF_SIZE);
        transfer_bytes((&raw mut the_hero.philstuff).cast(), STUFF_SIZE);
        transfer_bytes((&raw mut the_hero.kevstuff).cast(), STUFF_SIZE);
        the_hero.stuff = brother_stats[the_hero.brother as usize - 1].stuff;

        for i in 0..6 {
            transfer_missile((&raw mut the_actors.missiles).cast::<Missile>().add(i));
        }
    }
}
