namespace FaeryTale.Audio;

/// One hardware channel of the sound chip: plays signed 8 bit data at a rate set by its period.
internal sealed class Channel
{
    public sbyte[] Data = [];
    public double Phase;

    /// Data bytes per output sample.
    public double Step;

    /// 0 to MAX_VOLUME.
    public byte Volume;

    public bool Active;
    public bool Looping = true;

    /// Next output sample, -1.0 to 1.0.
    public float Next()
    {
        if (!Active || Data.Length == 0)
        {
            return 0f;
        }

        int length = Data.Length;
        int index = (int)Phase;
        float a = Data[Math.Min(index, length - 1)];
        float b = index + 1 < length ? Data[index + 1] : Looping ? Data[0] : a;
        float fraction = (float)(Phase - Math.Floor(Phase));
        float sample = (a + (b - a) * fraction) / 128f * (Volume / (float)AudioTables.MAX_VOLUME);

        Advance();

        return sample;
    }

    // Moves along the data, wrapping when looping and stopping at the end otherwise.
    private void Advance()
    {
        Phase += Step;
        double length = Data.Length;

        if (Phase < length)
        {
            return;
        }

        if (Looping)
        {
            Phase -= length;
        }
        else
        {
            Active = false;
        }
    }
}
