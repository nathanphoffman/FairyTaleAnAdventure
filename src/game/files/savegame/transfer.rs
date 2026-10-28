//! Writing the game to a byte buffer, or reading it back, field by field.
//!
//! The same code does both: each `Transfer` method writes a value when saving and reads it back
//! when loading, so the order of the calls in `transfer_all` is the file format. Numbers are
//! stored big-endian (as the Amiga's 68000 kept them).

use crate::game::data::globals::{
    princesses_rescued, save_padding_1, save_padding_2, save_padding_3,
};
use crate::game::data::object_tables::{dstobs, glbobs, mapobs, ob_listg, region_objects};
use crate::game::data::state::{
    SAVED_STUFF_SLOTS, player, the_actors, the_encounter, the_hero, the_ui, the_world,
};
use crate::game::data::tables::extent_list;
use crate::game::data::types::{Extent, Missile, Shape, WorldObject};

/// Entries of `extent_list` that are saved, and entries in the per-region object tables.
const SAVED_EXTENTS: usize = 2;
const OBJECT_REGIONS: usize = 10;

/// A 16 bit number, signed or not.
pub trait Word: Copy {
    fn to_bits(self) -> u16;
    fn from_bits(bits: u16) -> Self;
}

impl Word for u16 {
    fn to_bits(self) -> u16 {
        self
    }

    fn from_bits(bits: u16) -> u16 {
        bits
    }
}

impl Word for i16 {
    fn to_bits(self) -> u16 {
        self as u16
    }

    fn from_bits(bits: u16) -> i16 {
        bits as i16
    }
}

/// A one byte number, signed or not.
pub trait Byte: Copy {
    fn to_byte(self) -> u8;
    fn from_byte(byte: u8) -> Self;
}

impl Byte for u8 {
    fn to_byte(self) -> u8 {
        self
    }

    fn from_byte(byte: u8) -> u8 {
        byte
    }
}

impl Byte for i8 {
    fn to_byte(self) -> u8 {
        self as u8
    }

    fn from_byte(byte: u8) -> i8 {
        byte as i8
    }
}

/// The bytes of a save file being written or read.
pub struct Transfer {
    saving: bool,
    data: Vec<u8>,
    pos: usize,
}

impl Transfer {
    /// Starts a new save file.
    pub fn saving() -> Transfer {
        Transfer {
            saving: true,
            data: Vec::new(),
            pos: 0,
        }
    }

    /// Reads from the bytes of a save file.
    pub fn loading(data: Vec<u8>) -> Transfer {
        Transfer {
            saving: false,
            data,
            pos: 0,
        }
    }

    /// The bytes written so far.
    pub fn finish(self) -> Vec<u8> {
        self.data
    }

    /// Writes the bytes, or fills them from the file. A file that ends early leaves the rest as it was.
    pub fn bytes(&mut self, buffer: &mut [u8]) {
        if self.saving {
            self.data.extend_from_slice(buffer);

            return;
        }

        let end = (self.pos + buffer.len()).min(self.data.len());
        let available = &self.data[self.pos.min(end)..end];
        buffer[..available.len()].copy_from_slice(available);
        self.pos += buffer.len();
    }

    pub fn u8<T: Byte>(&mut self, value: &mut T) {
        let mut byte = [value.to_byte()];
        self.bytes(&mut byte);
        *value = T::from_byte(byte[0]);
    }

    pub fn u16<T: Word>(&mut self, value: &mut T) {
        let mut bytes = value.to_bits().to_be_bytes();
        self.bytes(&mut bytes);
        *value = T::from_bits(u16::from_be_bytes(bytes));
    }
}

/// The 40 variables the original saved as one 80 byte block, in that order: the map and hero
/// positions, the stats (bravery, luck, kindness, wealth, hunger, fatigue), which brother is
/// playing, the day/night clock, the magic timers and so on. The order is the file format.
fn transfer_variables(t: &mut Transfer) {
    unsafe {
        t.u16(&mut *(&raw mut the_world.map_x));
        t.u16(&mut *(&raw mut the_world.map_y));
        t.u16(&mut *(&raw mut the_hero.x));
        t.u16(&mut *(&raw mut the_hero.y));
        t.u16(&mut *(&raw mut the_hero.safe_x));
        t.u16(&mut *(&raw mut the_hero.safe_y));
        t.u16(&mut *(&raw mut the_hero.safe_r));
        t.u16(&mut *(&raw mut the_world.tile_x));
        t.u16(&mut *(&raw mut the_world.tile_y));
        t.u16(&mut *(&raw mut the_ui.cheat_enabled));
        t.u16(&mut *(&raw mut the_hero.riding));
        t.u16(&mut *(&raw mut the_hero.flying));
        t.u16(&mut *(&raw mut the_actors.carrier_slot));
        t.u16(&mut *(&raw mut the_actors.turtle_nearness));
        t.u16(&mut *(&raw mut the_actors.raft_nearness));
        t.u16(&mut *(&raw mut the_hero.brave));
        t.u16(&mut *(&raw mut the_hero.luck));
        t.u16(&mut *(&raw mut the_hero.kindness));
        t.u16(&mut *(&raw mut the_hero.wealth));
        t.u16(&mut *(&raw mut the_hero.hunger));
        t.u16(&mut *(&raw mut the_hero.fatigue));
        t.u16(&mut *(&raw mut the_hero.brother));
        t.u16(&mut *(&raw mut princesses_rescued));
        t.u16(&mut *(&raw mut the_hero.sector));
        t.u16(&mut *(&raw mut the_hero.place));
        t.u16(&mut *(&raw mut the_world.day_clock));
        t.u16(&mut *(&raw mut the_world.light_level));
        t.u16(&mut *(&raw mut the_actors.loaded_actor_file));
        t.u16(&mut *(&raw mut the_actors.loaded_setfig_file));
        t.u16(&mut *(&raw mut the_actors.active_carrier));
        t.u16(&mut *(&raw mut the_world.extent_type));
        t.u16(&mut *(&raw mut the_actors.leader));
        t.u16(&mut *(&raw mut the_world.secret_timer));
        t.u16(&mut *(&raw mut the_world.light_timer));
        t.u16(&mut *(&raw mut the_world.freeze_timer));
        t.u16(&mut *(&raw mut the_ui.current_menu));
        t.u16(&mut *(&raw mut the_encounter.kind));
        t.u16(&mut *(&raw mut save_padding_1));
        t.u16(&mut *(&raw mut save_padding_2));
        t.u16(&mut *(&raw mut save_padding_3));
    }
}

/// Saves or loads one actor.
fn transfer_actor(t: &mut Transfer, a: &mut Shape) {
    t.u16(&mut a.abs_x);
    t.u16(&mut a.abs_y);
    t.u16(&mut a.rel_x);
    t.u16(&mut a.rel_y);
    t.u8(&mut a.kind);
    t.u8(&mut a.race);
    t.u8(&mut a.index);
    t.u8(&mut a.visible);
    t.u8(&mut a.weapon);
    t.u8(&mut a.environ);
    t.u8(&mut a.goal);
    t.u8(&mut a.tactic);
    t.u8(&mut a.state);
    t.u8(&mut a.facing);
    t.u16(&mut a.vitality);
    t.u8(&mut a.vel_x);
    t.u8(&mut a.vel_y);
}

/// Saves or loads one extent (a special rectangle of the world).
fn transfer_extent(t: &mut Transfer, e: &mut Extent) {
    t.u16(&mut e.x1);
    t.u16(&mut e.y1);
    t.u16(&mut e.x2);
    t.u16(&mut e.y2);
    t.u8(&mut e.etype);
    t.u8(&mut e.v1);
    t.u8(&mut e.v2);
    t.u8(&mut e.v3);
}

/// Saves or loads one object (an item or person in the world).
fn transfer_object(t: &mut Transfer, o: &mut WorldObject) {
    t.u16(&mut o.xc);
    t.u16(&mut o.yc);
    t.u8(&mut o.ob_id);
    t.u8(&mut o.ob_stat);
}

/// Saves or loads one missile (arrow, rock, fireball).
fn transfer_missile(t: &mut Transfer, m: &mut Missile) {
    t.u16(&mut m.abs_x);
    t.u16(&mut m.abs_y);
    t.u8(&mut m.missile_type);
    t.u8(&mut m.time_of_flight);
    t.u8(&mut m.speed);
    t.u8(&mut m.direction);
    t.u8(&mut m.archer);
}

/// Saves or loads the three brothers' item lists and the missiles (the count `next_missile` is saved
/// elsewhere).
fn transfer_party(t: &mut Transfer) {
    unsafe {
        t.bytes(&mut player().julstuff[..SAVED_STUFF_SLOTS]);
        t.bytes(&mut player().philstuff[..SAVED_STUFF_SLOTS]);
        t.bytes(&mut player().kevstuff[..SAVED_STUFF_SLOTS]);

        for missile in (*(&raw mut the_actors.missiles)).iter_mut() {
            transfer_missile(t, missile);
        }
    }
}

/// Writes (`Transfer::saving`) or reads the whole game state: the misc variables, the region, the
/// actors, the inventories and missiles, the extents and the object lists.
pub fn transfer_all(t: &mut Transfer) {
    unsafe {
        transfer_variables(t);
        t.u16(&mut *(&raw mut the_world.region));

        t.u16(&mut *(&raw mut the_actors.active));
        t.u16(&mut *(&raw mut the_actors.drawn));
        t.u16(&mut *(&raw mut the_actors.next_missile));

        for i in 0..the_actors.active as usize {
            transfer_actor(t, &mut *(&raw mut the_actors.list[i]));
        }

        transfer_party(t);

        for extent in (*(&raw mut extent_list)).iter_mut().take(SAVED_EXTENTS) {
            transfer_extent(t, extent);
        }

        for object in (*(&raw mut ob_listg)).iter_mut().take(glbobs as usize) {
            transfer_object(t, object);
        }

        for count in (*(&raw mut mapobs)).iter_mut().take(OBJECT_REGIONS) {
            t.u16(count);
        }

        for count in (*(&raw mut dstobs)).iter_mut().take(OBJECT_REGIONS) {
            t.u16(count);
        }

        for (i, &count) in (*(&raw const mapobs))
            .iter()
            .enumerate()
            .take(OBJECT_REGIONS)
        {
            for object in region_objects(i).iter_mut().take(count as usize) {
                transfer_object(t, object);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_stored_big_endian() {
        let mut t = Transfer::saving();
        let (mut word, mut byte, mut signed) = (0x1234u16, 0x56u8, -2i16);
        t.u16(&mut word);
        t.u8(&mut byte);
        t.u16(&mut signed);

        assert_eq!(t.finish(), vec![0x12, 0x34, 0x56, 0xff, 0xfe]);
    }

    #[test]
    fn reading_gives_back_what_was_written() {
        let mut t = Transfer::loading(vec![0x12, 0x34, 0x56, 0xff, 0xfe]);
        let (mut word, mut byte, mut signed) = (0u16, 0u8, 0i16);
        t.u16(&mut word);
        t.u8(&mut byte);
        t.u16(&mut signed);

        assert_eq!((word, byte, signed), (0x1234, 0x56, -2));
    }

    #[test]
    fn a_file_that_ends_early_leaves_the_rest_alone() {
        let mut t = Transfer::loading(vec![0x12]);
        let mut word = 0x5678u16;
        t.u16(&mut word);

        assert_eq!(word, 0x1278);
    }
}
