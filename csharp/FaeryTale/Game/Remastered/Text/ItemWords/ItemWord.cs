using System.Text;
using FaeryTale.Game.Reimagined;

namespace FaeryTale.Game.Remastered;

/// Where an item's name stands in a line: the first character, how many, and which pack slot.
public readonly record struct ItemWord(int Start, int Length, int Slot);
