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
