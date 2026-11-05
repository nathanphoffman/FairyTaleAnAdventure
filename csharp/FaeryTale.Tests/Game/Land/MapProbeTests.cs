using FaeryTale.Game.Files;
using FaeryTale.Game.Land;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;

namespace FaeryTale.Tests;

/// Loads every region from the real game disk and hashes what the map lookups return, then
/// compares with hashes the Rust game produced for the same positions. Skipped (passes without
/// checking) when the original game files are not there.
[Collection("Game disk")]
public class MapProbeTests
{
    private const ulong FNV_START = 0xcbf29ce484222325;
    private const ulong FNV_PRIME = 0x100000001b3;

    private static void Mix(ref ulong hash, long value)
    {
        foreach (byte part in BitConverter.GetBytes(value))
        {
            hash = (hash ^ part) * FNV_PRIME;
        }
    }

    private static string HashRegion(ushort region, out int outputCalls)
    {
        FaeryTale.Game.Data.Game game = new();
        Recorder output = new();
        game.Map.Allocate();

        uint mapX = region < 8 ? (region & 1u) * 16384 + 5000 : 8000;
        uint mapY = region < 8 ? (uint)(region >> 1) * 8192 + 2000 : 35000;

        game.World.MapX = (ushort)mapX;
        game.World.MapY = (ushort)mapY;
        game.World.TileX = (ushort)(mapX >> 4);
        game.World.TileY = (ushort)(mapY >> 5);
        game.Hero.X = (ushort)(mapX + 100);
        game.Hero.Y = (ushort)(mapY + 60);

        if (region >= 8)
        {
            game.World.NewRegion = region;
            Regions.LoadRegionNow(game, output);
        }

        Regions.RefreshMinimap(game, output);
        ulong hash = FNV_START;

        for (int x = 0; x < 600; x += 13)
        {
            for (int y = 0; y < 300; y += 7)
            {
                int pixelX = (int)mapX + x;
                int pixelY = (int)mapY + y;

                Mix(ref hash, MapLookup.TerrainAt(game, pixelX, pixelY));
                Mix(ref hash, MapLookup.TileIndex(game, pixelX, pixelY));
                Mix(ref hash, MapLookup.ProbeCollision(game, pixelX, pixelY));
            }
        }

        foreach (short tile in game.Map.Minimap)
        {
            Mix(ref hash, tile);
        }

        Mix(ref hash, game.Hero.Sector);
        Mix(ref hash, game.World.OriginX);
        Mix(ref hash, game.World.OriginY);
        Mix(ref hash, game.World.Region);
        outputCalls = output.Calls.Count;

        return $"{region} {hash:x16} {outputCalls}";
    }

    [Fact]
    public void EveryRegionLooksUpTheSameGroundAsTheRustGame()
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
            string[] expected = File.ReadAllLines(Path.Combine(AppContext.BaseDirectory, "Fixtures", "map_probe_hashes.txt"));

            for (ushort region = 0; region < 10; region++)
            {
                Assert.Equal(expected[region], HashRegion(region, out _));
            }
        }
        finally
        {
            Disk.Close();
            Environment.SetEnvironmentVariable("FTA_DATA", null);
        }
    }
}
