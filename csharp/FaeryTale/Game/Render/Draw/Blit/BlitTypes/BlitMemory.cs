using FaeryTale.Gfx;

namespace FaeryTale.Game.Render;

/// The memory a blit can reach: the bit planes being drawn on, the two work regions and the sprite
/// sheets. A read outside an area gives zeros, and a write outside it is dropped.
internal sealed class BlitMemory(Bitmap bitmap, byte[] queue, byte[] mask, byte[] shapes)
{
    private const int BYTE_BITS = 8;

    public byte Byte(Loc offset)
    {
        (byte[] bytes, long baseOffset) = AreaBytes(offset.Area);
        long index = baseOffset + offset.Pos;

        return index >= 0 && index < bytes.Length ? bytes[index] : (byte)0;
    }

    public void SetByte(Loc offset, byte value)
    {
        (byte[] bytes, long baseOffset) = AreaBytes(offset.Area);
        long index = baseOffset + offset.Pos;

        if (index >= 0 && index < bytes.Length)
        {
            bytes[index] = value;
        }
    }

    /// Reads a big-endian 16-bit word (the first byte is the high half).
    public uint ReadWord(Loc offset) => ((uint)Byte(offset) << BYTE_BITS) | Byte(offset.Add(1));

    /// Writes a big-endian 16-bit word (the high byte is stored first).
    public void WriteWord(Loc offset, uint value)
    {
        SetByte(offset, (byte)(value >> BYTE_BITS));  // the high byte of the 16-bit value is bits 8 and up
        SetByte(offset.Add(1), (byte)value);
    }

    // The bytes of an area, and how far in the position 0 of the area is.
    private (byte[] Bytes, long Base) AreaBytes(Area area) => area.Kind switch
    {
        AreaKind.Plane when area.PlaneNumber < bitmap.Depth => (bitmap.PaddedPlane(area.PlaneNumber), Bitmap.PAD),
        AreaKind.Plane => ([], 0),
        AreaKind.Queue => (queue, 0),
        AreaKind.Mask => (mask, 0),
        _ => (shapes, 0),
    };
}
