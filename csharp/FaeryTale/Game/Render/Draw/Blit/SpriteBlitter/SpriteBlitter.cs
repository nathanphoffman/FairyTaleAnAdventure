namespace FaeryTale.Game.Render;

/// Sprite drawing routines that drove the original blitter (from the 68000 assembly).
///
/// How a sprite gets onto the screen:
/// 1. SaveBackground copies the background under the sprite's rectangle to a save area.
/// 2. AddSceneryMask copies, for every scenery tile that should hide the sprite (trees, walls ...),
///    the tile's mask into a screen-sized scenery mask.
/// 3. CombineMasks combines the sprite's own mask with that: the sprite is visible only where it
///    is opaque and nothing is in front of it.
/// 4. DrawShape draws the sprite's pixels through that mask into the 5 bitplanes of the screen.
/// 5. Next frame RestoreBackground puts the saved background back, which erases the sprite.
///
/// MakeSpriteMasks builds each frame's own mask once, when the sprite sheet is loaded.
public static partial class SpriteBlitter
{
    /// Bitplanes in the play field.
    private const int PLANE_COUNT = 5;
    private const int SHIFT_MASK = 15;

    /// Bytes in the mask for one scenery tile (32 lines of one word).
    private const int TILE_MASK_BYTES = 64;
    private const int TILE_LINES = 32;

    /// The scenery mask starts this many bytes into the mask region.
    private const int MASK_START = 2;

    /// A mask byte with every pixel set, the starting point for ANDing the planes together.
    private const int ALL_PIXELS_SET = 0xff;

    /// Zeroes the start of the scenery mask, a rectangle sized like a BLTSIZE value.
    public static void ClearMemory(Video video, int size)
    {
        int bytes = Blit.HeightOf((short)size) * Blit.WidthOf((short)size) * 2;
        BlitMemory memory = MemoryOf(video);

        for (int index = 0; index < bytes; index++)
        {
            memory.SetByte(new Loc(Area.Mask, index), 0);
        }
    }

    /// Copies the background under the sprite (all planes) to saveMemory.
    public static void SaveBackground(Video video, Loc saveMemory)
    {
        SpriteRegs regs = SpriteRegs.Of(video);
        BlitMemory memory = MemoryOf(video);

        for (int plane = PLANE_COUNT - 1; plane >= 0; plane--)
        {
            Blit blit = new(Blit.USE_B | Blit.USE_D, Blit.D_EQUALS_B, regs.SShape.BlitSize)
            {
                B = PlaneAtSprite(regs, plane),
                BMod = regs.SShape.Cmod,
                D = saveMemory.Add(regs.SShape.SaveSize * plane),
            };

            BlitEngine.Run(blit, memory);
        }
    }

    /// Puts the saved background back (this erases the sprite).
    public static void RestoreBackground(Video video, Loc saveMemory)
    {
        SpriteRegs regs = SpriteRegs.Of(video);
        BlitMemory memory = MemoryOf(video);

        for (int plane = PLANE_COUNT - 1; plane >= 0; plane--)
        {
            Blit blit = new(Blit.USE_B | Blit.USE_D, Blit.D_EQUALS_B, regs.SShape.BlitSize)
            {
                B = saveMemory.Add(regs.SShape.SaveSize * plane),
                D = PlaneAtSprite(regs, plane),
                DMod = regs.SShape.Cmod,
            };

            BlitEngine.Run(blit, memory);
        }
    }

    /// Draws the sprite into the planes through the mask: D = mask ? shape : screen. The first
    /// and last word masks stop the sprite's shifted edges leaking into neighbouring pixels.
    public static void DrawShape(Video video)
    {
        int bitShift = video.Blitter.SpriteXShift & SHIFT_MASK;

        // The first word is masked so the bits left of the sprite's pixel offset stay clear.
        uint first = Blit.ALL_BITS >> bitShift;
        uint last = ~first & Blit.ALL_BITS;

        if (last == 0)
        {
            last = Blit.ALL_BITS;
        }

        SpriteRegs regs = SpriteRegs.Of(video);
        BlitMemory memory = MemoryOf(video);

        for (int plane = PLANE_COUNT - 1; plane >= 0; plane--)
        {
            DrawShapePlane(regs, memory, plane, bitShift, first, last);
        }
    }

    // The memory a blit can reach.
    private static BlitMemory MemoryOf(Video video)
    {
        (Gfx.Bitmap bitmap, byte[] queue, byte[] mask) = video.Screen.BlitterMemory();

        return new BlitMemory(bitmap, queue, mask, video.Graphics.Shape);
    }

    // Draws one bit plane of the sprite: the mask picks, bit by bit, the sprite's pixel or the
    // screen's. first and last are the masks for the first and last word of each row.
    private static void DrawShapePlane(SpriteRegs regs, BlitMemory memory, int plane, int bitShift, uint first, uint last)
    {
        Loc screen = PlaneAtSprite(regs, plane);

        Blit blit = new(Blit.USE_A | Blit.USE_B | Blit.USE_C | Blit.USE_D, Blit.D_EQUALS_A_THEN_B_ELSE_C, regs.SShape.BlitSize)
        {
            BShift = bitShift,
            FirstWordMask = first & (ushort)regs.WordMask,
            LastWordMask = last,
            A = SceneryMaskAt(regs),
            B = new Loc(Area.Shapes, regs.ShapeData).Add(regs.MaskOffset + (long)regs.PlaneSize * plane),
            BMod = regs.MaskModulo,
            C = screen,
            D = screen,
            CMod = regs.SShape.Cmod,
            DMod = regs.SShape.Cmod,
        };

        BlitEngine.Run(blit, memory);
    }
}
