//! Loading the music scores and sound effects from disk, and playing a sound effect.

use crate::game::data::tables::menus;
use crate::game::files::disk::read_blocks;
use crate::platform::audio::{Track, playsample};
use crate::platform::dos::resolve;
use std::sync::Mutex;

/// The file with the song tracks.
const SONG_FILE: &str = "songs";

// Songs: up to 28 tracks, packed into the score memory.
const TRACK_COUNT: usize = 4 * 7;
const SCORE_MEMORY: usize = 5900;

/// A length in the score and sample files is a 4 byte big-endian number.
const LENGTH_BYTES: usize = 4;

/// Where the sound effects are on the disk (block) and how many blocks they take.
const SAMPLE_BLOCK: i16 = 920;
const SAMPLE_BLOCKS: i16 = 11;

/// There are 6 sound effects.
const SAMPLE_COUNT: usize = 6;

/// The game menu entry that switches sound effects on and off.
const GAME_MENU: usize = 4;
const SOUND_ENTRY: usize = 7;

/// The music tracks and sound effects that were read in.
struct Sounds {
    tracks: Vec<Track>,
    samples: Vec<Vec<i8>>,
}

static SOUNDS: Mutex<Sounds> = Mutex::new(Sounds {
    tracks: Vec::new(),
    samples: Vec::new(),
});

fn sounds() -> std::sync::MutexGuard<'static, Sounds> {
    SOUNDS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Music track number `n` (four to a tune). A track that was not read is empty, which is silence.
pub fn track(n: usize) -> Track {
    sounds()
        .tracks
        .get(n)
        .cloned()
        .unwrap_or_else(|| Track::from(Vec::new()))
}

/// The four tracks of tune `tune`.
pub fn tune_tracks(tune: usize) -> [Track; 4] {
    [0, 1, 2, 3].map(|voice| track(tune * 4 + voice))
}

/// The big-endian number at `at`, or 0 when the data ends first.
fn be32(data: &[u8], at: usize) -> usize {
    data.get(at..at + LENGTH_BYTES)
        .map_or(0, |b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
}

/// Cuts the tracks out of the song file's data.
///
/// The file is a list of tracks, each a 4 byte big-endian length (counted in 16-bit words) and
/// then that many words. There are up to 28 tracks (4 voices times 7 songs). Reading stops early
/// if they would not fit in the 5900 bytes of score memory.
fn split_tracks(data: &[u8]) -> Vec<Track> {
    let mut tracks = Vec::new();
    let (mut at, mut loaded) = (0, 0);

    for _ in 0..TRACK_COUNT {
        let bytes = be32(data, at) * 2;
        at += LENGTH_BYTES;

        if bytes + loaded > SCORE_MEMORY {
            break;
        }

        let end = (at + bytes).min(data.len());
        tracks.push(Track::from(data[at.min(end)..end].to_vec()));
        at += bytes;
        loaded += bytes;
    }

    tracks
}

/// Reads the song tracks from the file `songs`.
pub fn read_score() {
    let Some(data) = resolve(SONG_FILE, false).and_then(|path| std::fs::read(path).ok()) else {
        return;
    };

    sounds().tracks = split_tracks(&data);
}

/// Cuts the sound effects out of the data read from the disk: each one is a 4 byte big-endian length
/// followed by that many bytes of sound.
fn split_samples(data: &[u8]) -> Vec<Vec<i8>> {
    let mut samples = Vec::new();
    let mut at = 0;

    for _ in 0..SAMPLE_COUNT {
        let length = be32(data, at);
        at += LENGTH_BYTES;
        let end = (at + length).min(data.len());
        samples.push(data[at.min(end)..end].iter().map(|&b| b as i8).collect());
        at += length;
    }

    samples
}

/// Reads the sound effects from the disk.
pub fn read_sample() {
    sounds().samples = split_samples(&read_blocks(SAMPLE_BLOCK, SAMPLE_BLOCKS));
}

/// Plays sound effect `num` at the pitch `speed`, unless sound effects are switched off.
pub fn play_effect(num: i16, speed: i32) {
    // Masks with GAME_MENU, keeping only the bits that matter here.
    if unsafe { menus[GAME_MENU].enabled[SOUND_ENTRY] } & 1 != 0
        && let Some(sample) = sounds().samples.get(num as usize)
    {
        playsample(sample, speed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_are_cut_out_by_their_lengths() {
        let mut data = Vec::new();
        data.extend(2u32.to_be_bytes()); // 2 words
        data.extend([1, 2, 3, 4]);
        data.extend(1u32.to_be_bytes());
        data.extend([5, 6]);

        let tracks = split_tracks(&data);

        assert_eq!(&*tracks[0], &[1, 2, 3, 4]);
        assert_eq!(&*tracks[1], &[5, 6]);
    }

    #[test]
    fn tracks_stop_when_the_score_memory_is_full() {
        let mut data = Vec::new();
        data.extend(((SCORE_MEMORY / 2 + 1) as u32).to_be_bytes());

        assert!(split_tracks(&data).is_empty());
    }

    #[test]
    fn samples_are_cut_out_by_their_lengths() {
        let mut data = Vec::new();
        data.extend(3u32.to_be_bytes());
        data.extend([1, 255, 3]);

        let samples = split_samples(&data);

        assert_eq!(samples[0], vec![1, -1, 3]);
        assert_eq!(samples.len(), SAMPLE_COUNT);
    }
}
