using FaeryTale.Platform;

namespace FaeryTale.Game.Remastered;

/// One square of the hotbar: which item of the pack it shows, whether it is a spell (else a weapon), and
/// where its top left corner is in the window (pixels).
public readonly record struct HotbarSlot(int Item, bool IsSpell, int X, int Y);
