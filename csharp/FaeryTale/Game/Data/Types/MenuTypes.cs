namespace FaeryTale.Game.Data;

/// One of the game's menus (struct menu): the text labels, how many entries it has, the colour of
/// the entries, and for each entry whether it is enabled. In Enabled, bit 0 means selected, and the
/// value (2 = plain, 3 = on, 8-10 = from an item the hero has) says how it is drawn.
public sealed class Menu
{
    public byte[] LabelList = Array.Empty<byte>();
    public sbyte Num;
    public sbyte Color;
    public sbyte[] Enabled = new sbyte[12];

    /// An independent copy, so the game can change its menus without touching the start tables.
    public Menu Copy() => new()
    {
        LabelList = LabelList,
        Num = Num,
        Color = Color,
        Enabled = (sbyte[])Enabled.Clone(),
    };
}

/// A keyboard shortcut for a menu entry (struct letters).
public sealed class Letter
{
    public sbyte Key;
    public sbyte Menu;
    public sbyte Choice;
}

/// The brothers' starting stats (struct bro).
public sealed class Bro
{
    public sbyte Brave;
    public sbyte Luck;
    public sbyte Kind;
    public sbyte Wealth;
}

/// One kind of item the hero can carry (struct inv_item): where its picture is, how it is stacked
/// on the item list screen, and its name.
public sealed class InvItem
{
    public byte ImageNumber;
    public byte Xoff;
    public byte Yoff;
    public byte Ydelta;
    public byte ImgOff;
    public byte ImgHeight;
    public byte Maxshown;
    public string Name = "";
}
