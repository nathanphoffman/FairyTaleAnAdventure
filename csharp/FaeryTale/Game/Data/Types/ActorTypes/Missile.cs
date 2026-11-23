namespace FaeryTale.Game.Data;

/// An arrow, rock or fireball in flight (struct missile).
public sealed class Missile
{
    public ushort AbsX;
    public ushort AbsY;
    public sbyte MissileType;
    public sbyte TimeOfFlight;
    public sbyte Speed;
    public sbyte Direction;
    public sbyte Archer;
}
