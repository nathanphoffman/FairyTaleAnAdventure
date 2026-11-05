using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Tests;

public class BlitTests
{
    // The bit-at-a-time definition of a minterm, as the C original computed it.
    private static uint MintermByBits(uint minterm, uint a, uint b, uint c)
    {
        uint result = 0;

        for (int bit = 0; bit < 16; bit++)
        {
            // The A, B and C bits of this position make the minterm's bit number (A * 4 + B * 2 + C).
            uint index = (((a >> bit) & 1) << 2) | (((b >> bit) & 1) << 1) | ((c >> bit) & 1);

            result |= ((minterm >> (int)index) & 1) << bit;
        }

        return result;
    }

    [Fact]
    public void MintermWordsMatchTheBitByBitVersion()
    {
        uint[] samples = [0x0000, 0xffff, 0x1234, 0xa5c3, 0x0ff0, 0x8001];

        foreach (uint minterm in new[] { Blit.D_EQUALS_B, Blit.D_EQUALS_A_AND_NOT_C, Blit.D_EQUALS_A_THEN_B_ELSE_C })
        {
            foreach (uint a in samples)
            {
                foreach (uint b in samples)
                {
                    foreach (uint c in samples)
                    {
                        Assert.Equal(MintermByBits(minterm, a, b, c), BlitEngine.MintermWord(minterm, a, b, c));
                    }
                }
            }
        }
    }

    [Fact]
    public void BlitSizeFields()
    {
        Assert.Equal(32, Blit.HeightOf(0x0802));
        Assert.Equal(2, Blit.WidthOf(0x0802));
        Assert.Equal(64, Blit.WidthOf(0x0800));
    }

    // Runs job on work memory holding shapes and an empty queue of 8 bytes; returns the queue.
    private static byte[] Run(byte[] shapes, Action<BlitMemory> job)
    {
        byte[] queue = new byte[8];

        job(new BlitMemory(Bitmap.Empty(), queue, new byte[8], (byte[])shapes.Clone()));

        return queue;
    }

    [Fact]
    public void ACopyBlitCopiesWords()
    {
        byte[] source = [1, 2, 3, 4, 5, 6, 7, 8];

        byte[] result = Run(source, memory =>
        {
            Blit blit = new(Blit.USE_B | Blit.USE_D, Blit.D_EQUALS_B, 2 << 6 | 2) { B = new Loc(Area.Shapes, 0), D = new Loc(Area.Queue, 0) };

            BlitEngine.Run(blit, memory);
        });

        Assert.Equal(source, result);
    }

    [Fact]
    public void AShiftedCopyMovesBitsRight()
    {
        byte[] result = Run([0xff, 0x00], memory =>
        {
            Blit blit = new(Blit.USE_B | Blit.USE_D, Blit.D_EQUALS_B, 1 << 6 | 1) { B = new Loc(Area.Shapes, 0), D = new Loc(Area.Queue, 0), BShift = 4 };

            BlitEngine.Run(blit, memory);
        });

        Assert.Equal(new byte[] { 0x0f, 0xf0 }, result[..2]);
    }

    [Fact]
    public void ReadsOutsideAnAreaAreZeroWithWritesDropped()
    {
        byte[] result = Run([9, 9], memory =>
        {
            Blit blit = new(Blit.USE_B | Blit.USE_D, Blit.D_EQUALS_B, 1 << 6 | 1) { B = new Loc(Area.Shapes, -1), D = new Loc(Area.Queue, 7) };

            BlitEngine.Run(blit, memory);
        });

        Assert.Equal(0, result[7]);
    }

    [Fact]
    public void ScreenHoldsThePortsAndFlipsBitmapsByName()
    {
        Screen screen = new();
        screen.SetBitmap(BitmapId.Page1, new Bitmap(320, 200, 5));
        screen.SetBitmap(BitmapId.Page2, new Bitmap(320, 200, 5));
        screen.SetBitmap(BitmapId.Text, new Bitmap(640, 57, 4));
        screen.GetPort(PortId.Text2).Target = BitmapId.Text;
        screen.GetPort(PortId.Text).Target = BitmapId.TextPlane0;

        screen.SetAPen(PortId.Text, 0b1111);
        screen.MoveTo(PortId.Text, 0, 8);
        screen.Text(PortId.Text, "A"u8);

        // The message port draws only on the first plane, so every pixel is 0 or 1.
        Assert.Contains(Enumerable.Range(0, 8), y => Enumerable.Range(0, 8).Any(x => screen.GetBitmap(BitmapId.Text).Pixel(x, y) == 1));
        Assert.All(Enumerable.Range(0, 8), y => Assert.All(Enumerable.Range(0, 8), x => Assert.InRange(screen.GetBitmap(BitmapId.Text).Pixel(x, y), 0, 1)));

        screen.SetAPen(PortId.Text2, 0b1010);
        screen.RectFill(PortId.Text2, (0, 20), (3, 23));
        Assert.Equal(0b1010, screen.GetBitmap(BitmapId.Text).Pixel(1, 21));

        screen.GetBitmap(BitmapId.Page2).Plot(0, 0, 1, false, 1);
        screen.Copy(BitmapId.Page2, new Rect(0, 0, 2, 2), BitmapId.Page1, (4, 4), 1);
        Assert.Equal(1, screen.GetBitmap(BitmapId.Page1).Pixel(4, 4));
    }
}
