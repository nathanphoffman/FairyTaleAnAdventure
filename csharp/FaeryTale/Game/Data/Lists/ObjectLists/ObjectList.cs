namespace FaeryTale.Game.Data;

/// One of the object lists: the global one or the list of a region.
public readonly record struct ObjectList(int Region)
{
    public static readonly ObjectList Global = new(-1);

    public bool IsGlobal => Region < 0;
}
