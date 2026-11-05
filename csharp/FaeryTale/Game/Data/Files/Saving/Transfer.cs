using System.Buffers.Binary;

namespace FaeryTale.Game.Files;

/// The bytes of a save file being written or read, field by field. The same code does both: each
/// method writes a value when saving and reads it back when loading, so the order of the calls
/// (see GameTransfer) is the file format. Numbers are big-endian, as the original's 68000 kept them.
public sealed class Transfer
{
    private const int WORD_BYTES = 2;

    private readonly bool saving;
    private readonly List<byte> written = new();
    private readonly byte[] source;
    private int position;

    private Transfer(bool saving, byte[] source)
    {
        this.saving = saving;
        this.source = source;
    }

    /// Starts a new save file.
    public static Transfer ForSaving() => new(true, Array.Empty<byte>());

    /// Reads from the bytes of a save file.
    public static Transfer ForLoading(byte[] data) => new(false, data);

    /// The bytes written so far.
    public byte[] Finish() => written.ToArray();

    /// Writes the bytes, or fills them from the file. A file that ends early leaves the rest as it was.
    public void Bytes(Span<byte> buffer)
    {
        if (saving)
        {
            written.AddRange(buffer.ToArray());

            return;
        }

        int end = Math.Min(position + buffer.Length, source.Length);
        int available = Math.Max(end - position, 0);

        source.AsSpan(position, available).CopyTo(buffer);
        position += buffer.Length;
    }

    public void Byte(ref byte value)
    {
        Span<byte> one = [value];

        Bytes(one);
        value = one[0];
    }

    public void Byte(ref sbyte value)
    {
        byte raw = unchecked((byte)value);

        Byte(ref raw);
        value = unchecked((sbyte)raw);
    }

    public void Word(ref ushort value)
    {
        Span<byte> two = stackalloc byte[WORD_BYTES];

        BinaryPrimitives.WriteUInt16BigEndian(two, value);
        Bytes(two);
        value = BinaryPrimitives.ReadUInt16BigEndian(two);
    }

    public void Word(ref short value)
    {
        ushort raw = unchecked((ushort)value);

        Word(ref raw);
        value = unchecked((short)raw);
    }
}
