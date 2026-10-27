//! The calls the game makes (formerly gdriver.asm).

use super::sequencer::Track;
use super::tables::*;
use super::with_sound;

/// Gives the music driver its data: `instruments` is the table of instrument words (waveform
/// number in the high byte, volume envelope in the low one), `waves` 8 waveforms of 128 bytes and
/// `volumes` 10 volume envelopes of 256 bytes.
pub fn init_music(instruments: Vec<u16>, waves: Vec<u8>, volumes: Vec<u8>) {
    with_sound(|s| {
        s.ins = instruments;
        s.wav = waves;
        s.vol = volumes;
        s.tempo = DEFAULT_TEMPO;
        s.nosound = true;
    });
}

/// Changes one entry of the instrument table (the game does this to give caves and houses their
/// own sound).
pub fn set_instrument(slot: usize, word: u16) {
    with_sound(|s| {
        if let Some(entry) = s.ins.get_mut(slot) {
            *entry = word;
        }
    });
}

pub fn wrap_music() {
    stopscore();
    with_sound(|s| {
        s.ins.clear();
        s.wav.clear();
        s.vol.clear();
    });
}

/// Sets the tracks to switch to when the current ones loop.
pub fn setscore(tracks: [Track; VOICE_COUNT]) {
    with_sound(|s| {
        for (v, t) in tracks.into_iter().enumerate() {
            s.voices[v].trak_beg = Some(t);
        }
    });
}

/// Starts playing four tracks from the beginning.
pub fn playscore(tracks: [Track; VOICE_COUNT]) {
    with_sound(|s| {
        s.play(tracks);
    });
}

pub fn stopscore() {
    with_sound(|s| {
        for v in 0..VOICE_COUNT {
            s.chans[v].volume = 0;
            s.chans[v].active = false;
            s.voices[v].vol_delay = 0;
            s.voices[v].stat = 0;
        }

        s.nosound = true;
    });
}

/// One-shot sound effect on voice 2: signed 8 bit data (a whole number of 16 bit words) played at
/// `period`.
pub fn playsample(data: &[i8], period: i32) {
    let bytes = data[..data.len() & !1].to_vec();

    if bytes.is_empty() {
        return;
    }

    with_sound(|s| {
        let ch = &mut s.chans[EFFECT_VOICE];
        ch.data = bytes;
        ch.phase = 0.0;
        ch.volume = MAX_VOLUME;
        ch.looping = false;
        ch.active = true;
        s.voices[EFFECT_VOICE].stat = 1;
        s.voices[EFFECT_VOICE].vol_delay = -1;
        s.set_period(EFFECT_VOICE, period.max(1) as u32);
    });
}
