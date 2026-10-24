//! Music and sound effects, replacing gdriver.asm and the Amiga's four-channel sound chip.
//!
//! The game plays music from four tracks of (command, value) events, one per voice, and one-shot
//! sound effects on voice 2. The original ran the sequencer from the vertical blank interrupt and
//! let the chip play the waveforms by DMA. Here an SDL audio callback does both: it advances the
//! sequencer 60 times a second and mixes the four channels (voices 1 and 4 left, 2 and 3 right,
//! as on the Amiga).
//!
//! Track format (see docs/data-formats.md): command byte < 128 is a note number and the value's
//! low 6 bits index `NOTE_LENGTHS`; 128 = rest, 129 = set instrument, 0x90 = set tempo, 255 = end
//! of track (value non-zero = loop).

use sdl2::AudioSubsystem;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::{Arc, Mutex, OnceLock};

/// (period, start offset in units of 4 bytes) for each note; high notes play a shorter copy of
/// the waveform stored later in its 128 bytes.
const NOTES: [(u16, u16); 78] = [
    (1440, 0),
    (1356, 0),
    (1280, 0),
    (1208, 0),
    (1140, 0),
    (1076, 0),
    (1016, 0),
    (960, 0),
    (906, 0),
    (856, 0),
    (808, 0),
    (762, 0),
    (720, 0),
    (678, 0),
    (640, 0),
    (604, 0),
    (570, 0),
    (538, 0),
    (508, 0),
    (480, 0),
    (453, 0),
    (428, 0),
    (404, 0),
    (381, 0),
    (360, 0),
    (339, 0),
    (320, 0),
    (302, 0),
    (285, 0),
    (269, 0),
    (508, 16),
    (480, 16),
    (453, 16),
    (428, 16),
    (404, 16),
    (381, 16),
    (360, 16),
    (339, 16),
    (320, 16),
    (302, 16),
    (285, 16),
    (269, 16),
    (508, 24),
    (480, 24),
    (453, 24),
    (428, 24),
    (404, 24),
    (381, 24),
    (360, 24),
    (339, 24),
    (320, 24),
    (302, 24),
    (285, 24),
    (269, 24),
    (508, 28),
    (480, 28),
    (453, 28),
    (428, 28),
    (404, 28),
    (381, 28),
    (360, 28),
    (339, 28),
    (320, 28),
    (302, 28),
    (285, 28),
    (269, 28),
    (254, 28),
    (240, 28),
    (226, 28),
    (214, 28),
    (202, 28),
    (190, 28),
    (180, 28),
    (170, 28),
    (160, 28),
    (151, 28),
    (143, 28),
    (135, 28),
];

/// Note durations in sequencer clock units, indexed by the low 6 bits of the value byte.
const NOTE_LENGTHS: [i32; 64] = [
    26880, 13440, 6720, 3360, 1680, 840, 420, 210, 40320, 20160, 10080, 5040, 2520, 1260, 630, 315,
    17920, 8960, 4480, 2240, 1120, 560, 280, 140, 26880, 13440, 6720, 3360, 1680, 840, 420, 210,
    21504, 10752, 5376, 2688, 1344, 672, 336, 168, 32256, 16128, 8064, 4032, 2016, 1008, 504, 252,
    23040, 11520, 5760, 2880, 1440, 720, 360, 180, 34560, 17280, 8640, 4320, 2160, 1080, 540, 270,
];

const PAULA_CLOCK: f64 = 3_546_895.0; // PAL; the period values are relative to this
const TICK_HZ: f64 = 60.0; // the sequencer ran from the vertical blank
const GAP: i32 = 300; // silence at the end of a note

#[derive(Clone, Copy)]
struct Voice {
    trak_ptr: *const u8,
    trak_beg: *const u8,
    wave_num: u8,
    vol_num: u8,
    vol_delay: i8,
    /// non-zero while a sound effect owns this voice's channel
    stat: u8,
    event_start: i32,
    event_stop: i32,
    vol_list: *const u8,
}

impl Voice {
    const fn new() -> Voice {
        Voice {
            trak_ptr: std::ptr::null(),
            trak_beg: std::ptr::null(),
            wave_num: 0,
            vol_num: 0,
            vol_delay: 0,
            stat: 0,
            event_start: 0,
            event_stop: 0,
            vol_list: std::ptr::null(),
        }
    }
}

/// One hardware channel: plays signed 8 bit data at a rate set by its period.
#[derive(Clone)]
struct Channel {
    data: Vec<i8>,
    phase: f64,
    step: f64,  // data bytes per output sample
    volume: u8, // 0-64
    active: bool,
    looping: bool,
}

impl Channel {
    fn new() -> Channel {
        Channel {
            data: Vec::new(),
            phase: 0.0,
            step: 0.0,
            volume: 0,
            active: false,
            looping: true,
        }
    }
}

struct Sound {
    voices: [Voice; 4],
    chans: [Channel; 4],
    tempo: i32,
    timeclock: i32,
    nosound: bool,
    /// instrument table (the game's `new_wave`): high byte = waveform, low byte = envelope
    ins: *const u16,
    wav: *const u8,
    vol: *const u8,
    rate: f64,
    tick_acc: f64,
}

// The raw pointers refer to game memory that outlives the audio device.
unsafe impl Send for Sound {}

impl Sound {
    fn new(rate: f64) -> Sound {
        Sound {
            voices: [Voice::new(); 4],
            chans: [
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
            ],
            tempo: 150,
            timeclock: 0,
            nosound: true,
            ins: std::ptr::null(),
            wav: std::ptr::null(),
            vol: std::ptr::null(),
            rate,
            tick_acc: 0.0,
        }
    }

    fn set_period(&mut self, ch: usize, period: u32) {
        // hardware limit: periods below 124 are not honoured
        let period = period.max(124);
        self.chans[ch].step = PAULA_CLOCK / period as f64 / self.rate;
    }

    fn silence(&mut self, ch: usize) {
        self.chans[ch].volume = 0;
    }

    /// One vertical blank of the music driver (`vblank_server` in gdriver.asm).
    fn tick(&mut self) {
        self.timeclock = self.timeclock.wrapping_add(self.tempo);
        if self.nosound {
            return;
        }
        for v in 0..4 {
            self.do_voice(v);
        }
    }

    fn do_voice(&mut self, v: usize) {
        if self.voices[v].trak_ptr.is_null() {
            return;
        }
        let now = self.timeclock;
        let vc = self.voices[v];
        if now.wrapping_sub(vc.event_start) >= 0 {
            self.new_note(v);
            return;
        }
        if vc.stat != 0 {
            return; // a sound effect is playing on this channel
        }
        if now.wrapping_sub(vc.event_stop) >= 0 {
            self.chans[v].volume = 0; // gap before the next note
            return;
        }
        if vc.vol_delay != 0 || vc.vol_list.is_null() {
            return;
        }
        let level = unsafe { *vc.vol_list } as i8;
        if level >= 0 {
            self.chans[v].volume = level as u8;
            self.voices[v].vol_list = unsafe { vc.vol_list.add(1) };
        }
    }

    fn new_note(&mut self, v: usize) {
        loop {
            let (cmd, value) = unsafe {
                let p = self.voices[v].trak_ptr;
                self.voices[v].trak_ptr = p.add(2);
                (*p, *p.add(1))
            };
            if cmd & 0x80 == 0 || cmd == 128 {
                self.start_note(v, cmd, value);
                return;
            }
            match cmd {
                129 => {
                    // set instrument
                    let word = unsafe { *self.ins.add((value & 15) as usize) };
                    self.voices[v].wave_num = (word >> 8) as u8;
                    self.voices[v].vol_num = word as u8;
                }
                0x90 => self.tempo = value as i32,
                255 => {
                    if value != 0 && !self.voices[v].trak_beg.is_null() {
                        self.voices[v].trak_ptr = self.voices[v].trak_beg;
                    } else {
                        self.voices[v].trak_ptr = std::ptr::null();
                        self.voices[v].vol_delay = -1;
                        self.chans[v].volume = 0;
                        return;
                    }
                }
                _ => {} // unknown command, skip it
            }
        }
    }

    fn start_note(&mut self, v: usize, cmd: u8, value: u8) {
        let len = NOTE_LENGTHS[(value & 0x3f) as usize];
        let mut stop = len - GAP;
        if stop < 0 {
            stop = len;
        }
        let vc = &mut self.voices[v];
        vc.event_stop = vc.event_start.wrapping_add(stop);
        vc.event_start = vc.event_start.wrapping_add(len);
        if vc.stat != 0 {
            return; // sound effect has the channel
        }
        let env = unsafe { self.vol.add(vc.vol_num as usize * 256) };
        let start_vol = unsafe { *env };
        vc.vol_list = unsafe { env.add(1) };
        vc.vol_delay = 0;
        if cmd == 128 {
            self.chans[v].volume = 0; // rest
            vc.event_stop = self.timeclock;
            return;
        }
        let (period, offset) = NOTES.get(cmd as usize).copied().unwrap_or((508, 0));
        let wave = unsafe {
            self.wav
                .add(vc.wave_num as usize * 128 + offset as usize * 4)
        };
        let words = 32 - offset as usize;
        let ch = &mut self.chans[v];
        ch.data = (0..words * 2)
            .map(|i| unsafe { *wave.add(i) } as i8)
            .collect();
        ch.looping = true;
        ch.phase = 0.0;
        ch.volume = start_vol.min(64);
        ch.active = true;
        self.set_period(v, period as u32);
    }

    /// Mixes `out` (interleaved left, right).
    fn mix(&mut self, out: &mut [f32]) {
        let per_tick = self.rate / TICK_HZ;
        for frame in out.chunks_exact_mut(2) {
            self.tick_acc += 1.0;
            if self.tick_acc >= per_tick {
                self.tick_acc -= per_tick;
                self.tick();
            }
            let (mut left, mut right) = (0.0f32, 0.0f32);
            for c in 0..4 {
                let s = self.chans[c].next();
                if c == 0 || c == 3 {
                    left += s;
                } else {
                    right += s;
                }
            }
            frame[0] = (left * 0.45).clamp(-1.0, 1.0);
            frame[1] = (right * 0.45).clamp(-1.0, 1.0);
        }
        // a finished sound effect gives voice 2 back to the music
        if self.voices[1].stat != 0 && !self.chans[1].active {
            self.voices[1].stat = 0;
        }
    }
}

impl Channel {
    /// Next output sample, -1.0 to 1.0.
    fn next(&mut self) -> f32 {
        if !self.active || self.data.is_empty() {
            return 0.0;
        }
        let len = self.data.len();
        let i = self.phase as usize;
        let frac = (self.phase - i as f64) as f32;
        let a = self.data[i.min(len - 1)] as f32;
        let b = if i + 1 < len {
            self.data[i + 1] as f32
        } else if self.looping {
            self.data[0] as f32
        } else {
            a
        };
        let sample = (a + (b - a) * frac) / 128.0 * (self.volume as f32 / 64.0);
        self.phase += self.step;
        if self.phase >= len as f64 {
            if self.looping {
                self.phase -= len as f64;
            } else {
                self.active = false;
            }
        }
        sample
    }
}

/// Test helper: with FTA_AUDIO_DUMP=file.wav the mixed output is also kept and written by dump().
static DUMP: Mutex<Vec<i16>> = Mutex::new(Vec::new());

fn dump_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("FTA_AUDIO_DUMP").is_some())
}

/// Writes the captured audio as a 16 bit stereo WAV (44100 Hz).
pub fn dump() {
    let Some(path) = std::env::var_os("FTA_AUDIO_DUMP") else {
        return;
    };
    let samples = DUMP.lock().map(|d| d.clone()).unwrap_or_default();
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&2u16.to_le_bytes()); // stereo
    out.extend_from_slice(&44100u32.to_le_bytes());
    out.extend_from_slice(&(44100u32 * 4).to_le_bytes());
    out.extend_from_slice(&4u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    let _ = std::fs::write(path, out);
}

struct Callback(Arc<Mutex<Sound>>);

impl AudioCallback for Callback {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        match self.0.lock() {
            Ok(mut s) => s.mix(out),
            Err(_) => out.fill(0.0),
        }
        if dump_enabled()
            && let Ok(mut d) = DUMP.lock()
        {
            d.extend(out.iter().map(|x| (x * 32767.0) as i16));
        }
    }
}

static SOUND: OnceLock<Arc<Mutex<Sound>>> = OnceLock::new();

/// Opens the audio device. Without one the game runs silent.
pub fn init(audio: &AudioSubsystem) {
    let desired = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2),
        samples: Some(1024),
    };
    let mut shared = None;
    let device: Result<AudioDevice<Callback>, String> =
        audio.open_playback(None, &desired, |spec| {
            let sound = Arc::new(Mutex::new(Sound::new(spec.freq as f64)));
            shared = Some(sound.clone());
            Callback(sound)
        });
    match device {
        Ok(d) => {
            d.resume();
            std::mem::forget(d); // keep playing for the life of the program
            if let Some(s) = shared {
                let _ = SOUND.set(s);
            }
        }
        Err(e) => eprintln!("No audio: {e}"),
    }
}

fn with_sound(f: impl FnOnce(&mut Sound)) {
    if let Some(s) = SOUND.get()
        && let Ok(mut s) = s.lock()
    {
        f(&mut s);
    }
}

// ---- the calls the game makes (formerly gdriver.asm) ----

/// `ins` is the game's instrument table, `wav` 8 waveforms of 128 bytes, `vol` 10 volume
/// envelopes of 256 bytes. The game fills `wav` and `vol` from the file v6 after this call.
#[unsafe(no_mangle)]
pub extern "C" fn init_music(ins: *const u16, wav: *const u8, vol: *const u8) {
    with_sound(|s| {
        s.ins = ins;
        s.wav = wav;
        s.vol = vol;
        s.tempo = 150;
        s.nosound = true;
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn wrap_music() {
    stopscore();
    with_sound(|s| {
        s.ins = std::ptr::null();
        s.wav = std::ptr::null();
        s.vol = std::ptr::null();
    });
}

/// Sets the tracks to switch to when the current ones loop.
#[unsafe(no_mangle)]
pub extern "C" fn setscore(t1: *const u8, t2: *const u8, t3: *const u8, t4: *const u8) {
    with_sound(|s| {
        for (v, t) in [t1, t2, t3, t4].into_iter().enumerate() {
            s.voices[v].trak_beg = t;
        }
    });
}

/// Starts playing four tracks from the beginning.
#[unsafe(no_mangle)]
pub extern "C" fn playscore(t1: *const u8, t2: *const u8, t3: *const u8, t4: *const u8) {
    with_sound(|s| {
        if s.ins.is_null() || s.wav.is_null() || s.vol.is_null() {
            return;
        }
        for (v, t) in [t1, t2, t3, t4].into_iter().enumerate() {
            let word = unsafe { *s.ins.add(v) };
            let ch = &mut s.chans[v];
            ch.volume = 0;
            ch.active = false;
            s.voices[v] = Voice {
                trak_ptr: t,
                trak_beg: t,
                wave_num: (word >> 8) as u8,
                vol_num: word as u8,
                vol_delay: -1,
                ..Voice::new()
            };
        }
        s.timeclock = 0;
        s.nosound = false;
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn stopscore() {
    with_sound(|s| {
        for v in 0..4 {
            s.silence(v);
            s.chans[v].active = false;
            s.voices[v].vol_delay = 0;
            s.voices[v].stat = 0;
        }
        s.nosound = true;
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn set_tempo(t: i32) {
    with_sound(|s| s.tempo = t & 0xffff);
}

/// One-shot sound effect on voice 2: `len` words of signed 8 bit data at `period`.
#[unsafe(no_mangle)]
pub extern "C" fn playsample(data: *const i8, len: i32, period: i32) {
    if data.is_null() || len <= 0 {
        return;
    }
    let bytes: Vec<i8> = unsafe { std::slice::from_raw_parts(data, len as usize * 2) }.to_vec();
    with_sound(|s| {
        let ch = &mut s.chans[1];
        ch.data = bytes;
        ch.phase = 0.0;
        ch.volume = 64;
        ch.looping = false;
        ch.active = true;
        s.voices[1].stat = 1;
        s.voices[1].vol_delay = -1;
        s.set_period(1, period.max(1) as u32);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn stopsample() {
    with_sound(|s| {
        s.chans[1].active = false;
        s.chans[1].volume = 0;
        s.voices[1].stat = 0;
    });
}
