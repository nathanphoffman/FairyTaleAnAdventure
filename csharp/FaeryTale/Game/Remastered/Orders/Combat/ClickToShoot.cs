using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;

namespace FaeryTale.Game.Remastered;

/// How the hero uses a bow or a wand on an enemy he was told to attack (Remastered and Reimagined).
/// Shots fly straight along one of eight directions, so he only fires when the enemy is on one of the
/// four lines through him (across, up and down, or the two slants); until then he steps to get on a
/// line, and he backs away from an enemy that has come too close. The old game just turned to the
/// nearest direction and shot, which missed unless the enemy happened to be lined up.
public static class ClickToShoot
{
    /// What to do this frame: a direction (0 to 7, clockwise from north-west) and whether to shoot
    /// that way or to walk that way.
    public readonly record struct Plan(short Direction, bool Fire);

    // An enemy this close to the line of fire is hit (an arrow hits within a few pixels).
    private const int LINE_TOLERANCE = 4;

    // The hero likes to keep at least this far from the enemy, and shoots up to this far away.
    private const int KEEP_AWAY = 60;
    public const int SHOOTING_DISTANCE = 240;

    // How far one frame of walking takes the hero, and how much worse being too close is than being
    // off the line (a pixel too close counts this fraction of a pixel off the line). It is small so
    // that getting on a line wins: an enemy that is chasing the hero would otherwise make him back away
    // for ever and never fire.
    private const int STEP = 3;
    private const double CROWDING_WEIGHT = 0.3;

    // Each walk direction as steps across and down, clockwise from north-west.
    private static readonly (int X, int Y)[] DIRECTIONS = [(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];

    // Where the enemy lies as a direction (see DIRECTIONS) for each line: across, up and down, and the slants.
    private const short EAST = 3;
    private const short WEST = 7;
    private const short SOUTH = 5;
    private const short NORTH = 1;

    /// The plan for a hero who has an enemy in sight, or null when there is nothing better than the old
    /// way (the enemy is too far, or no step gets him any closer to a line).
    public static Plan? PlanFor(Data.Game game, Shape hero, Shape enemy, bool clearShot)
    {
        int dx = enemy.AbsX - hero.AbsX;
        int dy = enemy.AbsY - hero.AbsY;

        if (Math.Max(Math.Abs(dx), Math.Abs(dy)) > SHOOTING_DISTANCE)
        {
            return null;
        }

        (double offset, short direction) = NearestLine(dx, dy);

        if (offset <= LINE_TOLERANCE && clearShot)
        {
            return new Plan(direction, true);
        }

        return BestStep(game, hero, dx, dy) is { } step ? new Plan(step, false) : null;
    }

    // How far the enemy is off the closest of the four lines through the hero, and the direction that
    // line points to the enemy in. A line across is off by the up and down gap, and so on; a slant is
    // off by the gap between the two gaps over the square root of 2 (the pixels between two slants).
    internal static (double Offset, short Direction) NearestLine(int dx, int dy)
    {
        (double Offset, short Direction)[] lines =
        [
            (Math.Abs(dy), dx >= 0 ? EAST : WEST),
            (Math.Abs(dx), dy >= 0 ? SOUTH : NORTH),
            (Math.Abs(dx - dy) / Math.Sqrt(2), dx >= 0 ? (short)4 : (short)0),
            (Math.Abs(dx + dy) / Math.Sqrt(2), dx >= 0 ? (short)2 : (short)6),
        ];

        return lines.MinBy(line => line.Offset);
    }

    // The walk direction that most improves how lined up the hero is (counting being too close as bad),
    // among the steps he can take; null when none is better than standing.
    private static short? BestStep(Data.Game game, Shape hero, int dx, int dy)
    {
        double best = Cost(dx, dy);
        short? bestDirection = null;

        for (short direction = 0; direction < DIRECTIONS.Length; direction++)
        {
            (int stepX, int stepY) = DIRECTIONS[direction];
            double cost = Cost(dx - stepX * STEP, dy - stepY * STEP);

            if (cost < best && Blocking.BlockedAt(game, (short)(hero.AbsX + stepX * STEP), (short)(hero.AbsY + stepY * STEP), 0) == 0)
            {
                best = cost;
                bestDirection = direction;
            }
        }

        return bestDirection;
    }

    // How bad a place is: off the line of fire, plus extra when the enemy is nearer than KEEP_AWAY.
    private static double Cost(int dx, int dy)
    {
        double near = Math.Max(0, KEEP_AWAY - Math.Max(Math.Abs(dx), Math.Abs(dy)));

        return NearestLine(dx, dy).Offset + near * CROWDING_WEIGHT;
    }
}
