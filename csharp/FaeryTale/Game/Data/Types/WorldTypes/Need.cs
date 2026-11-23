namespace FaeryTale.Game.Data;

/// The disk blocks one region needs (struct need): four image banks, two terrain tables, the
/// sector data, the region map and the character set. The game also keeps one of these for what is
/// loaded now, so it can be changed in place; Copy makes an independent one.
public sealed class Need
{
    // The number of picture banks the game loads.
    private const int IMAGE_BANK_COUNT = 4;

    public ushort[] Image = new ushort[IMAGE_BANK_COUNT];
    public ushort Terra1;
    public ushort Terra2;
    public ushort Sector;
    public ushort Region;
    public ushort Setchar;

    public Need Copy() => new()
    {
        Image = (ushort[])Image.Clone(),
        Terra1 = Terra1,
        Terra2 = Terra2,
        Sector = Sector,
        Region = Region,
        Setchar = Setchar,
    };
}
