using FaeryTale.Game.Data;

namespace FaeryTale.Game.Render;

/// Drawing the actors each frame: every actor and its weapon is drawn into the page, clipping to
/// the screen, hiding parts behind scenery, and saving the background under each sprite so it can
/// be put back next frame.
public static partial class SpriteDrawing
{
    /// How one actor's drawing ended.
    private enum Drawn
    {
        /// Drawn, or skipped because it is off screen.
        Done,

        /// The save buffer is full: no more actors this frame.
        OutOfRoom,
    }

    /// Screen size in pixels the sprites are clipped to (the play field is 320 by 174).
    internal const short SCREEN_RIGHT = 319;
    internal const short SCREEN_BOTTOM = 173;

    // Bytes in a page row, and the first row of the spare bit planes used as quick background saves.
    private const int ROW_BYTES = 40;
    private const int SPARE_PLANE_OFFSET = 192 * ROW_BYTES;
    private const byte SPARE_SAVES = 5;

    /// A saved background smaller than this many bytes (per plane) goes in a spare plane.
    private const short SMALL_SAVE = 64;
    private const int PLANES = 5;

    /// The background-save buffer holds this many bytes; running out stops drawing for the frame.
    private const int SAVE_LIMIT = 74 * 80;

    // Bit 7 of an object's image number means it is drawn from the middle of its picture (8 lines in).
    private const short FROM_MIDDLE_BIT = 128;
    private const sbyte MIDDLE_START = 8;
    private const short IMAGE_NUMBER_MASK = 0x7f;

    /// The blitter's size word has the height in the bits above the 6 bit width.
    private const int BLIT_HEIGHT_SHIFT = 6;

    // A sprite is 16 pixels to a word (2 bytes); the shadow mask repeats every 32 lines; the page
    // is 40 bytes wide.
    private const int WORD_PIXELS_MASK = 15;
    private const int WORD_PIXELS_SHIFT = 4;
    private const int WORD_BYTES = 2;
    private const int SHADOW_LINES_MASK = 31;
    private const int PAGE_ROW_BYTES = 40;
    private const sbyte PIXELS_PER_BYTE = 8;

    // The map scrolls in whole tiles (16 x 32 pixels), so the part of a tile it is offset by is added.
    private const int TILE_X_MASK = 15;
    private const int TILE_Y_MASK = 31;

    /// Draws every actor, and its weapon, into the page being drawn, and saves the background
    /// under each (BackSave) so it can be undone next frame. cycle is the frame counter.
    public static void DrawSprites(Data.Game game, Video video, short cycle)
    {
        FPage page = video.Screen.DrawingPage;

        page.ObCount = 0;
        page.SaveUsed = 0;
        video.Capture.BeginFrame(game);

        Frame frame = new() { Crack = 0, BackAlloc = page.BackSave };

        for (int j = 0; j < game.Actors.Drawn; j++)
        {
            int slot = game.Actors.DrawOrder[j];
            Shape actor = game.Actors.List[slot];

            actor.Visible = 0;
            byte pass = 0;

            do
            {
                bool passmode = SpriteImages.WeaponPassmode(actor, game.World, ref pass);

                if (DrawOne(game, video, frame, actor, slot, cycle, passmode) == Drawn.OutOfRoom)
                {
                    return;
                }
            }

            while (pass == 1);
        }
    }

    /// Cuts the sprite to the screen. Also notes in the image number whether the sprite is drawn
    /// from the middle of its picture (bit 7).
    internal static Clip ClipToScreen(Sprite sprite)
    {
        short xStart = Math.Max(sprite.XStart, (short)0);
        short yStart = Math.Max(sprite.YStart, (short)0);
        Clip clip = new()
        {
            XStart = xStart,
            YStart = yStart,
            XStop = Math.Min(sprite.XStop, SCREEN_RIGHT),
            YStop = Math.Min(sprite.YStop, SCREEN_BOTTOM),
            XOff = unchecked((sbyte)(xStart - sprite.XStart)),
            YOff = unchecked((sbyte)(yStart - sprite.YStart)),
        };

        if (sprite.Atype == SpriteImages.OBJECTS && (sprite.Inum & FROM_MIDDLE_BIT) != 0)
        {
            sprite.Inum &= IMAGE_NUMBER_MASK;
            clip.YOff = unchecked((sbyte)(clip.YOff + MIDDLE_START));
        }

        return clip;
    }

    // Draws one actor (or its weapon) for the current pass.
    private static Drawn DrawOne(Data.Game game, Video video, Frame frame, Shape actor, int slot, short cycle, bool passmode)
    {
        SceneryMasks.ClearSceneryMask(video);

        if (SpriteImages.ChooseImage(game.Actors, game.Hero, actor, slot, passmode) is not { } sprite)
        {
            return Drawn.Done;
        }

        sbyte bytesWide = PlaceOnScreen(game.World, video, sprite);

        if (!SpriteImages.AdjustForDepth(game.Hero, actor, slot, cycle, passmode, sprite))
        {
            return Drawn.Done;
        }

        // The new world drawer of Remastered shows more than the old screen does, so it is told
        // about sprites that are off the old screen too.
        video.Capture.Record(game, video, actor, slot, passmode, sprite);

        if (IsOffScreen(sprite))
        {
            return Drawn.Done;
        }

        Clip clip = ClipToScreen(sprite);

        return BlitSprite(game, video, frame, actor, slot, new Placed(sprite, clip, bytesWide));
    }

    // Moves the sprite from map to screen coordinates and works out its size and bottom right
    // corner. Returns the width of the image in bytes.
    private static sbyte PlaceOnScreen(World world, Video video, Sprite sprite)
    {
        sprite.XStart += (short)(world.MapX & TILE_X_MASK);
        sprite.YStart += (short)(world.MapY & TILE_Y_MASK);
        sprite.Ground += (short)(world.MapY & TILE_Y_MASK);

        SeqInfo seq = SpriteImages.SpriteSheet(video, sprite.Atype);

        // Sprite width in pixels: 16 pixels to each word.
        sbyte xSize = unchecked((sbyte)(seq.Width << WORD_PIXELS_SHIFT));
        sbyte ySize = SpriteImages.ImageHeight(sprite.Atype, sprite.Inum, unchecked((sbyte)seq.Height));

        sprite.XStop = unchecked((short)(sprite.XStart + xSize - 1));
        sprite.YStop = unchecked((short)(sprite.YStart + ySize - 1));

        return (sbyte)(xSize / PIXELS_PER_BYTE);
    }

    // Whether the sprite is completely outside the screen.
    private static bool IsOffScreen(Sprite sprite) =>
        sprite.XStart > SCREEN_RIGHT || sprite.YStart > SCREEN_BOTTOM || sprite.XStop < 0 || sprite.YStop < 0;

    // Works out the extent of the clipped sprite.
    private static BlitExtent BlitExtentOf(Clip clip)
    {
        // Pixel columns to word columns (16 pixels to a word).
        sbyte xbw = unchecked((sbyte)(clip.XStart >> WORD_PIXELS_SHIFT));
        sbyte xew = unchecked((sbyte)(clip.XStop >> WORD_PIXELS_SHIFT));  // shift right = divide pixels by 16 per word
        sbyte wide = unchecked((sbyte)(xew - xbw + 1));
        sbyte high = unchecked((sbyte)(clip.YStop - clip.YStart + 1));

        if ((clip.XOff & WORD_PIXELS_MASK) != 0)
        {
            wide = unchecked((sbyte)(wide + 1));
        }

        return new BlitExtent(xbw, wide, high);
    }
}
