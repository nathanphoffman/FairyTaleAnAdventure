using FaeryTale.Game.Data;
using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Remastered;

/// A more forgiving sword for the hero (Remastered and Reimagined). The original only hits what is
/// near a point a few pixels straight ahead, so the hero has to face an enemy almost exactly. Here
/// anything within the weapon's reach of the hero, up to 90 degrees to either side of where he
/// faces, is hit as well, so an enemy beside him is hit without turning.
public static class SwingArc
{
    /// How far to either side of the facing direction a swing reaches, in degrees.
    private const double HALF_ARC_DEGREES = 90;

    /// A step this long finds which way a direction points.
    private const int DIRECTION_STEP = 16;

    private static readonly double MIN_COSINE = Math.Cos(HALF_ARC_DEGREES * Math.PI / 180);

    /// Whether a target is in the swing: within range of the hero and in the arc in front of him.
    public static bool Covers(Shape attacker, Shape target, int facing, int range)
    {
        double toX = target.AbsX - attacker.AbsX;
        double toY = target.AbsY - attacker.AbsY;
        double distance = Math.Sqrt(toX * toX + toY * toY);

        if (distance > range)
        {
            return false;
        }

        double faceX = MapSteps.StepX(attacker.AbsX, facing, DIRECTION_STEP) - attacker.AbsX;
        double faceY = MapSteps.StepY(attacker.AbsY, facing, DIRECTION_STEP) - attacker.AbsY;
        double faceLength = Math.Sqrt(faceX * faceX + faceY * faceY);

        // On top of the hero, or facing nowhere in particular: any way is the front.
        if (distance == 0 || faceLength == 0)
        {
            return true;
        }

        return (toX * faceX + toY * faceY) / (distance * faceLength) >= MIN_COSINE - 1e-9;
    }
}
