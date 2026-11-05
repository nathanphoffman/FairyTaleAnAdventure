using System.Text;
using FaeryTale.Testing;

namespace FaeryTale.Tests;

public class PpmWriterTests
{
    [Fact]
    public void HasAHeaderThenThreeBytesPerPixel()
    {
        byte[] ppm = PpmWriter.FrameAsPpm(new uint[] { 0xFF8040, 0x000001 }, 2, 1);
        int headerLength = Encoding.ASCII.GetBytes("P6\n2 1\n255\n").Length;

        Assert.Equal(headerLength + 6, ppm.Length);
        Assert.Equal(new byte[] { 0xFF, 0x80, 0x40, 0, 0, 1 }, ppm[headerLength..]);
    }
}
