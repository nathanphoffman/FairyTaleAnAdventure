using FaeryTale.Game.Creatures.Combat;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

namespace FaeryTale.Tests;

public class MissileRangeTests
{
    private const int START = 1000;
    private const int FIREBALL_SPEED = 5;
    private const int EAST = 3;

    // How far a hero's fireball flies east over open ground before it gives up.
    private static int DistanceFlown(Experience experience)
    {
        FaeryTale.Game.Data.Game game = new() { Experience = experience };
        Missile missile = game.Actors.Missiles[0];

        missile.AbsX = START;
        missile.AbsY = START;
        missile.MissileType = Missiles.MISSILE_ROCK;
        missile.Speed = FIREBALL_SPEED;
        missile.Direction = EAST;
        missile.Archer = 0;

        for (int frame = 0; frame < 200 && missile.MissileType != Missiles.MISSILE_NONE; frame++)
        {
            Missiles.AdvanceMissiles(game, new Recorder());
        }

        return missile.AbsX - START;
    }

    [Fact]
    public void TheHerosFireballFliesTwiceAsFarOutsideOriginal()
    {
        int original = DistanceFlown(Experience.Original);
        int remastered = DistanceFlown(Experience.Remastered);

        Assert.True(original > 0);
        Assert.True(remastered >= original * 19 / 10, $"original {original}, remastered {remastered}");
    }
}
