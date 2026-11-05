using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Platform;

namespace FaeryTale.Tests;

public class SmoothMotionTests
{
    private const double TICK_MS = 20;
    private const double HERO_STEP = 3;

    // Frames of the given lengths (in ticks) with a picture every pictureMs; returns the camera x
    // of every picture, the way the renderer works it out (hero position minus the unfinished part
    // of the last step).
    private static List<double> CameraPositions(int[] frameTicks, double pictureMs)
    {
        FrameSpan span = new();
        List<double> camera = [];
        double frameStart = 0;
        double position = 0;
        double picture = 0;

        foreach (int ticks in frameTicks)
        {
            span.Begin(frameStart);
            position += HERO_STEP;

            for (; picture < frameStart + ticks * TICK_MS; picture += pictureMs)
            {
                camera.Add(position - HERO_STEP * (1 - span.AlphaAt(picture)));
            }

            frameStart += ticks * TICK_MS;
        }

        return camera;
    }

    private static void AssertEvenSteps(List<double> camera, int skip)
    {
        double[] steps = camera.Skip(skip).Zip(camera.Skip(skip + 1), (before, after) => after - before).ToArray();
        double average = steps.Average();

        Assert.All(steps, step => Assert.InRange(step, average * 0.9, average * 1.1));
    }

    [Fact]
    public void TheCameraMovesEvenlyWhenEveryFrameIsTwoTicks()
    {
        List<double> camera = CameraPositions(Enumerable.Repeat(2, 60).ToArray(), 1000.0 / 144);

        AssertEvenSteps(camera, 20);
    }

    [Fact]
    public void TheCameraNeverStandsStillInTheMiddleOfAWalk()
    {
        List<double> camera = CameraPositions(Enumerable.Repeat(2, 60).ToArray(), 1000.0 / 60);

        Assert.All(camera.Skip(20).Zip(camera.Skip(21), (before, after) => after - before), step => Assert.True(step > 0));
    }

    // The frame's length: ticks the game waits in the print queue routine when a request is handled.
    private static uint TicksWaited(Experience experience, bool requestQueued)
    {
        FaeryTale.Game.Data.Game game = new() { Experience = experience };
        Recorder recorder = new();

        if (requestQueued)
        {
            game.Ui.PrintQueue.Items[0] = 4;
            game.Ui.PrintQueue.Write = 1;
        }

        ScreenEffects.HandlePrintRequest(game, recorder);

        return recorder.Ticks;
    }

    [Fact]
    public void RemasteredFramesAreTheSameLengthWithOrWithoutAPrintRequest()
    {
        Assert.Equal(TicksWaited(Experience.Remastered, false), TicksWaited(Experience.Remastered, true));
    }

    [Fact]
    public void TheOriginalKeepsItsShortFrameWhenItHandlesARequest()
    {
        Assert.Equal(1u, TicksWaited(Experience.Original, false));
        Assert.Equal(0u, TicksWaited(Experience.Original, true));
    }
}
