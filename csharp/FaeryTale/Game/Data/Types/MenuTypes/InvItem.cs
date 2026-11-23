namespace FaeryTale.Game.Data;

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
