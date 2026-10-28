//! Loading the sprite sheets of the hero, enemies, carriers and objects, and the pending region.

use super::{CARRIER, CFileInfo, DRAGON, cfile};
use crate::game::data::state::Game;
use crate::game::data::state::Graphics;
use crate::game::data::state::Video;
use crate::game::data::types::SeqInfo;
use crate::game::files::disk::{
    is_last_read_pending, last_read_finished, load_track_range, mark_read_unused, motor_off,
    wait_for_read,
};
use crate::game::output::Output;

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
pub static SPRITE_FILES: [CFileInfo; 18] = [
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

// Indexes into `SPRITE_FILES` that are always loaded together at the start.
const OBJECTS_FILE: i32 = 3;
const RAFT_FILE: i32 = 4;

/// Loads the sprite sets the game starts with: objects, the current brother, the raft, the
/// current monsters and set figures, then the region.
pub fn load_start_sprites(game: &mut Game, out: &mut dyn Output) {
    out.forget_sprites();
    out.load_sprite_file(OBJECTS_FILE);
    out.finish_sprite_load(OBJECTS as i16);
    out.load_sprite_file(game.hero.brother as i32 - 1);
    out.finish_sprite_load(PHIL as i16);
    out.load_sprite_file(RAFT_FILE);
    out.finish_sprite_load(RAFT as i16);
    out.start_sprite_sheet(ENEMY);
    out.load_sprite_file(game.actors.loaded_actor_file as i32);
    out.finish_sprite_load(SPRITE_FILES[game.actors.loaded_actor_file as usize].seq_num as i16);
    out.load_sprite_file(game.actors.loaded_setfig_file as i32);
    out.finish_sprite_load(SETFIG as i16);
    game.world.new_region = game.world.region;
    load_region_now(game, out);
    motor_off();
}

/// The `seq_list` entry for a sprite slot (PHIL, OBJECTS, ENEMY ...).
pub(super) fn sequence(graphics: &mut Graphics, slot: u8) -> &mut SeqInfo {
    &mut graphics.seq_list[slot as usize]
}

/// Starts loading sprite file `num` into the next free shape memory, if it fits.
///
/// Each sequence is stored as 5 planes of `size` bytes followed by a one-plane mask (`size` bytes).
/// The check makes sure that all of it, with room to spare, fits in the scratch area.
pub fn load_sprite_file(video: &mut Video, num: i32) {
    let file = &SPRITE_FILES[num as usize];
    let next = video.graphics.nextshape;
    let slot = sequence(&mut video.graphics, file.seq_num);
    slot.bytes = file.height as i16 * file.width as i16 * 2;
    let size = slot.bytes as usize * file.count as usize;
    slot.location = next;
    slot.width = file.width as i16;
    slot.height = file.height as i16;
    slot.count = file.count as i16;

    if next + size * SIZE_WITH_MASK <= SHAPE_SIZE {
        let end = (next + file.numblocks as usize * BLOCK_BYTES).min(video.graphics.shape.len());
        let area = &mut video.graphics.shape[next..end];
        load_track_range(
            file.file_id as i16,
            file.numblocks as i16,
            area,
            SPRITE_REQUEST,
        );

        let mask = next + size * SIZE_WITHOUT_MASK;
        sequence(&mut video.graphics, file.seq_num).maskloc = mask;
        video.graphics.nextshape = mask + size;
    }
}

/// Waits for the sprite load to finish and builds the masks for slot `slot`.
pub fn finish_sprite_load(video: &mut Video, slot: i16) {
    wait_for_read(SPRITE_REQUEST as i32);
    mark_read_unused(SPRITE_REQUEST as i32);
    {
        let s = *sequence(&mut video.graphics, slot as u8);
        make_sprite_masks(
            &mut video.graphics,
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
pub fn load_pending_region(game: &mut Game, out: &mut dyn Output) {
    if is_last_read_pending() == 0 || last_read_finished() != 0 {
        load_region_piece(game, out);
    }
}

#[cfg(test)]
mod tests {
    use super::super::CFileInfo;
    use super::*;

    #[test]
    fn sprite_table_matches_the_c_struct() {
        assert_eq!(size_of::<CFileInfo>(), 8);
        assert_eq!(SPRITE_FILES.len(), 18);
        assert_eq!(SPRITE_FILES[16].file_id, 946); // witch
    }
}
