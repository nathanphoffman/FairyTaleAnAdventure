//! Loading a sprite file into the sprite memory and building its masks.

use crate::game::data::state::Graphics;
use crate::game::data::state::Video;
use crate::game::data::types::SeqInfo;
use crate::game::files::disk::{load_track_range, mark_read_unused, wait_for_read};
use crate::game::files::loading::SPRITE_FILES;
use crate::game::render::blit::make_sprite_masks;

/// Bytes in a disk block.
const BLOCK_BYTES: usize = 512;

/// Scratch memory for sprites: bytes available in `shape_mem`.
const SHAPE_SIZE: usize = 78000;

/// A sprite sequence is loaded with room for its mask and the shifted copies after it.
const SIZE_WITH_MASK: usize = 6;
const SIZE_WITHOUT_MASK: usize = 5;

/// Drive request used when loading sprite files.
const SPRITE_REQUEST: i16 = 8;

/// The `seq_list` entry for a sprite slot (PHIL, OBJECTS, ENEMY ...).
fn sequence(graphics: &mut Graphics, slot: u8) -> &mut SeqInfo {
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
