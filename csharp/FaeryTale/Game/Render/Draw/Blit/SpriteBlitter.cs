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
public static class SpriteBlitter
{
    /// Bitplanes in the play field.
    private const int PLANE_COUNT = 5;
    private const int SHIFT_MASK = 15;

    /// Bytes in the mask for one scenery tile (32 lines of one word).
    private const int TILE_MASK_BYTES = 64;
    private const int TILE_LINES = 32;

    /// The scenery mask starts this many bytes into the mask region.
    private const int MASK_START = 2;

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

    /// Combines the sprite's own mask with the scenery mask built by AddSceneryMask. The result
    /// goes in the scenery mask: the sprite is opaque here and nothing is in front of it (D = A and not C).
    public static void CombineMasks(Video video)
    {
        SpriteRegs regs = SpriteRegs.Of(video);
        Loc sceneryMask = SceneryMaskAt(regs);

        Blit blit = new(Blit.USE_A | Blit.USE_C | Blit.USE_D, Blit.D_EQUALS_A_AND_NOT_C, regs.SShape.BlitSize)
        {
            A = new Loc(Area.Shapes, video.Graphics.MaskBuffer).Add(regs.MaskOffset),
            AMod = regs.MaskModulo,
            AShift = video.Blitter.SpriteXShift & SHIFT_MASK,
            C = sceneryMask,
            D = sceneryMask,
        };

        BlitEngine.Run(blit, MemoryOf(video));
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

    /// Copies the 32 line mask of scenery tile tile (from the shadow masks) into the scenery mask
    /// at word column x, tile row y; modWords is the mask width in words.
    public static void AddSceneryMask(Video video, int x, int y, int modWords, int tile)
    {
        int rowBytes = modWords * 2;
        Loc destination = new Loc(Area.Mask, MASK_START).Add((long)y * rowBytes * TILE_LINES + x * 2);
        byte[] shadow = video.Graphics.Shadow;
        int source = tile * TILE_MASK_BYTES;
        BlitMemory memory = MemoryOf(video);

        for (int line = 0; line < TILE_LINES; line++)
        {
            memory.SetByte(destination, shadow[source]);
            memory.SetByte(destination.Add(1), shadow[source + 1]);
            source += 2;
            destination = destination.Add(rowBytes);
        }
    }

    /// For every sprite frame builds a one plane mask: all 5 planes ANDed then inverted, so colour
    /// 31 (all bits set) is transparent. The frames start at source in the sprite memory and the
    /// masks are stored from destination on.
    public static void MakeSpriteMasks(Graphics graphics, int source, int destination, int words, int scans, int chars)
    {
        int planeBytes = (ushort)(words * scans) * 2;
        byte[] shapes = graphics.Shape;

        for (int frame = 0; frame < chars; frame++)
        {
            for (int index = 0; index < planeBytes; index++)
            {
                // A pixel is kept only if every plane has it set (AND of all planes), 0xff when no planes.
                int all = 0xff;

                for (int plane = 0; plane < PLANE_COUNT; plane++)
                {
                    int at = source + plane * planeBytes + index;

                    all &= at >= 0 && at < shapes.Length ? shapes[at] : 0;
                }

                if (destination >= 0 && destination < shapes.Length)
                {
                    shapes[destination] = (byte)~all;
                }

                destination++;
            }

            source += PLANE_COUNT * planeBytes;
        }
    }

    // The blitter settings of the sprite being drawn, copied out of the video.
    private readonly record struct SpriteRegs(SShape SShape, short ShadowMaskOffset, short MaskOffset, short MaskModulo, short WordMask, int PlaneSize, int ShapeData)
    {
        public static SpriteRegs Of(Video video) => new(
            video.Blitter.SShape,
            video.Blitter.ShadowMaskOffset,
            video.Blitter.SpriteMaskOffset,
            video.Blitter.SpriteMaskModulo,
            video.Blitter.LastWordMask,
            video.Screen.PlaneSize,
            video.Graphics.ShapeData);
    }

    // The memory a blit can reach.
    private static BlitMemory MemoryOf(Video video)
    {
        (Gfx.Bitmap bitmap, byte[] queue, byte[] mask) = video.Screen.BlitterMemory();

        return new BlitMemory(bitmap, queue, mask, video.Graphics.Shape);
    }

    // Where plane p of the play field starts for the current sprite.
    private static Loc PlaneAtSprite(SpriteRegs regs, int plane) => new(Area.Plane(plane), regs.SShape.Coff);

    // Where the scenery mask of the current sprite starts.
    private static Loc SceneryMaskAt(SpriteRegs regs) => new Loc(Area.Mask, MASK_START).Add(regs.ShadowMaskOffset);

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
