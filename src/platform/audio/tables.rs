//! Constant tables and numbers of the music driver.

/// (period, start offset in units of 4 bytes) for each note; high notes play a shorter copy of
/// the waveform stored later in its 128 bytes.
pub(super) const NOTES: [(u16, u16); 78] = [
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
pub(super) const NOTE_LENGTHS: [i32; 64] = [
    26880, 13440, 6720, 3360, 1680, 840, 420, 210, 40320, 20160, 10080, 5040, 2520, 1260, 630, 315,
    17920, 8960, 4480, 2240, 1120, 560, 280, 140, 26880, 13440, 6720, 3360, 1680, 840, 420, 210,
    21504, 10752, 5376, 2688, 1344, 672, 336, 168, 32256, 16128, 8064, 4032, 2016, 1008, 504, 252,
    23040, 11520, 5760, 2880, 1440, 720, 360, 180, 34560, 17280, 8640, 4320, 2160, 1080, 540, 270,
];

pub(super) const PAULA_CLOCK: f64 = 3_546_895.0; // PAL; the period values are relative to this
pub(super) const TICK_HZ: f64 = 60.0; // the sequencer ran from the vertical blank
pub(super) const GAP: i32 = 300; // silence at the end of a note
/// Hardware limit: periods below this are not honoured.
pub(super) const MIN_PERIOD: u32 = 124;

/// Played for note numbers past the end of `NOTES`.
pub(super) const FALLBACK_NOTE: (u16, u16) = (508, 0);

pub(super) const VOICE_COUNT: usize = 4;
pub(super) const MAX_VOLUME: u8 = 64;
pub(super) const DEFAULT_TEMPO: i32 = 150;

/// The voice that sound effects take over.
pub(super) const EFFECT_VOICE: usize = 1;

/// Voices 1 and 4 play on the left, 2 and 3 on the right, as in the original.
pub(super) const LEFT_VOICES: [usize; 2] = [0, 3];

/// Scales the four mixed channels down so they rarely clip.
pub(super) const MIX_LEVEL: f32 = 0.45;

/// Track commands (the command byte of an event).
pub(super) const CMD_NOTE_FLAG: u8 = 0x80;
pub(super) const CMD_REST: u8 = 128;
pub(super) const CMD_INSTRUMENT: u8 = 129;
pub(super) const CMD_TEMPO: u8 = 0x90;
pub(super) const CMD_END: u8 = 255;
pub(super) const NOTE_LENGTH_MASK: u8 = 0x3f;
pub(super) const INSTRUMENT_MASK: u8 = 15;

/// Waveforms are 128 bytes (32 words); volume envelopes are 256 bytes.
pub(super) const WAVE_BYTES: usize = 128;
pub(super) const WAVE_WORDS: usize = 32;
pub(super) const ENVELOPE_BYTES: usize = 256;

/// `NOTES` start offsets are in units of this many bytes.
pub(super) const OFFSET_UNIT: usize = 4;

pub(super) const SAMPLE_RATE: i32 = 44100;
pub(super) const OUTPUT_CHANNELS: u8 = 2;
pub(super) const BUFFER_SAMPLES: u16 = 1024;
