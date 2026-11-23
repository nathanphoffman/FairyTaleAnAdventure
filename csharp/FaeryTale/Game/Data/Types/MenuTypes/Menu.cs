namespace FaeryTale.Game.Data;

/// One of the game's menus (struct menu): the text labels, how many entries it has, the colour of
/// the entries, and for each entry whether it is enabled. In Enabled, bit 0 means selected, and the
/// value (2 = plain, 3 = on, 8-10 = from an item the hero has) says how it is drawn.
public sealed class Menu
{
    // The most entries a menu can have.
    private const int MAX_MENU_ENTRIES = 12;

    public byte[] LabelList = Array.Empty<byte>();
    public sbyte Num;
    public sbyte Color;
    public sbyte[] Enabled = new sbyte[MAX_MENU_ENTRIES];

    /// An independent copy, so the game can change its menus without touching the start tables.
    public Menu Copy() => new()
    {
        LabelList = LabelList,
        Num = Num,
        Color = Color,
        Enabled = (sbyte[])Enabled.Clone(),
    };
}
