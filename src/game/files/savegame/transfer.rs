//! Writing the game to a byte buffer, or reading it back, field by field.
//!
//! The same code does both: each `Transfer` method writes a value when saving and reads it back
//! when loading, so the order of the calls in `transfer_all` is the file format. Numbers are
//! stored big-endian (as the original's 68000 kept them).

use crate::game::data::object_tables::glbobs;
use crate::game::data::state::Actors;
use crate::game::data::state::Game;
use crate::game::data::state::Hero;
use crate::game::data::state::SAVED_STUFF_SLOTS;
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
fn transfer_variables(game: &mut Game, transfer: &mut Transfer) {
    transfer.u16(&mut game.world.map_x);
    transfer.u16(&mut game.world.map_y);
    transfer.u16(&mut game.hero.x);
    transfer.u16(&mut game.hero.y);
    transfer.u16(&mut game.hero.safe_x);
    transfer.u16(&mut game.hero.safe_y);
    transfer.u16(&mut game.hero.safe_r);
    transfer.u16(&mut game.world.tile_x);
    transfer.u16(&mut game.world.tile_y);
    transfer.u16(&mut game.ui.cheat_enabled);
    transfer.u16(&mut game.hero.riding);
    transfer.u16(&mut game.hero.flying);
    transfer.u16(&mut game.actors.carrier_slot);
    transfer.u16(&mut game.actors.turtle_nearness);
    transfer.u16(&mut game.actors.raft_nearness);
    transfer.u16(&mut game.hero.brave);
    transfer.u16(&mut game.hero.luck);
    transfer.u16(&mut game.hero.kindness);
    transfer.u16(&mut game.hero.wealth);
    transfer.u16(&mut game.hero.hunger);
    transfer.u16(&mut game.hero.fatigue);
    transfer.u16(&mut game.hero.brother);
    transfer.u16(&mut game.hero.princesses_rescued);
    transfer.u16(&mut game.hero.sector);
    transfer.u16(&mut game.hero.place);
    transfer.u16(&mut game.world.day_clock);
    transfer.u16(&mut game.world.light_level);
    transfer.u16(&mut game.actors.loaded_actor_file);
    transfer.u16(&mut game.actors.loaded_setfig_file);
    transfer.u16(&mut game.actors.active_carrier);
    transfer.u16(&mut game.world.extent_type);
    transfer.u16(&mut game.actors.leader);
    transfer.u16(&mut game.world.secret_timer);
    transfer.u16(&mut game.world.light_timer);
    transfer.u16(&mut game.world.freeze_timer);
    transfer.u16(&mut game.ui.current_menu);
    transfer.u16(&mut game.encounter.kind);
    transfer.u16(&mut game.save_padding[0]);
    transfer.u16(&mut game.save_padding[1]);
    transfer.u16(&mut game.save_padding[2]);
}

/// Saves or loads one actor.
fn transfer_actor(transfer: &mut Transfer, a: &mut Shape) {
    transfer.u16(&mut a.abs_x);
    transfer.u16(&mut a.abs_y);
    transfer.u16(&mut a.rel_x);
    transfer.u16(&mut a.rel_y);
    transfer.u8(&mut a.kind);
    transfer.u8(&mut a.race);
    transfer.u8(&mut a.index);
    transfer.u8(&mut a.visible);
    transfer.u8(&mut a.weapon);
    transfer.u8(&mut a.environ);
    transfer.u8(&mut a.goal);
    transfer.u8(&mut a.tactic);
    transfer.u8(&mut a.state);
    transfer.u8(&mut a.facing);
    transfer.u16(&mut a.vitality);
    transfer.u8(&mut a.vel_x);
    transfer.u8(&mut a.vel_y);
}

/// Saves or loads one extent (a special rectangle of the world).
fn transfer_extent(transfer: &mut Transfer, e: &mut Extent) {
    transfer.u16(&mut e.x1);
    transfer.u16(&mut e.y1);
    transfer.u16(&mut e.x2);
    transfer.u16(&mut e.y2);
    transfer.u8(&mut e.etype);
    transfer.u8(&mut e.v1);
    transfer.u8(&mut e.v2);
    transfer.u8(&mut e.v3);
}

/// Saves or loads one object (an item or person in the world).
fn transfer_object(transfer: &mut Transfer, o: &mut WorldObject) {
    transfer.u16(&mut o.xc);
    transfer.u16(&mut o.yc);
    transfer.u8(&mut o.ob_id);
    transfer.u8(&mut o.ob_stat);
}

/// Saves or loads one missile (arrow, rock, fireball).
fn transfer_missile(transfer: &mut Transfer, m: &mut Missile) {
    transfer.u16(&mut m.abs_x);
    transfer.u16(&mut m.abs_y);
    transfer.u8(&mut m.missile_type);
    transfer.u8(&mut m.time_of_flight);
    transfer.u8(&mut m.speed);
    transfer.u8(&mut m.direction);
    transfer.u8(&mut m.archer);
}

/// Saves or loads the three brothers' item lists and the missiles (the count `next_missile` is saved
/// elsewhere).
fn transfer_party(actors: &mut Actors, hero: &mut Hero, transfer: &mut Transfer) {
    transfer.bytes(&mut hero.julstuff[..SAVED_STUFF_SLOTS]);
    transfer.bytes(&mut hero.philstuff[..SAVED_STUFF_SLOTS]);
    transfer.bytes(&mut hero.kevstuff[..SAVED_STUFF_SLOTS]);

    for missile in actors.missiles.iter_mut() {
        transfer_missile(transfer, missile);
    }
}

/// Writes (`Transfer::saving`) or reads the whole game state: the misc variables, the region, the
/// actors, the inventories and missiles, the extents and the object lists.
pub fn transfer_all(game: &mut Game, transfer: &mut Transfer) {
    transfer_variables(game, transfer);
    transfer.u16(&mut game.world.region);

    transfer.u16(&mut game.actors.active);
    transfer.u16(&mut game.actors.drawn);
    transfer.u16(&mut game.actors.next_missile);

    for position in 0..game.actors.active as usize {
        transfer_actor(transfer, &mut game.actors.list[position]);
    }

    transfer_party(&mut game.actors, &mut game.hero, transfer);

    for extent in game.extents.iter_mut().take(SAVED_EXTENTS) {
        transfer_extent(transfer, extent);
    }

    for object in game.objects.global.iter_mut().take(glbobs as usize) {
        transfer_object(transfer, object);
    }

    for count in game.objects.used.iter_mut().take(OBJECT_REGIONS) {
        transfer.u16(count);
    }

    for count in game.objects.distributed.iter_mut().take(OBJECT_REGIONS) {
        transfer.u16(count);
    }

    for position in 0..OBJECT_REGIONS {
        let count = game.objects.used[position];

        for object in game
            .objects
            .region(position)
            .iter_mut()
            .take(count as usize)
        {
            transfer_object(transfer, object);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_stored_big_endian() {
        let mut transfer = Transfer::saving();
        let (mut word, mut byte, mut signed) = (0x1234u16, 0x56u8, -2i16);
        transfer.u16(&mut word);
        transfer.u8(&mut byte);
        transfer.u16(&mut signed);

        assert_eq!(transfer.finish(), vec![0x12, 0x34, 0x56, 0xff, 0xfe]);
    }

    #[test]
    fn reading_gives_back_what_was_written() {
        let mut transfer = Transfer::loading(vec![0x12, 0x34, 0x56, 0xff, 0xfe]);
        let (mut word, mut byte, mut signed) = (0u16, 0u8, 0i16);
        transfer.u16(&mut word);
        transfer.u8(&mut byte);
        transfer.u16(&mut signed);

        assert_eq!((word, byte, signed), (0x1234, 0x56, -2));
    }

    #[test]
    fn a_file_that_ends_early_leaves_the_rest_alone() {
        let mut transfer = Transfer::loading(vec![0x12]);
        let mut word = 0x5678u16;
        transfer.u16(&mut word);

        assert_eq!(word, 0x1278);
    }
}
