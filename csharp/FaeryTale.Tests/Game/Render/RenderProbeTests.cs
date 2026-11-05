using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Land;
using FaeryTale.Game.Render;

namespace FaeryTale.Tests;

/// Draws the map and some sprites from the real game disk and compares a hash of the screen planes
/// with the one the Rust game produced for the same set-up. Skipped (passes without checking) when
/// the original game files are not there.
[Collection("Game disk")]
public class RenderProbeTests
{
    private const ulong FNV_START = 0xcbf29ce484222325;
    private const ulong FNV_PRIME = 0x100000001b3;

    private static ulong HashPage(Video video)
    {
        ulong hash = FNV_START;

        for (int plane = 0; plane < 5; plane++)
        {
            foreach (byte value in video.Screen.GetBitmap(BitmapId.Page1).Plane(plane))
            {
                hash = (hash ^ value) * FNV_PRIME;
            }
        }

        return hash;
    }

    private static Shape ActorAt(sbyte kind, ushort x, ushort y, ushort relX, ushort relY, sbyte index, sbyte weapon, byte race, sbyte state, sbyte facing) =>
        new() { Kind = kind, AbsX = x, AbsY = y, RelX = relX, RelY = relY, Index = index, Weapon = weapon, Race = race, State = state, Facing = facing };

    private static List<string> Probe(string name, ushort heroX, ushort heroY, ushort region)
    {
        FaeryTale.Game.Data.Game game = new();
        Video video = new();

        video.Screen.SetUp([[], []]);
        video.Graphics.Image = new byte[16384 * 5 + 128];
        video.Graphics.Shape = new byte[78000 + 128];
        video.Graphics.Shadow = new byte[8192 + 4096 + 128];
        game.Map.Allocate();

        // The shadow masks come from disk in the real game; a pattern makes the masking matter.
        for (int index = 0; index < video.Graphics.Shadow.Length; index++)
        {
            video.Graphics.Shadow[index] = (byte)(index * 7 + 3);
        }

        game.Hero.Brother = 1;
        game.World.Region = region;
        game.World.MapX = (ushort)(heroX - 144);
        game.World.MapY = (ushort)(heroY - 90);
        game.World.TileX = (ushort)(game.World.MapX >> 4);
        game.World.TileY = (ushort)(game.World.MapY >> 5);
        game.Actors.LoadedActorFile = 6;
        game.Actors.LoadedSetfigFile = 13;
        game.World.NewRegion = region;
        Regions.LoadRegionNow(game, video);
        SpriteLoading.LoadStartSprites(game, video);
        Regions.RefreshMinimap(game, video);
        video.Screen.DrawTarget = BitmapId.Page1;
        MapTiles.DrawMap(game, video);

        List<string> lines = [$"{name} map {HashPage(video):x16}"];

        game.Actors.List[0] = ActorAt(0, heroX, heroY, 144 - 8, 90 - 26, 5, 1, 255, 13, 0);
        game.Actors.List[1] = ActorAt(3, (ushort)(heroX + 30), (ushort)(heroY + 10), 144 + 22, 90 - 16 + 10, 1, 0, 1, 0, 0);
        game.Actors.List[3] = ActorAt(2, (ushort)(heroX - 40), (ushort)(heroY + 20), 144 - 48, 90 - 26 + 20, 9, 2, 1, 13, 3);
        game.Actors.List[4] = ActorAt(2, (ushort)(heroX + 60), (ushort)(heroY - 25), 144 + 52, 90 - 26 - 25, 3, 4, 2, 13, 6);
        game.Actors.Drawn = 5;
        game.Actors.Active = 5;

        ActorOrder.SortActors(game);
        SpriteDrawing.DrawSprites(game, video, 7);
        lines.Add($"{name} sprites {HashPage(video):x16} used {video.Screen.DrawingPage.ObCount}");

        ActorOrder.UndoSprites(game, video);
        lines.Add($"{name} undone {HashPage(video):x16}");

        return lines;
    }

    [Fact]
    public void TheMapAndSpritesDrawTheSameAsTheRustGame()
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
            string[] expected = File.ReadAllLines(Path.Combine(AppContext.BaseDirectory, "Fixtures", "render_probe_hashes.txt"));
            List<string> actual = [.. Probe("village", 19036, 15755, 3), .. Probe("north", 12000, 4000, 0)];

            Assert.Equal(expected, actual);
        }
        finally
        {
            Disk.Close();
            Environment.SetEnvironmentVariable("FTA_DATA", null);
        }
    }
}
