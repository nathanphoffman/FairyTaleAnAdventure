namespace FaeryTale.Audio;

/// One voice of a tune: pairs of (command, value) bytes.
public sealed class Voice
{
    /// The track being played, and where in it the next event is. Null when the voice is stopped.
    public byte[]? Track;
    public int Pos;

    /// Where the track starts again when it loops.
    public byte[]? TrackStart;
    public byte WaveNum;
    public byte VolNum;
    public sbyte VolDelay;

    /// Non-zero while a sound effect owns this voice's channel.
    public byte Stat;
    public int EventStart;
    public int EventStop;

    /// Where the next level of the volume envelope is in the envelope table.
    public int? VolList;

    /// How many times the track has looped back to its start.
    public uint Loops;
}

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
public sealed class Sound
{
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
            ReplaceVoice(voice, new Voice { Track = tracks[voice], TrackStart = tracks[voice], WaveNum = (byte)(word >> 8), VolNum = (byte)word, VolDelay = -1 });
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

    /// One-shot sound effect on voice 2: signed 8 bit data (a whole number of 16 bit words)
    /// played at period.
    public void PlaySample(sbyte[] data, int period)
    {
        sbyte[] bytes = data[..(data.Length & ~1)];

        if (bytes.Length == 0)
        {
            return;
        }

        Channel channel = Chans[AudioTables.EFFECT_VOICE];

        channel.Data = bytes;
        channel.Phase = 0;
        channel.Volume = AudioTables.MAX_VOLUME;
        channel.Looping = false;
        channel.Active = true;
        Voices[AudioTables.EFFECT_VOICE].Stat = 1;
        Voices[AudioTables.EFFECT_VOICE].VolDelay = -1;
        SetPeriod(AudioTables.EFFECT_VOICE, (uint)Math.Max(period, 1));
    }

    /// Mixes the output (interleaved left, right).
    public void Mix(Span<float> output)
    {
        double perTick = rate / AudioTables.TICK_HZ;

        for (int at = 0; at + 1 < output.Length; at += 2)
        {
            tickAccumulator += 1.0;

            if (tickAccumulator >= perTick)
            {
                tickAccumulator -= perTick;
                Tick();
            }

            (float left, float right) = MixFrame();

            output[at] = Math.Clamp(left * AudioTables.MIX_LEVEL, -1f, 1f);
            output[at + 1] = Math.Clamp(right * AudioTables.MIX_LEVEL, -1f, 1f);
        }

        // A finished sound effect gives its voice back to the music.
        if (Voices[AudioTables.EFFECT_VOICE].Stat != 0 && !Chans[AudioTables.EFFECT_VOICE].Active)
        {
            Voices[AudioTables.EFFECT_VOICE].Stat = 0;
        }
    }

    /// True once every voice has either ended or played its track through once.
    internal bool PlayedThrough() => Voices.All(voice => voice.Track is null || voice.Loops >= 1);

    internal void SetPeriod(int channel, uint period)
    {
        period = Math.Max(period, AudioTables.MIN_PERIOD);
        Chans[channel].Step = AudioTables.PAULA_CLOCK / period / rate;
    }

    // Loops back to the start when asked to, otherwise stops the voice. False when stopped.
    internal bool EndOfTrack(int voice, bool looping)
    {
        Voice v = Voices[voice];

        if (looping && v.TrackStart is not null)
        {
            v.Loops++;
            v.Track = v.TrackStart;
            v.Pos = 0;

            return true;
        }

        v.Track = null;
        v.VolDelay = -1;
        Chans[voice].Volume = 0;

        return false;
    }

    private void ReplaceVoice(int index, Voice voice) => Voices[index] = voice;

    // One vertical blank of the music driver.
    private void Tick()
    {
        TimeClock = unchecked(TimeClock + Tempo);

        if (NoSound)
        {
            return;
        }

        for (int voice = 0; voice < AudioTables.VOICE_COUNT; voice++)
        {
            DoVoice(voice);
        }
    }

    private void DoVoice(int voice)
    {
        Voice v = Voices[voice];

        if (v.Track is null)
        {
            return;
        }

        int now = TimeClock;

        if (unchecked(now - v.EventStart) >= 0)
        {
            NewNote(voice);
        }
        else if (v.Stat != 0)
        {
            // A sound effect is playing on this channel.
        }
        else if (unchecked(now - v.EventStop) >= 0)
        {
            Chans[voice].Volume = 0; // gap before the next note
        }
        else
        {
            FollowEnvelope(voice);
        }
    }

    // Steps the volume envelope of a sounding note.
    private void FollowEnvelope(int voice)
    {
        Voice v = Voices[voice];

        if (v.VolList is not { } at || v.VolDelay != 0)
        {
            return;
        }

        sbyte level = unchecked((sbyte)(at < Volumes.Length ? Volumes[at] : (byte)0xff));

        if (level >= 0)
        {
            Chans[voice].Volume = (byte)level;
            v.VolList = at + 1;
        }
    }

    // Reads events until one makes a sound (or the track ends).
    private void NewNote(int voice)
    {
        while (true)
        {
            (byte command, byte value) = NextEvent(voice);

            if ((command & AudioTables.CMD_NOTE_FLAG) == 0 || command == AudioTables.CMD_REST)
            {
                StartNote(voice, command, value);

                return;
            }

            if (!RunCommand(voice, command, value))
            {
                return;
            }
        }
    }

    // The next (command, value) pair of the voice's track. Past the end of the data it is the
    // end-of-track command.
    private (byte Command, byte Value) NextEvent(int voice)
    {
        Voice v = Voices[voice];
        int at = v.Pos;

        v.Pos += 2;

        return v.Track is { } track && at + 2 <= track.Length ? (track[at], track[at + 1]) : (AudioTables.CMD_END, (byte)0);
    }

    // Runs a non-note event. Returns false when the track ended.
    private bool RunCommand(int voice, byte command, byte value)
    {
        switch (command)
        {
            case AudioTables.CMD_INSTRUMENT:
                int index = value & AudioTables.INSTRUMENT_MASK;
                ushort word = index < Instruments.Length ? Instruments[index] : (ushort)0;

                // An instrument word holds the waveform number in its high byte and the volume envelope in the low one.
                Voices[voice].WaveNum = (byte)(word >> 8);
                Voices[voice].VolNum = (byte)word;

                break;

            case AudioTables.CMD_TEMPO:
                Tempo = value;

                break;

            case AudioTables.CMD_END:
                return EndOfTrack(voice, value != 0);
        }

        return true;
    }

    private void StartNote(int voice, byte command, byte value)
    {
        int length = AudioTables.NOTE_LENGTHS[value & AudioTables.NOTE_LENGTH_MASK];
        int stop = length - AudioTables.GAP;

        if (stop < 0)
        {
            stop = length;
        }

        Voice v = Voices[voice];

        v.EventStop = unchecked(v.EventStart + stop);
        v.EventStart = unchecked(v.EventStart + length);

        if (v.Stat != 0)
        {
            return; // sound effect has the channel
        }

        int envelope = v.VolNum * AudioTables.ENVELOPE_BYTES;
        byte startVolume = envelope < Volumes.Length ? Volumes[envelope] : (byte)0;

        v.VolList = envelope + 1;
        v.VolDelay = 0;

        if (command == AudioTables.CMD_REST)
        {
            Chans[voice].Volume = 0;
            v.EventStop = TimeClock;

            return;
        }

        StartWave(voice, command, startVolume);
    }

    // Loads the waveform for note command into the voice's channel and starts it.
    private void StartWave(int voice, byte command, byte startVolume)
    {
        (ushort period, ushort offset) = command < AudioTables.NOTES.Length ? AudioTables.NOTES[command] : AudioTables.FALLBACK_NOTE;
        int waveStart = Voices[voice].WaveNum * AudioTables.WAVE_BYTES + offset * AudioTables.OFFSET_UNIT;
        int words = AudioTables.WAVE_WORDS - offset;
        Channel channel = Chans[voice];

        channel.Data = new sbyte[words * 2];

        for (int index = 0; index < channel.Data.Length; index++)
        {
            int at = waveStart + index;

            channel.Data[index] = unchecked((sbyte)(at < Waves.Length ? Waves[at] : (byte)0));
        }

        channel.Looping = true;
        channel.Phase = 0;
        channel.Volume = Math.Min(startVolume, AudioTables.MAX_VOLUME);
        channel.Active = true;
        SetPeriod(voice, period);
    }

    // One sample from every channel, summed into (left, right).
    private (float Left, float Right) MixFrame()
    {
        float left = 0f;
        float right = 0f;

        for (int channel = 0; channel < AudioTables.VOICE_COUNT; channel++)
        {
            float sample = Chans[channel].Next();

            if (AudioTables.LEFT_VOICES.Contains(channel))
            {
                left += sample;
            }
            else
            {
                right += sample;
            }
        }

        return (left, right);
    }
}
