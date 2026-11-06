using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Land;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using FaeryTale.Platform;

namespace FaeryTale.Tests;

public class WorldDrawerTests
{
    [Fact]
    public void AScreenPointAndItsWorldPositionGoBothWays()
    {
        WorldView view = new();

        view.Resize(1920, 1080, 228);
        view.SetZoom(0.5);
        view.CenterX = 19036;
        view.CenterY = 15743;

        (double worldX, double worldY) = view.ToWorld(1200, 300);
        (double screenX, double screenY) = view.ToScreen(worldX, worldY);

        Assert.Equal(1200, screenX, 6);
        Assert.Equal(300, screenY, 6);
    }

    [Fact]
    public void TheHeroIsInTheMiddleOfTheWorldAboveThePanel()
    {
        WorldView view = new();

        view.Resize(1920, 1080, 228);
        view.CenterX = 500;
        view.CenterY = 700;

        (double x, double y) = view.ToScreen(500, 700);

        Assert.Equal(960, x, 6);
        Assert.Equal((1080 - 228) / 2, y, 6);
    }

    [Fact]
    public void TheScaleIsAlwaysAWholeNumberSoPixelsStaySquare()
    {
        WorldView view = new();

        view.Resize(1920, 1080, 228);

        foreach (double zoom in new[] { 0.25, 0.37, 0.5, 0.8, 1.0, 1.3, 2.0 })
        {
            view.SetZoom(zoom);

            Assert.InRange(view.Scale, view.MinScale, view.MaxScale);
        }
    }

    [Fact]
    public void ZoomOutShowsAboutFourTimesAsMuchAndInTwiceAsClose()
    {
        WorldView view = new();

        view.Resize(1920, 1080, 228);
        view.SetZoom(1);

        double normal = view.Scale;

        view.SetZoom(0.01);
        Assert.InRange(view.Scale / normal, 0.2, 0.34);

        view.SetZoom(100);
        Assert.InRange(view.Scale / normal, 1.8, 2.2);
    }

    [Fact]
    public void ZoomStepsGoOutAndInAndStopAtTheLimits()
    {
        WorldView view = new();

        view.Resize(1920, 1080, 228);
        view.SetZoom(1);

        int normal = view.Scale;

        view.ZoomBySteps(-1);
        Assert.True(view.Scale < normal);

        view.ZoomBySteps(-50);
        Assert.Equal(view.MinScale, view.Scale);

        view.ZoomBySteps(50);
        Assert.Equal(view.MaxScale, view.Scale);
    }

    [Fact]
    public void EveryZoomStepChangesTheScaleEvenWhenTheScaleIsSmall()
    {
        WorldView view = new();

        view.Resize(1280, 720, 114);
        view.SetZoom(0.5);

        int before = view.Scale;

        view.ZoomBySteps(1);
        Assert.True(view.Scale > before);
    }

    [Fact]
    public void VSyncTurnsOnAndOffAndTheFixedRateGoesRound()
    {
        int saved = WindowControls.FrameRate;

        try
        {
            WindowControls.FrameRate = WindowControls.MONITOR_SYNC;
            WindowControls.CycleFrameRate();
            Assert.Equal(WindowControls.MONITOR_SYNC, WindowControls.FrameRate);
            Assert.Equal("Monitor", WindowControls.FrameRateText);

            WindowControls.ToggleVSync();
            Assert.False(WindowControls.VSync);
            Assert.Equal(150, WindowControls.FrameRate);

            WindowControls.FrameRate = 300;
            WindowControls.CycleFrameRate();
            Assert.Equal(50, WindowControls.FrameRate);

            WindowControls.ToggleVSync();
            Assert.True(WindowControls.VSync);
        }
        finally
        {
            WindowControls.FrameRate = saved;
        }
    }

    [Fact]
    public void ATileIsPutTogetherFromItsFivePlanes()
    {
        Graphics graphics = new() { Image = new byte[5 * 16384] };
        int tile = 3;

        // Pixel (0, 0): bit set in plane 0 and plane 4 -> colour 17. Pixel (15, 1): plane 1 only -> colour 2.
        graphics.Image[0 * 16384 + tile * 64] = 0x80;
        graphics.Image[4 * 16384 + tile * 64] = 0x80;
        graphics.Image[1 * 16384 + tile * 64 + 2 + 1] = 0x01;

        byte[] pixels = new WorldImages().Tile(graphics, tile);

        Assert.Equal(17, pixels[0]);
        Assert.Equal(2, pixels[1 * 16 + 15]);
        Assert.Equal(0, pixels[5]);
    }

    [Fact]
    public void ASpriteHasItsOwnMask()
    {
        // One word wide, two lines high: a plane is 4 bytes, a picture 5 planes (20 bytes), then its mask.
        Graphics graphics = new() { Shape = new byte[20 + 4] };
        SeqInfo sheet = new() { Width = 1, Height = 2, Count = 1, Location = 0, MaskLoc = 20, Bytes = 4 };

        graphics.Shape[0] = 0x80;       // plane 0, line 0, pixel 0
        graphics.Shape[20] = 0x80;      // mask: that pixel is part of the figure

        SpriteImage? image = new WorldImages().Sprite(graphics, sheet, 0);

        Assert.NotNull(image);
        Assert.Equal(16, image!.Width);
        Assert.Equal(2, image.Height);
        Assert.Equal(1, image.Pixels[0]);
        Assert.True(image.Opaque[0]);
        Assert.False(image.Opaque[1]);
    }

    [Fact]
    public void ASpriteThatIsNotInMemoryIsNotDrawn()
    {
        Graphics graphics = new() { Shape = new byte[8] };
        SeqInfo sheet = new() { Width = 2, Height = 32, Count = 4, Location = 0, MaskLoc = 5000, Bytes = 128 };

        Assert.Null(new WorldImages().Sprite(graphics, sheet, 3));
    }

    [Fact]
    public void WhatMovedInTheLastTickIsDrawnThatMuchBackBetweenTicks()
    {
        WorldCapture capture = new() { Enabled = true };
        FaeryTale.Game.Data.Game game = new();

        game.Actors.List[0].AbsX = 100;
        game.Actors.List[0].AbsY = 200;
        capture.BeginFrame(game);

        game.Actors.List[0].AbsX = 103;
        capture.BeginFrame(game);
        capture.Prepare();

        Assert.Equal((-3, 0), capture.HeroDelta);
    }

    [Fact]
    public void ATeleportIsNotSmoothedAcrossTheMap()
    {
        WorldCapture capture = new() { Enabled = true };
        FaeryTale.Game.Data.Game game = new();

        game.Actors.List[0].AbsX = 100;
        capture.BeginFrame(game);

        game.Actors.List[0].AbsX = 9000;
        capture.BeginFrame(game);
        capture.Prepare();

        Assert.Equal((0, 0), capture.HeroDelta);
    }

    [Fact]
    public void WithoutAWorldTheComposerShowsTheOldPageScaledAndCentred()
    {
        NativeComposer composer = new();
        Scene scene = new();

        uint[] picture = composer.Compose(1920, 1080, scene, new Overlay(), null, 1.0, (0, 0));

        Assert.Equal(1920 * 1080, picture.Length);
        Assert.False(composer.ShowsWorld);
        Assert.Equal(2, composer.Scale);
        Assert.Equal((1920 - 1280) / 2, composer.GridLeft);
        Assert.False(composer.IsOverPanel(100, 100));
        Assert.Equal((0, 0), composer.ToGrid(composer.GridLeft, composer.GridTop));
    }

    // Draws the real starting village with the new drawer.
    [Collection("Game disk")]
    public class WithTheRealMap
    {
        [Fact]
        public void TheVillageIsDrawnFromTheGamesOwnTiles()
        {
            string? folder = TestData.FindGameFolder();

            if (folder is null)
            {
                return;
            }

            Environment.SetEnvironmentVariable("FTA_DATA", folder);

            try
            {
                Assert.Equal(0, Disk.Open());

                FaeryTale.Game.Data.Game game = new() { Experience = Experience.Remastered };
                Video video = new();

                // The memory the game sets up when it starts: the tile pictures (5 planes of 16384).
                video.Graphics.Image = new byte[5 * 16384 + 4096];
                game.Map.Allocate();
                game.World.MapX = 18892;
                game.World.MapY = 15675;
                game.World.TileX = (ushort)(game.World.MapX >> 4);
                game.World.TileY = (ushort)(game.World.MapY >> 5);
                game.World.NewRegion = 3;
                Regions.LoadRegionNow(game, video);
                Regions.RefreshMinimap(game, video);
                game.Actors.List[0].AbsX = 19036;
                game.Actors.List[0].AbsY = 15755;

                // A palette with different colours so tiles can be told apart.
                video.Screen.VpPage.Colors = new ushort[32];

                for (int index = 0; index < 32; index++)
                {
                    video.Screen.VpPage.Colors[index] = (ushort)(index * 0x081);
                }

                video.Capture.Enabled = true;
                video.Capture.BeginFrame(game);

                WorldView view = new();
                uint[] frame = new uint[640 * 360];

                view.Resize(640, 360, 0);
                new WorldRenderer().Draw(frame, view, video.Capture, video, 1.0);

                Assert.True(frame.Distinct().Count() > 4, "the village should have several colours");
                Assert.Contains(frame, pixel => pixel != 0xff000000);
            }
            finally
            {
                Disk.Close();
                Environment.SetEnvironmentVariable("FTA_DATA", null);
            }
        }
    }

    [Fact]
    public void TheClickRingGrowsWithTheZoom()
    {
        var small = FaeryTale.Game.Remastered.ClickRing.PictureAtPixel((100, 100), true, 2);
        var big = FaeryTale.Game.Remastered.ClickRing.PictureAtPixel((100, 100), true, 8);

        Assert.InRange(big.Width / (double)small.Width, 3.8, 4.2);
    }
}
