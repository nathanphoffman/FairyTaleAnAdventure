using FaeryTale.Game.Input;
using FaeryTale.Game.Land;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Platform;

/// NativeComposer: what is drawn over the world.
internal sealed partial class NativeComposer
{
    // What an item is, in a box over its name in the message log, while the pointer rests on it.
    private void DrawItemTip(int width, Video video)
    {
        if (video.Capture.Game is { } game)
        {
            MessageItemTip.Of(game, video.History, video.MessageAdvance, InputHandler.GridPointer)
                .DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
        }
    }

    // The pointer: a door or a face when over one, else the game's own.
    private void DrawPointer(int width, Scene scene, Overlay overlay, Video video, (int X, int Y) pointer)
    {
        WorldCapture capture = video.Capture;
        Image? face = capture.TalkingTo is null && capture.HoverPerson is { } person ? FaceOf(video, person) : null;

        if (overlay.HidePointer)
        {
            return;
        }

        if (capture.LookMode)
        {
            new Overlay { Images = { EyeCursor.At((0, 0)) } }.DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else if (capture.EnemyHover)
        {
            new Overlay { Images = { Crosshair.At((0, 0), POINTER_CROSSHAIR_RADIUS) } }.DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else if (capture.ItemHover && capture.TalkingTo is null)
        {
            new Overlay { Images = { HandCursor.At((0, 0)) } }.DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else if (face is not null)
        {
            new Overlay { Images = { face with { X = -face.Width / 2, Y = -face.Height / 2 } } }
                .DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else if (capture.DoorHover)
        {
            new Overlay { Images = { DoorCursor.At((0, 0)) } }.DrawAt(picture, width, new Overlay.Placement(pointer.X, pointer.Y, Scale));
        }
        else
        {
            scene.DrawPointerAtPixel(picture, width, pointer, POINTER_PIXELS_PER_GAME_PIXEL * Scale);
        }
    }

    // The red crosshair over the enemy being attacked.
    private void DrawAttackTarget(int width, WorldView view, WorldCapture capture)
    {
        if (capture.AttackTarget is not { } target)
        {
            return;
        }

        (double x, double y) = view.ToScreen(target.X, target.Y);

        new Overlay { Images = { Crosshair.At(((int)Math.Round(x), (int)Math.Round(y)), TARGET_RADIUS_WORLD_PIXELS * view.Scale) } }.Draw(picture, width);
    }

    // A bar over each enemy that was hurt lately: black edge, dark red where health is gone, green where it is left.
    private void DrawHealthBars(int width, WorldView view, WorldCapture capture)
    {
        foreach (EnemyHealthBars.Bar bar in capture.HealthBars)
        {
            (double x, double y) = view.ToScreen(bar.X, bar.Y);
            int barWidth = HEALTH_BAR_WIDTH * view.Scale;
            int barHeight = HEALTH_BAR_HEIGHT * view.Scale;
            int left = (int)Math.Round(x) - barWidth / 2;
            int top = (int)Math.Round(y);
            int green = (int)Math.Round(bar.Fraction * barWidth);

            Overlay overlay = new();

            overlay.Images.Add(new Image(left - view.Scale, top - view.Scale, barWidth + 2 * view.Scale, barHeight + 2 * view.Scale, Filled((barWidth + 2 * view.Scale) * (barHeight + 2 * view.Scale), HEALTH_BAR_EDGE)));
            overlay.Images.Add(new Image(left, top, barWidth, barHeight, Filled(barWidth * barHeight, HEALTH_BAR_GONE)));

            if (green > 0)
            {
                overlay.Images.Add(new Image(left, top, green, barHeight, Filled(green * barHeight, HEALTH_BAR_LEFT)));
            }

            overlay.Draw(picture, width);
        }
    }

    // The small map in the top right corner of the world (outdoors only).
    private void DrawMinimap(int width, WorldView view, WorldCapture capture)
    {
        if (!MapSettings.Enabled || capture.Game is not { } game || game.World.Region >= Regions.FIRST_INDOOR_REGION)
        {
            return;
        }

        int unit = Math.Max(1, (int)Math.Round(view.Height / UNIT_HEIGHT));
        (int column, int row) = WorldOverview.TileOf(capture.HeroNow.X, capture.HeroNow.Y);
        Image map = MinimapPicture.Of(WorldOverview.Pixels, (column, row));

        new Overlay { Images = { map } }.DrawAt(picture, width, new Overlay.Placement(width - (MinimapPicture.WIDTH + MINIMAP_MARGIN) * unit, MINIMAP_MARGIN * unit, unit));
    }

    // The circle where the player clicked, over the ground it is on.
    private void DrawMarker(int width, WorldView view, WorldCapture capture)
    {
        if (capture.Marker is not { } marker)
        {
            return;
        }

        (double x, double y) = view.ToScreen(marker.World.X, marker.World.Y);

        new Overlay { Images = { ClickRing.PictureAtPixel(((int)Math.Round(x), (int)Math.Round(y)), marker.Walkable, view.Scale) } }
            .Draw(picture, width);
    }
}
