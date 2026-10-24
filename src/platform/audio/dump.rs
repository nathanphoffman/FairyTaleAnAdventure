//! Test helper: with FTA_AUDIO_DUMP=file.wav the mixed output is also kept and written by dump().

use super::tables::{OUTPUT_CHANNELS, SAMPLE_RATE};
use std::sync::{Mutex, OnceLock};

static DUMP: Mutex<Vec<i16>> = Mutex::new(Vec::new());

const DUMP_VAR: &str = "FTA_AUDIO_DUMP";
const BITS_PER_SAMPLE: u16 = 16;
const BYTES_PER_FRAME: u16 = OUTPUT_CHANNELS as u16 * BITS_PER_SAMPLE / 8;
const WAV_HEADER_AFTER_SIZE: u32 = 36;
const WAV_FMT_CHUNK_SIZE: u32 = 16;
const WAV_FORMAT_PCM: u16 = 1;
const FULL_SCALE: f32 = 32767.0;

pub(super) fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os(DUMP_VAR).is_some())
}

/// Keeps a copy of the samples that were just played.
pub(super) fn capture(out: &[f32]) {
    if let Ok(mut d) = DUMP.lock() {
        d.extend(out.iter().map(|x| (x * FULL_SCALE) as i16));
    }
}

/// The 44 byte header of a 16 bit stereo PCM WAV holding `data_len` bytes of samples.
fn wav_header(data_len: u32) -> Vec<u8> {
    let rate = SAMPLE_RATE as u32;
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(WAV_HEADER_AFTER_SIZE + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&WAV_FMT_CHUNK_SIZE.to_le_bytes());
    out.extend_from_slice(&WAV_FORMAT_PCM.to_le_bytes());
    out.extend_from_slice(&(OUTPUT_CHANNELS as u16).to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * BYTES_PER_FRAME as u32).to_le_bytes());
    out.extend_from_slice(&BYTES_PER_FRAME.to_le_bytes());
    out.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out
}

/// Writes the captured audio as a 16 bit stereo WAV.
pub fn dump() {
    let Some(path) = std::env::var_os(DUMP_VAR) else {
        return;
    };
    let samples = DUMP.lock().map(|d| d.clone()).unwrap_or_default();
    let mut out = wav_header((samples.len() * 2) as u32);
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    if let Err(e) = std::fs::write(&path, out) {
        eprintln!("Could not write audio dump: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_describes_the_data() {
        let h = wav_header(1000);
        assert_eq!(h.len(), 44);
        assert_eq!(&h[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(h[4..8].try_into().unwrap()), 1036);
        assert_eq!(u32::from_le_bytes(h[40..44].try_into().unwrap()), 1000);
    }
}
