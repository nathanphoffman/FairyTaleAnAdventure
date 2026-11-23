namespace FaeryTale.Audio;

/// The music sequencer: plays four tracks of events and mixes the four channels.
///
/// The game plays music from four tracks of (command, value) events, one per voice, and one-shot
/// sound effects on voice 2. The original ran the sequencer from the vertical blank interrupt and
/// let the chip play the waveforms by DMA. Here the audio callback does both: it advances the
/// sequencer 60 times a second and mixes the four channels (voices 1 and 4 left, 2 and 3 right,
/// as in the original).
///
/// Track format: command byte < 128 is a note number and the value's low 6 bits index
/// NOTE_LENGTHS; 128 = rest, 129 = set instrument, 0x90 = set tempo, 255 = end of track (value
/// non-zero = loop).
public sealed partial class Sound
{
    // An instrument word's waveform number is its high byte, so it is shifted down by one byte.
    private const int BITS_PER_BYTE = 8;

    internal readonly Voice[] Voices = Enumerable.Range(0, AudioTables.VOICE_COUNT).Select(_ => new Voice()).ToArray();
    internal readonly Channel[] Chans = Enumerable.Range(0, AudioTables.VOICE_COUNT).Select(_ => new Channel()).ToArray();
    internal int Tempo = AudioTables.DEFAULT_TEMPO;
    internal int TimeClock;
    internal bool NoSound = true;

    /// The instrument table (the game's new_wave): high byte = waveform, low byte = envelope.
    internal ushort[] Instruments = [];
    internal byte[] Waves = [];
    internal byte[] Volumes = [];

    private readonly double rate;
    private double tickAccumulator;

    public Sound(double rate) => this.rate = rate;

    /// Gives the music driver its data: instruments is the table of instrument words (waveform
    /// number in the high byte, volume envelope in the low one), waves 8 waveforms of 128 bytes
    /// and volumes 10 volume envelopes of 256 bytes.
    public void InitMusic(ushort[] instruments, byte[] waves, byte[] volumes)
    {
        Instruments = instruments;
        Waves = waves;
        Volumes = volumes;
        Tempo = AudioTables.DEFAULT_TEMPO;
        NoSound = true;
    }

    /// Changes one entry of the instrument table (the game does this to give caves and houses
    /// their own sound).
    public void SetInstrument(int slot, ushort word)
    {
        if (slot >= 0 && slot < Instruments.Length)
        {
            Instruments[slot] = word;
        }
    }

    /// Sets the tracks to switch to when the current ones loop.
    public void SetScore(byte[][] tracks)
    {
        for (int voice = 0; voice < tracks.Length; voice++)
        {
            Voices[voice].TrackStart = tracks[voice];
        }
    }

    /// Starts playing four tracks from the beginning.
    public void Play(byte[][] tracks)
    {
        if (Instruments.Length == 0 || Waves.Length == 0 || Volumes.Length == 0)
        {
            return;
        }

        for (int voice = 0; voice < tracks.Length; voice++)
        {
            ushort word = voice < Instruments.Length ? Instruments[voice] : (ushort)0;

            Chans[voice].Volume = 0;
            Chans[voice].Active = false;

            // An instrument word holds the waveform number in its high byte and the volume envelope in the low one.
            Voices[voice] = null!;
            ReplaceVoice(voice, new Voice { Track = tracks[voice], TrackStart = tracks[voice], WaveNum = (byte)(word >> BITS_PER_BYTE), VolNum = (byte)word, VolDelay = -1 }); // word >> 8 is the high byte, the cast to byte keeps the low one
        }

        TimeClock = 0;
        NoSound = false;
    }

    /// Stops the music.
    public void StopScore()
    {
        for (int voice = 0; voice < AudioTables.VOICE_COUNT; voice++)
        {
            Chans[voice].Volume = 0;
            Chans[voice].Active = false;
            Voices[voice].VolDelay = 0;
            Voices[voice].Stat = 0;
        }

        NoSound = true;
    }
}
