namespace FaeryTale.Game.Render;

/// SpriteBlitter: the masks that keep sprites behind scenery.
public static partial class SpriteBlitter
{
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

    /// Copies the 32 line mask of scenery tile tile (from the shadow masks) into the scenery mask
    /// at word column x, tile row y; modWords is the mask width in words.
    public static void AddSceneryMask(Video video, int positionX, int positionY, int modWords, int tile)
    {
        int rowBytes = modWords * 2;
        Loc destination = new Loc(Area.Mask, MASK_START).Add((long)positionY * rowBytes * TILE_LINES + positionX * 2);
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
                int all = ALL_PIXELS_SET;

                for (int plane = 0; plane < PLANE_COUNT; plane++)
                {
                    int offset = source + plane * planeBytes + index;

                    all &= offset >= 0 && offset < shapes.Length ? shapes[offset] : 0;
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

    // Where plane p of the play field starts for the current sprite.
    private static Loc PlaneAtSprite(SpriteRegs regs, int plane) => new(Area.Plane(plane), regs.SShape.Coff);

    // Where the scenery mask of the current sprite starts.
    private static Loc SceneryMaskAt(SpriteRegs regs) => new Loc(Area.Mask, MASK_START).Add(regs.ShadowMaskOffset);
}
