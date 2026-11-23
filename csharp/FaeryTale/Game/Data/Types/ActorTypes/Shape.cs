namespace FaeryTale.Game.Data;

/// One actor (struct shape): the hero, a monster, a person or an object lying around.
/// Positions are world coordinates in pixels; RelX/RelY are the position on screen. Kind says what
/// sort of thing it is and Race its species. State is what it is doing now (walking, fighting,
/// dead ...), Goal and Tactic are what an enemy is trying to do and how. Facing is a direction 0-7.
public sealed class Shape
{
    public ushort AbsX;
    public ushort AbsY;
    public ushort RelX;
    public ushort RelY;
    public sbyte Kind;
    public byte Race;
    public sbyte Index;
    public sbyte Visible;
    public sbyte Weapon;
    public sbyte Environ;
    public sbyte Goal;
    public sbyte Tactic;
    public sbyte State;
    public sbyte Facing;
    public short Vitality;
    public sbyte VelX;
    public sbyte VelY;
}
