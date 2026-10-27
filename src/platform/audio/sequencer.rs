//! The music sequencer: plays four tracks of events and mixes the four channels.

use super::channel::Channel;
use super::tables::*;

#[derive(Clone, Copy)]
pub(super) struct Voice {
    pub(super) trak_ptr: *const u8,
    pub(super) trak_beg: *const u8,
    pub(super) wave_num: u8,
    pub(super) vol_num: u8,
    pub(super) vol_delay: i8,
    /// non-zero while a sound effect owns this voice's channel
    pub(super) stat: u8,
    pub(super) event_start: i32,
    pub(super) event_stop: i32,
    pub(super) vol_list: *const u8,
    /// how many times the track has looped back to its start
    pub(super) loops: u32,
}

impl Voice {
    pub(super) const fn new() -> Voice {
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
            loops: 0,
        }
    }
}

pub(super) struct Sound {
    pub(super) voices: [Voice; VOICE_COUNT],
    pub(super) chans: [Channel; VOICE_COUNT],
    pub(super) tempo: i32,
    pub(super) timeclock: i32,
    pub(super) nosound: bool,
    /// instrument table (the game's `new_wave`): high byte = waveform, low byte = envelope
    pub(super) ins: *const u16,
    pub(super) wav: *const u8,
    pub(super) vol: *const u8,
    rate: f64,
    tick_acc: f64,
}

// The raw pointers refer to game memory that outlives the audio device.
unsafe impl Send for Sound {}

impl Sound {
    pub(super) fn new(rate: f64) -> Sound {
        Sound {
            voices: [Voice::new(); VOICE_COUNT],
            chans: [
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
            ],
            tempo: DEFAULT_TEMPO,
            timeclock: 0,
            nosound: true,
            ins: std::ptr::null(),
            wav: std::ptr::null(),
            vol: std::ptr::null(),
            rate,
            tick_acc: 0.0,
        }
    }

    /// Starts playing four tracks from the beginning.
    pub(super) fn play(&mut self, tracks: [*const u8; VOICE_COUNT]) {
        if self.ins.is_null() || self.wav.is_null() || self.vol.is_null() {
            return;
        }

        for (v, t) in tracks.into_iter().enumerate() {
            let word = unsafe { *self.ins.add(v) };
            let ch = &mut self.chans[v];
            ch.volume = 0;
            ch.active = false;
            self.voices[v] = Voice {
                trak_ptr: t,
                trak_beg: t,
                // An instrument word holds the waveform number in its high byte and the volume envelope in the low one.
                wave_num: (word >> 8) as u8,
                vol_num: word as u8,
                vol_delay: -1,
                ..Voice::new()
            };
        }

        self.timeclock = 0;
        self.nosound = false;
    }

    /// True once every voice has either ended or played its track through once.
    pub(super) fn played_through(&self) -> bool {
        self.voices
            .iter()
            .all(|v| v.trak_ptr.is_null() || v.loops >= 1)
    }

    pub(super) fn set_period(&mut self, ch: usize, period: u32) {
        let period = period.max(MIN_PERIOD);
        self.chans[ch].step = PAULA_CLOCK / period as f64 / self.rate;
    }

    /// One vertical blank of the music driver (`vblank_server` in gdriver.asm).
    fn tick(&mut self) {
        self.timeclock = self.timeclock.wrapping_add(self.tempo);

        if self.nosound {
            return;
        }

        for v in 0..VOICE_COUNT {
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
        } else if vc.stat != 0 {
            // a sound effect is playing on this channel
        } else if now.wrapping_sub(vc.event_stop) >= 0 {
            self.chans[v].volume = 0; // gap before the next note
        } else {
            self.follow_envelope(v);
        }
    }

    /// Steps the volume envelope of a sounding note.
    fn follow_envelope(&mut self, v: usize) {
        let vc = self.voices[v];

        if vc.vol_delay != 0 || vc.vol_list.is_null() {
            return;
        }

        let level = unsafe { *vc.vol_list } as i8;

        if level >= 0 {
            self.chans[v].volume = level as u8;

            self.voices[v].vol_list = unsafe { vc.vol_list.add(1) };
        }
    }

    /// Reads events until one makes a sound (or the track ends).
    fn new_note(&mut self, v: usize) {
        loop {
            let (cmd, value) = unsafe {
                let p = self.voices[v].trak_ptr;
                self.voices[v].trak_ptr = p.add(2);

                (*p, *p.add(1))
            };

            if cmd & CMD_NOTE_FLAG == 0 || cmd == CMD_REST {
                self.start_note(v, cmd, value);

                return;
            }

            if !self.run_command(v, cmd, value) {
                return;
            }
        }
    }

    /// Runs a non-note event. Returns false when the track ended.
    fn run_command(&mut self, v: usize, cmd: u8, value: u8) -> bool {
        match cmd {
            CMD_INSTRUMENT => {
                let word = unsafe { *self.ins.add((value & INSTRUMENT_MASK) as usize) };
                // An instrument word holds the waveform number in its high byte and the volume envelope in the low one.
                self.voices[v].wave_num = (word >> 8) as u8;
                self.voices[v].vol_num = word as u8;
            }

            CMD_TEMPO => self.tempo = value as i32,
            CMD_END => return self.end_of_track(v, value != 0),
            _ => {} // unknown command, skip it
        }

        true
    }

    /// Loops back to the start when asked to, otherwise stops the voice. False when stopped.
    fn end_of_track(&mut self, v: usize, looping: bool) -> bool {
        if looping && !self.voices[v].trak_beg.is_null() {
            self.voices[v].loops += 1;
            self.voices[v].trak_ptr = self.voices[v].trak_beg;

            return true;
        }

        self.voices[v].trak_ptr = std::ptr::null();
        self.voices[v].vol_delay = -1;
        self.chans[v].volume = 0;

        false
    }

    fn start_note(&mut self, v: usize, cmd: u8, value: u8) {
        let len = NOTE_LENGTHS[(value & NOTE_LENGTH_MASK) as usize];
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

        let env = unsafe { self.vol.add(vc.vol_num as usize * ENVELOPE_BYTES) };

        let start_vol = unsafe { *env };

        vc.vol_list = unsafe { env.add(1) };
        vc.vol_delay = 0;

        if cmd == CMD_REST {
            self.chans[v].volume = 0;
            vc.event_stop = self.timeclock;

            return;
        }

        self.start_wave(v, cmd, start_vol);
    }

    /// Loads the waveform for note `cmd` into the voice's channel and starts it.
    fn start_wave(&mut self, v: usize, cmd: u8, start_vol: u8) {
        let (period, offset) = NOTES.get(cmd as usize).copied().unwrap_or(FALLBACK_NOTE);
        let wave_start =
            self.voices[v].wave_num as usize * WAVE_BYTES + offset as usize * OFFSET_UNIT;

        let wave = unsafe { self.wav.add(wave_start) };
        let words = WAVE_WORDS - offset as usize;
        let ch = &mut self.chans[v];
        ch.data = (0..words * 2)
            .map(|i| unsafe { *wave.add(i) } as i8)
            .collect();
        ch.looping = true;
        ch.phase = 0.0;
        ch.volume = start_vol.min(MAX_VOLUME);
        ch.active = true;
        self.set_period(v, period as u32);
    }

    /// Mixes `out` (interleaved left, right).
    pub(super) fn mix(&mut self, out: &mut [f32]) {
        let per_tick = self.rate / TICK_HZ;

        for frame in out.chunks_exact_mut(2) {
            self.tick_acc += 1.0;

            if self.tick_acc >= per_tick {
                self.tick_acc -= per_tick;
                self.tick();
            }

            let (left, right) = self.mix_frame();
            frame[0] = (left * MIX_LEVEL).clamp(-1.0, 1.0);
            frame[1] = (right * MIX_LEVEL).clamp(-1.0, 1.0);
        }

        // a finished sound effect gives its voice back to the music
        if self.voices[EFFECT_VOICE].stat != 0 && !self.chans[EFFECT_VOICE].active {
            self.voices[EFFECT_VOICE].stat = 0;
        }
    }

    /// One sample from every channel, summed into (left, right).
    fn mix_frame(&mut self) -> (f32, f32) {
        let (mut left, mut right) = (0.0f32, 0.0f32);

        for c in 0..VOICE_COUNT {
            let s = self.chans[c].next();

            if LEFT_VOICES.contains(&c) {
                left += s;
            } else {
                right += s;
            }
        }

        (left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_sequencer_mixes_silence() {
        let mut sound = Sound::new(44100.0);
        let mut out = [1.0f32; 8];
        sound.mix(&mut out);
        assert!(out.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn end_of_track_without_loop_stops_the_voice() {
        let mut sound = Sound::new(44100.0);
        assert!(!sound.end_of_track(0, false));
        assert!(sound.voices[0].trak_ptr.is_null());
    }

    #[test]
    fn end_of_track_with_loop_returns_to_the_start() {
        let track = [0u8; 2];
        let mut sound = Sound::new(44100.0);
        sound.voices[0].trak_beg = track.as_ptr();
        assert!(sound.end_of_track(0, true));
        assert_eq!(sound.voices[0].trak_ptr, track.as_ptr());
    }
}
