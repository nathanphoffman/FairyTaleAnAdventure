namespace FaeryTale.Audio;

/// Sound: mixing samples into the output.
public sealed partial class Sound
{
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

        for (int offset = 0; offset + 1 < output.Length; offset += 2)
        {
            tickAccumulator += 1.0;

            if (tickAccumulator >= perTick)
            {
                tickAccumulator -= perTick;
                Tick();
            }

            (float left, float right) = MixFrame();

            output[offset] = Math.Clamp(left * AudioTables.MIX_LEVEL, -1f, 1f);
            output[offset + 1] = Math.Clamp(right * AudioTables.MIX_LEVEL, -1f, 1f);
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
