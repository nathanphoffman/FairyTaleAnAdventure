using FaeryTale.Game.Creatures;
using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Remastered;

namespace FaeryTale.Tests;

/// Walks an enemy across the real map of the starting village, past a house, with and without
/// route finding. Skipped (passes without checking) when the original game files are not there.
[Collection("Game disk")]
public class EnemyPathingTests
{
    private const int SLOT = 5;
    private const int STEP_SPEED = 2;
    private const int MAX_STEPS = 700;
    private const int CLOSE_ENOUGH = 24;

    // The enemy starts north of a house and the target is south of it: the straight line is blocked.
    private static readonly (int X, int Y) START = (19000, 15740);
    private static readonly (int X, int Y) TARGET = (19000, 15900);

    private static FaeryTale.Game.Data.Game VillageGame(Experience experience)
    {
        FaeryTale.Game.Data.Game game = new() { Experience = experience };

        game.Map.Allocate();
        game.World.MapX = (ushort)(START.X - 144);
        game.World.MapY = (ushort)(START.Y - 90);
        game.World.TileX = (ushort)(game.World.MapX >> 4);
        game.World.TileY = (ushort)(game.World.MapY >> 5);
        game.Hero.X = (ushort)START.X;
        game.Hero.Y = (ushort)START.Y;
        game.World.NewRegion = 3;
        Regions.LoadRegionNow(game, new Recorder());
        Regions.RefreshMinimap(game, new Recorder()); // works out the region origin

        Shape enemy = game.Actors.List[SLOT];

        enemy.Kind = ActorKind.ENEMY;
        enemy.AbsX = (ushort)START.X;
        enemy.AbsY = (ushort)START.Y;
        game.Actors.Active = SLOT + 1;

        return game;
    }

    // Walks the enemy towards the target one step at a time; returns how close it got.
    private static int WalkTowardsTarget(FaeryTale.Game.Data.Game game, bool smart)
    {
        Shape enemy = game.Actors.List[SLOT];
        int closest = int.MaxValue;

        for (int step = 0; step < MAX_STEPS && closest >= CLOSE_ENOUGH; step++)
        {
            (int aimX, int aimY) = smart ? EnemyPathing.AimPoint(game, SLOT, TARGET) : TARGET;
            Steering.Course course = Steering.Steer((short)(enemy.AbsX - aimX), (short)(enemy.AbsY - aimY), 0, () => false);

            if (!course.Stop)
            {
                Blocking.StepFigure(game, SLOT, (short)course.Facing, STEP_SPEED);
            }

            closest = Math.Min(closest, Math.Max(Math.Abs(enemy.AbsX - TARGET.X), Math.Abs(enemy.AbsY - TARGET.Y)));
        }

        return closest;
    }

    [Fact]
    public void AnEnemyWithARouteGetsRoundAHouseThatStopsAStraightWalker()
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

            int straight = WalkTowardsTarget(VillageGame(Experience.Remastered), false);

            EnemyPathing.Reset();

            int routed = WalkTowardsTarget(VillageGame(Experience.Remastered), true);

            Assert.True(straight >= CLOSE_ENOUGH, $"the straight walker should be stopped by the house, got within {straight}");
            Assert.True(routed < CLOSE_ENOUGH, $"the enemy with a route should get there, got within {routed}");
        }
        finally
        {
            Disk.Close();
            Environment.SetEnvironmentVariable("FTA_DATA", null);
        }
    }

    [Fact]
    public void OriginalModeAimsStraightAtTheTarget()
    {
        FaeryTale.Game.Data.Game game = new() { Experience = Experience.Original };

        Assert.Equal(TARGET, EnemyPathing.AimPoint(game, SLOT, TARGET));
    }
}
