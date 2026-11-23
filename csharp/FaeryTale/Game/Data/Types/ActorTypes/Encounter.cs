namespace FaeryTale.Game.Data;

/// One kind of monster (struct encounter): hit points, how aggressive, which weapon row it uses,
/// how clever, what treasure it drops, and which sprite file it needs.
public sealed class Encounter
{
    public sbyte Hitpoints;
    public sbyte Agressive;
    public sbyte Arms;
    public sbyte Cleverness;
    public sbyte Treasure;
    public sbyte FileId;
}
