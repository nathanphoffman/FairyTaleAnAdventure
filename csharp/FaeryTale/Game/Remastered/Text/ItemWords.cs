using System.Text;
using FaeryTale.Game.Data;
using FaeryTale.Game.Reimagined;

namespace FaeryTale.Game.Remastered;

/// Where an item's name stands in a line: the first character, how many, and which pack slot.
public readonly record struct ItemWord(int Start, int Length, int Slot);

/// Finds the names of items (Sword, Gold Key, ...) in a line of the message log, so Remastered and
/// Reimagined can colour and underline them and say what they are when the pointer rests on one.
public static class ItemWords
{
    // The pack slots that are real items (after these come gold and the quiver, which are not carried).
    private const int LAST_SLOT = 30;

    /// The item names in the line, left to right. A name counts only as a whole word, spelt as the
    /// game spells it ("Sword", not "swordsman"), and only inside one line.
    public static List<ItemWord> Find(Data.Game game, ReadOnlySpan<byte> line)
    {
        string text = Encoding.Latin1.GetString(line);
        List<ItemWord> found = [];

        for (int slot = 0; slot <= LAST_SLOT; slot++)
        {
            AddAll(found, text, InfernoSkull.NameOf(game, slot), slot);
        }

        found.Sort((a, b) => a.Start.CompareTo(b.Start));

        return found;
    }

    // Adds every whole-word place the name stands in the text.
    private static void AddAll(List<ItemWord> found, string text, string name, int slot)
    {
        int from = 0;

        while (text.IndexOf(name, from, StringComparison.Ordinal) is var start and >= 0)
        {
            int end = start + name.Length;

            if (!IsLetterAt(text, start - 1) && !IsLetterAt(text, end))
            {
                found.Add(new ItemWord(start, name.Length, slot));
            }

            from = end;
        }
    }

    private static bool IsLetterAt(string text, int index) => index >= 0 && index < text.Length && char.IsLetter(text[index]);
}
