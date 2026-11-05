using FaeryTale.Game.Data;

namespace FaeryTale.Game.Land.Map;

/// The camera: where the map is shown so the hero stays near the middle, and when the page turns.
public static class MapCamera
{
    // The hero may wander this far from the camera centre before it moves.
    private const int CAMERA_CENTER_X = 144;
    private const int CAMERA_CENTER_Y = 70;
    private const int CAMERA_JUMP_X = 70;
    private const int CAMERA_CREEP_X = 20;
    private const int CAMERA_JUMP_UP = 24;
    private const int CAMERA_JUMP_DOWN = 44;
    private const int CAMERA_CREEP_Y = 10;

    // Distances from the fold where the page turn inset changes, and the inset from there on.
    private const int FAR_EDGE = 137;
    private const int FAR_INSET = 10;
    private const int LAST_STEP = 136;

    private static readonly int[] NEAR_FOLD = [9, 9, 8, 7, 6, 5, 5, 5, 4, 4, 4];

    /// Sign-extends a 15 bit difference to 16 bits.
    internal static short SignExtend15(short value) =>
        // Shifting left then arithmetically right copies bit 14 into bit 15, making a 15 bit signed number.
        (short)((short)(value << 1) >> 1);

    /// Where the camera goes for a hero position: it creeps along once the hero is 20 pixels (10
    /// vertically) off centre, and jumps once the hero is 70 (24 up, 44 down) off. The hero is
    /// kept at about (144, 70) on screen.
    internal static (ushort X, ushort Y) CameraAfter(ushort cameraX, ushort cameraY, int x, int y)
    {
        short cx = (short)(x - CAMERA_CENTER_X);
        short cy = (short)(y - CAMERA_CENTER_Y);
        short dx = SignExtend15((short)(cx - cameraX));
        short dy = SignExtend15((short)(cy - cameraY));

        return (NewCameraX(cameraX, cx, dx), NewCameraY(cameraY, cy, dy));
    }

    private static ushort NewCameraX(ushort cameraX, short target, short delta)
    {
        if (delta < -CAMERA_JUMP_X)
        {
            return (ushort)(target + CAMERA_JUMP_X);
        }

        if (delta > CAMERA_JUMP_X)
        {
            return (ushort)(target - CAMERA_JUMP_X);
        }

        if (delta < -CAMERA_CREEP_X)
        {
            return (ushort)(cameraX - 1);
        }

        return delta > CAMERA_CREEP_X ? (ushort)(cameraX + 1) : cameraX;
    }

    private static ushort NewCameraY(ushort cameraY, short target, short delta)
    {
        if (delta < -CAMERA_JUMP_UP)
        {
            return (ushort)(target + CAMERA_JUMP_UP);
        }

        if (delta > CAMERA_JUMP_DOWN)
        {
            return (ushort)(target - CAMERA_JUMP_DOWN);
        }

        if (delta < -CAMERA_CREEP_Y)
        {
            return (ushort)(cameraY - 1);
        }

        return delta > CAMERA_CREEP_Y ? (ushort)(cameraY + 1) : cameraY;
    }

    /// Moves the camera (MapX, MapY) after the hero.
    public static void MoveCameraTo(World world, int x, int y) =>
        (world.MapX, world.MapY) = CameraAfter(world.MapX, world.MapY, x, y);

    /// Small lookup used by the page-turn animation: how far in from the top and bottom a strip of
    /// the curling page starts, given its distance from the fold.
    public static int PageTurnInset(int x) => x switch
    {
        <= 10 => NEAR_FOLD[x],
        >= FAR_EDGE => FAR_INSET,
        LAST_STEP => 7,
        >= 124 and <= 135 => 6,
        >= 99 and <= 123 => 5,
        >= 72 and <= 98 => 4,
        _ => 3,
    };
}
