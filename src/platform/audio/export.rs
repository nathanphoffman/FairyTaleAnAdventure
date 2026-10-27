//! Renders the game's tunes to WAV files without playing them (set FTA_MUSIC_EXPORT=dir).

use super::dump::wav_header;
use super::sequencer::{Sound, Track};
use super::tables::{DEFAULT_TEMPO, OUTPUT_CHANNELS, SAMPLE_RATE, VOICE_COUNT};
use super::with_sound;
use std::path::Path;

/// Frames rendered per step of the loop.
const CHUNK_FRAMES: usize = 4096;

/// A tune that never finishes is cut off after this many seconds.
const MAX_SECONDS: usize = 600;

/// The last seconds fade out, because the tunes are written to loop.
const FADE_SECONDS: usize = 3;
const FULL_SCALE: f32 = 32767.0;

/// Writes `name.wav` in `dir` for every tune: its four tracks, played until all of them have
/// ended or looped once. `before` runs before each tune (the game changes the instrument for some).
pub fn export_tunes(
    dir: &Path,
    tunes: &[(&str, [Track; VOICE_COUNT])],
    mut before: impl FnMut(&str),
) {
    for (name, tracks) in tunes {
        before(name);
        let mut sound = Sound::new(SAMPLE_RATE as f64);
        with_sound(|s| {
            (sound.ins, sound.wav, sound.vol) = (s.ins.clone(), s.wav.clone(), s.vol.clone())
        });
        sound.tempo = DEFAULT_TEMPO;
        sound.play(tracks.clone());
        let samples = render(&mut sound);
        let path = dir.join(format!("{name}.wav"));
        let mut out = wav_header((samples.len() * 2) as u32);

        for s in &samples {
            out.extend_from_slice(&s.to_le_bytes());
        }

        match std::fs::write(&path, out) {
            Ok(()) => eprintln!("{}: {:.1} seconds", path.display(), seconds(samples.len())),
            Err(e) => eprintln!("Could not write {}: {e}", path.display()),
        }
    }
}

fn seconds(samples: usize) -> f64 {
    samples as f64 / (OUTPUT_CHANNELS as usize * SAMPLE_RATE as usize) as f64
}

/// Mixes until the tune has played through, then fades the end.
fn render(sound: &mut Sound) -> Vec<i16> {
    let per_second = OUTPUT_CHANNELS as usize * SAMPLE_RATE as usize;
    let mut all: Vec<f32> = Vec::new();
    let mut chunk = vec![0.0f32; CHUNK_FRAMES * OUTPUT_CHANNELS as usize];

    while !sound.played_through() && all.len() < MAX_SECONDS * per_second {
        sound.mix(&mut chunk);
        all.extend_from_slice(&chunk);
    }

    let fade = (FADE_SECONDS * per_second).min(all.len());
    let start = all.len() - fade;

    for (i, s) in all[start..].iter_mut().enumerate() {
        *s *= 1.0 - i as f32 / fade as f32;
    }

    all.iter().map(|x| (x * FULL_SCALE) as i16).collect()
}
