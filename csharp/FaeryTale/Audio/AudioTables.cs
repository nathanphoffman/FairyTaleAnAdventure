namespace FaeryTale.Audio;

/// Constant tables and numbers of the music driver.
internal static class AudioTables
{
    /// (period, start offset in units of 4 bytes) for each note; high notes play a shorter copy of
    /// the waveform stored later in its 128 bytes.
    public static readonly (ushort Period, ushort Offset)[] NOTES =
    [
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
        (135, 28)
    ];

    /// Note durations in sequencer clock units, indexed by the low 6 bits of the value byte.
    public static readonly int[] NOTE_LENGTHS =
    [
        26880, 13440, 6720, 3360, 1680, 840, 420, 210,
        40320, 20160, 10080, 5040, 2520, 1260, 630, 315,
        17920, 8960, 4480, 2240, 1120, 560, 280, 140,
        26880, 13440, 6720, 3360, 1680, 840, 420, 210,
        21504, 10752, 5376, 2688, 1344, 672, 336, 168,
        32256, 16128, 8064, 4032, 2016, 1008, 504, 252,
        23040, 11520, 5760, 2880, 1440, 720, 360, 180,
        34560, 17280, 8640, 4320, 2160, 1080, 540, 270
    ];

    public const double PAULA_CLOCK = 3_546_895.0; // PAL; the period values are relative to this
    public const double TICK_HZ = 60.0; // the sequencer ran from the vertical blank
    public const int GAP = 300; // silence at the end of a note

    /// Hardware limit: periods below this are not honoured.
    public const uint MIN_PERIOD = 124;

    /// Played for note numbers past the end of NOTES.
    public static readonly (ushort Period, ushort Offset) FALLBACK_NOTE = (508, 0);

    public const int VOICE_COUNT = 4;
    public const byte MAX_VOLUME = 64;
    public const int DEFAULT_TEMPO = 150;

    /// The voice that sound effects take over.
    public const int EFFECT_VOICE = 1;

    /// Voices 1 and 4 play on the left, 2 and 3 on the right, as in the original.
    public static readonly int[] LEFT_VOICES = [0, 3];

    /// Scales the four mixed channels down so they rarely clip.
    public const float MIX_LEVEL = 0.45f;

    // Track commands (the command byte of an event).
    public const byte CMD_NOTE_FLAG = 0x80;
    public const byte CMD_REST = 128;
    public const byte CMD_INSTRUMENT = 129;
    public const byte CMD_TEMPO = 0x90;
    public const byte CMD_END = 255;
    public const byte NOTE_LENGTH_MASK = 0x3f;
    public const byte INSTRUMENT_MASK = 15;

    // Waveforms are 128 bytes (32 words); volume envelopes are 256 bytes.
    public const int WAVE_BYTES = 128;
    public const int WAVE_WORDS = 32;
    public const int ENVELOPE_BYTES = 256;

    /// NOTES start offsets are in units of this many bytes.
    public const int OFFSET_UNIT = 4;

    public const int SAMPLE_RATE = 44100;
    public const byte OUTPUT_CHANNELS = 2;
    public const ushort BUFFER_SAMPLES = 1024;
}
