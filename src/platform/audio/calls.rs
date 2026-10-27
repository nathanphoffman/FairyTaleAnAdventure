//! The calls the game makes (formerly gdriver.asm).

use super::tables::*;
use super::with_sound;

/// `ins` is the game's instrument table, `wav` 8 waveforms of 128 bytes, `vol` 10 volume
/// envelopes of 256 bytes. The game fills `wav` and `vol` from the file v6 after this call.
pub fn init_music(ins: *const u16, wav: *const u8, vol: *const u8) {
    with_sound(|s| {
        s.ins = ins;
        s.wav = wav;
        s.vol = vol;
        s.tempo = DEFAULT_TEMPO;
        s.nosound = true;
    });
}

pub fn wrap_music() {
    stopscore();
    with_sound(|s| {
        s.ins = std::ptr::null();
        s.wav = std::ptr::null();
        s.vol = std::ptr::null();
    });
}

/// Sets the tracks to switch to when the current ones loop.
pub fn setscore(t1: *const u8, t2: *const u8, t3: *const u8, t4: *const u8) {
    with_sound(|s| {
        for (v, t) in [t1, t2, t3, t4].into_iter().enumerate() {
            s.voices[v].trak_beg = t;
        }
    });
}

/// Starts playing four tracks from the beginning.
pub fn playscore(t1: *const u8, t2: *const u8, t3: *const u8, t4: *const u8) {
    with_sound(|s| {
        s.play([t1, t2, t3, t4]);
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

/// One-shot sound effect on voice 2: `len` words of signed 8 bit data at `period`.
pub fn playsample(data: *const i8, len: i32, period: i32) {
    if data.is_null() || len <= 0 {
        return;
    }

    let bytes: Vec<i8> = unsafe { std::slice::from_raw_parts(data, len as usize * 2) }.to_vec();
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
