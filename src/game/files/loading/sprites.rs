//! Loading the sprite sheets of the hero, enemies, carriers and objects, and the pending region.

use super::{CARRIER, CFileInfo, DRAGON, cfile};
use crate::game::data::state::{memory, the_actors, the_hero, the_memory, the_world};
use crate::game::data::types::SeqInfo;
use crate::game::files::disk::{
    is_last_read_pending, last_read_finished, load_track_range, mark_read_unused, motor_off,
    wait_for_read,
};

use crate::game::render::blit::make_sprite_masks;
use crate::game::world::regions::{load_region_now, load_region_piece};

// The sequence slots in `seq_list`.
const PHIL: u8 = 0;
const OBJECTS: u8 = 1;
const ENEMY: u8 = 2;
const RAFT: u8 = 3;
const SETFIG: u8 = 4;

/// Bytes in a disk block.
const BLOCK_BYTES: usize = 512;

/// Scratch memory for sprites: bytes available in `shape_mem`.
pub const SHAPE_SIZE: usize = 78000;

/// A sprite sequence is loaded with room for its mask and the shifted copies after it.
const SIZE_WITH_MASK: usize = 6;
const SIZE_WITHOUT_MASK: usize = 5;

/// Drive request used when loading sprite files.
const SPRITE_REQUEST: i16 = 8;

/// The sprite files on the disk image. Index 0-2 are the three brothers, 3 the objects, 4 the raft,
/// 5 the turtle, 6-9 and 12 monster sets, 10 the dragon, 11 the bird, 13-17 the people (wizard,
/// royal court, bartender, witch, ranger). `file_id` is the first disk block.
///
/// Entry 12 (snake and salamander) uses the same block as entry 0, which looks like a leftover
/// placeholder in the original; it is copied as it was.
pub static sprite_files: [CFileInfo; 18] = [
    cfile(1, 32, 67, 42, PHIL, 1376),     // julian
    cfile(1, 32, 67, 42, PHIL, 1418),     // phillip
    cfile(1, 32, 67, 42, PHIL, 1460),     // kevin
    cfile(1, 16, 116, 36, OBJECTS, 1312), // objects
    cfile(2, 32, 2, 3, RAFT, 1348),       // raft
    cfile(2, 32, 16, 20, CARRIER, 1351),  // turtle
    cfile(1, 32, 64, 40, ENEMY, 960),     // ogre file
    cfile(1, 32, 64, 40, ENEMY, 1080),    // ghost file
    cfile(1, 32, 64, 40, ENEMY, 1000),    // dknight file (spiders)
    cfile(1, 32, 64, 40, ENEMY, 1040),    // necromancer (farmer / loraii)
    cfile(3, 40, 5, 12, DRAGON, 1160),    // dragon
    cfile(4, 64, 8, 40, CARRIER, 1120),   // bird
    cfile(1, 32, 64, 40, ENEMY, 1376),    // snake and salamander
    cfile(1, 32, 8, 5, SETFIG, 936),      // wizard/priest
    cfile(1, 32, 8, 5, SETFIG, 931),      // royal set
    cfile(1, 32, 8, 5, SETFIG, 941),      // bartender
    cfile(1, 32, 8, 5, SETFIG, 946),      // witch
    cfile(1, 32, 8, 5, SETFIG, 951),      // ranger/begger
];

// Indexes into `sprite_files` that are always loaded together at the start.
const OBJECTS_FILE: i32 = 3;
const RAFT_FILE: i32 = 4;

/// Loads the sprite sets the game starts with: objects, the current brother, the raft, the
/// current monsters and set figures, then the region.
pub fn load_start_sprites() {
    unsafe {
        the_memory.nextshape = 0;
        load_sprite_file(OBJECTS_FILE);
        finish_sprite_load(OBJECTS as i16);
        load_sprite_file(the_hero.brother as i32 - 1);
        finish_sprite_load(PHIL as i16);
        load_sprite_file(RAFT_FILE);
        finish_sprite_load(RAFT as i16);
        sequence(ENEMY).location = the_memory.nextshape;
        load_sprite_file(the_actors.loaded_actor_file as i32);
        finish_sprite_load(sprite_files[the_actors.loaded_actor_file as usize].seq_num as i16);
        load_sprite_file(the_actors.loaded_setfig_file as i32);
        finish_sprite_load(SETFIG as i16);
        the_world.new_region = the_world.region;
        load_region_now();
        motor_off();
    }
}

/// The `seq_list` entry for a sprite slot (PHIL, OBJECTS, ENEMY ...).
pub(super) fn sequence(slot: u8) -> &'static mut SeqInfo {
    &mut memory().seq_list[slot as usize]
}

/// Starts loading sprite file `num` into the next free shape memory, if it fits.
///
/// Each sequence is stored as 5 planes of `size` bytes followed by a one-plane mask (`size` bytes).
/// The check makes sure that all of it, with room to spare, fits in the scratch area.
pub fn load_sprite_file(num: i32) {
    unsafe {
        let file = &sprite_files[num as usize];
        let slot = sequence(file.seq_num);
        slot.bytes = file.height as i16 * file.width as i16 * 2;
        let size = slot.bytes as usize * file.count as usize;
        slot.location = the_memory.nextshape;
        slot.width = file.width as i16;
        slot.height = file.height as i16;
        slot.count = file.count as i16;

        let next = the_memory.nextshape;

        if next + size * SIZE_WITH_MASK <= SHAPE_SIZE {
            let end = (next + file.numblocks as usize * BLOCK_BYTES).min(memory().shape.len());
            let area = &mut memory().shape[next..end];
            load_track_range(
                file.file_id as i16,
                file.numblocks as i16,
                area,
                SPRITE_REQUEST,
            );

            the_memory.nextshape = next + size * SIZE_WITHOUT_MASK;
            slot.maskloc = the_memory.nextshape;
            the_memory.nextshape += size;
        }
    }
}

/// Waits for the sprite load to finish and builds the masks for slot `slot`.
pub fn finish_sprite_load(slot: i16) {
    wait_for_read(SPRITE_REQUEST as i32);
    mark_read_unused(SPRITE_REQUEST as i32);
    {
        let s = sequence(slot as u8);
        make_sprite_masks(
            s.location,
            s.maskloc,
            s.width as i32,
            s.height as i32,
            s.count as i32,
        );
    }
}

/// Loads the next region once the previous disk read is done. On the Amiga the disk was slow, so
/// the game kept playing while a region loaded in the background and called this to check.
pub fn load_pending_region() {
    if is_last_read_pending() == 0 || last_read_finished() != 0 {
        load_region_piece();
    }
}

#[cfg(test)]
mod tests {
    use super::super::CFileInfo;
    use super::*;

    #[test]
    fn sprite_table_matches_the_c_struct() {
        assert_eq!(size_of::<CFileInfo>(), 8);
        assert_eq!(sprite_files.len(), 18);
        assert_eq!(sprite_files[16].file_id, 946); // witch
    }
}
