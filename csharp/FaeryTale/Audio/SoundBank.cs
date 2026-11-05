using FaeryTale.Game.Files;

namespace FaeryTale.Audio;

/// The music scores and sound effects that were read from the game files.
public static class SoundBank
{
    /// The file with the song tracks.
    private const string SONG_FILE = "songs";

    // Songs: up to 28 tracks, packed into the score memory.
    private const int TRACK_COUNT = 4 * 7;
    private const int SCORE_MEMORY = 5900;

    /// A length in the score and sample files is a 4 byte big-endian number.
    private const int LENGTH_BYTES = 4;

    // Where the sound effects are on the disk (block) and how many blocks they take.
    private const short SAMPLE_BLOCK = 920;
    private const short SAMPLE_BLOCKS = 11;

    /// There are 6 sound effects.
    private const int SAMPLE_COUNT = 6;

    /// Each tune uses four tracks.
    private const int TRACKS_PER_TUNE = 4;

    private static byte[][] tracks = [];
    private static sbyte[][] samples = [];

    /// Music track number n (four to a tune). A track that was not read is empty, which is silence.
    public static byte[] Track(int n) => n >= 0 && n < tracks.Length ? tracks[n] : [];

    /// The four tracks of tune tune.
    public static byte[][] TuneTracks(int tune) => Enumerable.Range(0, TRACKS_PER_TUNE).Select(voice => Track(tune * TRACKS_PER_TUNE + voice)).ToArray();

    /// The sound effect num, or null if there is none.
    public static sbyte[]? Sample(int num) => num >= 0 && num < samples.Length ? samples[num] : null;

    /// Reads the song tracks from the file "songs".
    public static void ReadScore()
    {
        string? path = Paths.Resolve(SONG_FILE, false);

        if (path is null || !File.Exists(path))
        {
            return;
        }

        tracks = SplitTracks(File.ReadAllBytes(path));
    }

    /// Reads the sound effects from the disk.
    public static void ReadSample() => samples = SplitSamples(Disk.ReadBlocks(SAMPLE_BLOCK, SAMPLE_BLOCKS));

    /// Cuts the tracks out of the song file's data.
    ///
    /// The file is a list of tracks, each a 4 byte big-endian length (counted in 16-bit words) and
    /// then that many words. There are up to 28 tracks (4 voices times 7 songs). Reading stops
    /// early if they would not fit in the 5900 bytes of score memory.
    internal static byte[][] SplitTracks(byte[] data)
    {
        List<byte[]> result = [];
        int at = 0;
        int loaded = 0;

        for (int count = 0; count < TRACK_COUNT; count++)
        {
            int bytes = (int)Be32(data, at) * 2;

            at += LENGTH_BYTES;

            if (bytes + loaded > SCORE_MEMORY)
            {
                break;
            }

            int end = Math.Min(at + bytes, data.Length);
            int start = Math.Min(at, end);

            result.Add(data[start..end]);
            at += bytes;
            loaded += bytes;
        }

        return [.. result];
    }

    /// Cuts the sound effects out of the data read from the disk: each one is a 4 byte
    /// big-endian length followed by that many bytes of sound.
    internal static sbyte[][] SplitSamples(byte[] data)
    {
        sbyte[][] result = new sbyte[SAMPLE_COUNT][];
        int at = 0;

        for (int count = 0; count < SAMPLE_COUNT; count++)
        {
            int length = (int)Be32(data, at);

            at += LENGTH_BYTES;

            int end = Math.Min(at + length, data.Length);
            int start = Math.Min(at, end);

            result[count] = data[start..end].Select(value => unchecked((sbyte)value)).ToArray();
            at += length;
        }

        return result;
    }

    // The big-endian number at at, or 0 when the data ends first.
    private static uint Be32(byte[] data, int at) =>
        at >= 0 && at + LENGTH_BYTES <= data.Length ? (uint)(data[at] << 24 | data[at + 1] << 16 | data[at + 2] << 8 | data[at + 3]) : 0;
}
