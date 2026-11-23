using FaeryTale.Game.Data;

namespace FaeryTale.Game.Render;

/// SpriteDrawing: setting up and queuing the blit of one sprite.
public static partial class SpriteDrawing
{
    // Saves the background under the sprite and draws it through its masks into the page.
    private static Drawn BlitSprite(Data.Game game, Video video, Frame frame, Shape actor, int slot, Placed placed)
    {
        BlitExtent extent = BlitExtentOf(placed.Clip);

        if (!SetUpBlit(video, frame, placed, extent))
        {
            return Drawn.OutOfRoom; // no more blits if no room
        }

        SpriteBlitter.SaveBackground(video, video.Blitter.SShape.BackSave);

        if (!SceneryMasks.DrawnInFront(game.World, game.Hero, actor, slot, placed.Sprite))
        {
            SceneryMasks.MaskAgainstScenery(game, video, actor, slot, placed, extent);
        }

        SpriteBlitter.CombineMasks(video);
        SpriteBlitter.DrawShape(video);
        actor.Visible = 1;
        QueueShape(video.Screen, video.Blitter.SShape);

        return Drawn.Done;
    }

    // Notes how the sprite just drawn was drawn, so it can be undone next frame.
    private static void QueueShape(Screen screen, SShape shape)
    {
        FPage page = screen.DrawingPage;
        int index = page.ObCount;

        if (index < page.ShapeQueue.Count)
        {
            page.ShapeQueue[index] = shape;
        }
        else
        {
            page.ShapeQueue.Add(shape);
        }

        page.ObCount++;
    }

    // Fills in the blitter's registers and the queue entry for this sprite: sizes, where the image,
    // its mask and the saved background are, and the shifts. Returns false when the save buffer is full.
    private static bool SetUpBlit(Video video, Frame frame, Placed placed, BlitExtent extent)
    {
        // The sheet is looked up again: a sunk actor is drawn from the objects sheet, but is as
        // wide and high as its own.
        SeqInfo seq = SpriteImages.SpriteSheet(video, placed.Sprite.Atype);

        video.Blitter.SShape = new SShape { BackSave = Loc.NONE };
        video.Screen.PlaneSize = seq.Bytes;
        video.Blitter.SShape.SaveSize = (short)(extent.BlitWide * extent.BlitHigh * WORD_BYTES);

        // Blitter size word: height in the high bits, width in words in the low 6 bits.
        video.Blitter.SShape.BlitSize = (short)((extent.BlitHigh << BLIT_HEIGHT_SHIFT) + extent.BlitWide);

        LocateSpriteData(video, seq, placed.Sprite.Inum);

        if (!AssignBackgroundSave(video, frame))
        {
            return false;
        }

        SetBlitterRegisters(video.Blitter, placed, extent);

        return true;
    }

    // Points the blitter at the picture and the mask of image inum in its sprite sheet.
    private static void LocateSpriteData(Video video, SeqInfo seq, short inum)
    {
        int planeSize = video.Screen.PlaneSize;

        video.Graphics.ShapeData = seq.Location + planeSize * PLANES * inum;
        video.Graphics.MaskBuffer = seq.MaskLoc + planeSize * inum;
    }

    // Reserves room to save the background under the sprite: a spare bit plane while they last
    // (for small sprites), otherwise the page's save buffer. Returns false when the buffer is full.
    private static bool AssignBackgroundSave(Video video, Frame frame)
    {
        FPage page = video.Screen.DrawingPage;
        BlitterRegisters blitter = video.Blitter;

        if (blitter.SShape.SaveSize < SMALL_SAVE && frame.Crack < SPARE_SAVES)
        {
            blitter.SShape.BackSave = new Loc(Area.Plane(frame.Crack), SPARE_PLANE_OFFSET);
            frame.Crack++;

            return true;
        }

        blitter.SShape.BackSave = frame.BackAlloc;
        int bytes = PLANES * blitter.SShape.SaveSize;
        frame.BackAlloc = frame.BackAlloc.Add(bytes);
        page.SaveUsed += bytes;

        return page.SaveUsed < SAVE_LIMIT;
    }

    // Sets the blitter's mask offsets, shift and modulo for the clipped sprite, and where in the
    // page it goes.
    private static void SetBlitterRegisters(BlitterRegisters blitter, Placed placed, BlitExtent extent)
    {
        Sprite sprite = placed.Sprite;
        Clip clip = placed.Clip;
        int wide = placed.BytesWide;

        blitter.ShadowMaskOffset = unchecked((short)((clip.YStart & SHADOW_LINES_MASK) * extent.BlitWide * WORD_BYTES));

        // Where the visible part starts in the mask: whole words (2 bytes per 16 pixels) in, and whole rows down.
        blitter.SpriteMaskOffset = unchecked((short)((clip.XOff >> WORD_PIXELS_SHIFT) * WORD_BYTES + wide * clip.YOff));
        blitter.SShape.Coff = unchecked((short)(extent.Xbw * WORD_BYTES + clip.YStart * PAGE_ROW_BYTES));
        blitter.SpriteXShift = (short)(sprite.XStart & WORD_PIXELS_MASK);

        if ((clip.XOff & WORD_PIXELS_MASK) != 0)
        {
            blitter.SShape.Coff -= WORD_BYTES;
            blitter.LastWordMask = 0;
        }
        else
        {
            blitter.LastWordMask = -1;
        }

        blitter.SpriteMaskModulo = unchecked((short)(wide - extent.BlitWide * WORD_BYTES));
        blitter.SShape.Cmod = unchecked((short)(PAGE_ROW_BYTES - extent.BlitWide * WORD_BYTES));
    }
}
