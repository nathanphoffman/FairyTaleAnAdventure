//! Loading the music scores and sound effects from disk, and playing a sound effect.

use super::{Close, Open, Read, playsample};
use crate::amiga::BPTR;
use crate::game::data::state::the_memory;
use crate::game::data::tables::menus;
use crate::game::files::disk::{load_track_range, mark_read_unused, wait_for_read};

const MODE_OLDFILE: i32 = 1005;

// Songs: up to 28 tracks, packed into the score memory.
const TRACK_COUNT: usize = 4 * 7;
const SCORE_MEMORY: i32 = 5900;

/// Reads the song tracks from the file `songs` into `scoremem`.
///
/// The file is a list of tracks, each a 4 byte big-endian length (counted in 16-bit words) and
/// then that many words. There are up to 28 tracks (4 voices times 7 songs). Reading stops early
/// if they would not fit in the 5900 bytes of score memory.
pub fn read_score() {
    unsafe {
        let file = Open(c"songs".as_ptr(), MODE_OLDFILE) as i32;

        if file == 0 {
            return;
        }

        let (mut loaded, mut tracks) = (0, 0);

        for _ in 0..TRACK_COUNT {
            let mut length = 0i32;
            Read(file as BPTR, (&raw mut length).cast(), LENGTH_BYTES);
            length = length.swap_bytes(); // the length is stored big-endian

            if length * 2 + loaded > SCORE_MEMORY {
                break;
            }

            the_memory.track[tracks] = the_memory.scoremem.offset(loaded as isize);
            tracks += 1;
            Read(
                file as BPTR,
                the_memory.scoremem.offset(loaded as isize),
                length * 2,
            );

            loaded += length * 2;
        }

        Close(file as BPTR);
    }
}

/// A length in the score and sample files is a 4 byte big-endian number.
const LENGTH_BYTES: i32 = 4;

/// Where the sound effects are on the disk (block) and how many blocks they take.
const SAMPLE_BLOCK: i16 = 920;
const SAMPLE_BLOCKS: i16 = 11;

/// The disk request number used for sound samples.
const SAMPLE_REQUEST: i16 = 8;

/// There are 6 sound effects.
const SAMPLE_COUNT: usize = 6;

/// The game menu entry that switches sound effects on and off.
const GAME_MENU: usize = 4;
const SOUND_ENTRY: usize = 7;

/// Reads the sound effects from the disk into `sample_mem`. Each one is a 4 byte big-endian
/// length followed by that many bytes of sound; `sample` and `sample_size` point at them.
pub fn read_sample() {
    unsafe {
        load_track_range(
            SAMPLE_BLOCK,
            SAMPLE_BLOCKS,
            the_memory.sample_mem.cast(),
            SAMPLE_REQUEST,
        );

        wait_for_read(SAMPLE_REQUEST as i32);
        mark_read_unused(SAMPLE_REQUEST as i32);

        let mut at = the_memory.sample_mem;

        for i in 0..SAMPLE_COUNT {
            let length = u32::from_be_bytes([*at, *at.add(1), *at.add(2), *at.add(3)]);
            at = at.add(LENGTH_BYTES as usize);
            the_memory.sample[i] = at;
            the_memory.sample_size[i] = length;
            at = at.add(length as usize);
        }
    }
}

/// Plays sound effect `num` at the pitch `speed`, unless sound effects are switched off.
pub fn play_effect(num: i16, speed: i32) {
    unsafe {
        // Masks with GAME_MENU, keeping only the bits that matter here.
        if menus[GAME_MENU].enabled[SOUND_ENTRY] & 1 != 0 {
            let n = num as usize;
            playsample(
                the_memory.sample[n],
                (the_memory.sample_size[n] / 2) as i32,
                speed,
            );
        }
    }
}
