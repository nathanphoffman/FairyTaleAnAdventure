using FaeryTale.Audio;
using FaeryTale.Platform;

namespace FaeryTale.Tests;

public class AudioTests
{
    private static Sound MakeSound()
    {
        Sound sound = new(44100);
        byte[] waves = new byte[1024];

        for (int index = 0; index < waves.Length; index++)
        {
            waves[index] = (byte)(index % 2 == 0 ? 100 : 156);
        }

        byte[] volumes = new byte[2560];

        Array.Fill(volumes, (byte)64);
        sound.InitMusic([0, 0, 0, 0], waves, volumes);

        return sound;
    }

    [Fact]
    public void TracksAreCutOutOfTheSongFile()
    {
        byte[] data = [0, 0, 0, 2, 1, 2, 3, 4, 0, 0, 0, 1, 9, 9];
        byte[][] tracks = SoundBank.SplitTracks(data);

        Assert.Equal([1, 2, 3, 4], tracks[0]);
        Assert.Equal([9, 9], tracks[1]);
    }

    [Fact]
    public void SamplesAreCutOutByLength()
    {
        byte[] data = [0, 0, 0, 2, 255, 1, 0, 0, 0, 1, 5];
        sbyte[][] samples = SoundBank.SplitSamples(data);

        Assert.Equal([-1, 1], samples[0]);
        Assert.Equal([5], samples[1]);
    }

    [Fact]
    public void ANotePlaysAndTheScoreEndsInSilence()
    {
        Sound sound = MakeSound();
        byte[] track = [24, 0x20, 255, 0];

        sound.Play([track, [], [], []]);

        float[] output = new float[4410];

        sound.Mix(output);

        Assert.Contains(output, sample => sample != 0f);
    }

    [Fact]
    public void ASoundEffectPlaysOnceThenStops()
    {
        Sound sound = MakeSound();
        float[] output = new float[8820];

        sound.PlaySample(Enumerable.Repeat((sbyte)50, 200).ToArray(), 400);
        sound.Mix(output);

        Assert.Contains(output, sample => sample != 0f);
        Assert.Equal(0f, output[^1]);
    }

    [Fact]
    public void ControlsLinesAreParsed()
    {
        Assert.Equal(("W", "Up"), KeyRemap.ParseLine("W = Up"));
        Assert.Equal(("A", "Left"), KeyRemap.ParseLine("  A=Left # strafe"));
        Assert.Null(KeyRemap.ParseLine("# just a comment"));
        Assert.Null(KeyRemap.ParseLine(""));
    }
}
