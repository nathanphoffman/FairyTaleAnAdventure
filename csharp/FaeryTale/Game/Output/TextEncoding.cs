using System.Text;

namespace FaeryTale.Game.Output;

/// The game's text is single bytes (the classic character set), so Latin-1 maps each character
/// to one byte and back without losing anything.
public static class TextEncoding
{
    public static byte[] ToBytes(string text) => Encoding.Latin1.GetBytes(text);

    public static string FromBytes(ReadOnlySpan<byte> bytes) => Encoding.Latin1.GetString(bytes);
}
