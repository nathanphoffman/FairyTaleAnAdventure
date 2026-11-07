namespace FaeryTale.Game.Data;

/// Everything that changes while the game runs, in one value that is passed down from the main
/// loop. It starts out zero apart from the few values set here.
public sealed class Game
{
    public const ushort STARTING_REGION = 3;

    public Experience Experience;

    /// The fireball and flames of the inferno skull (Reimagined); idle when none is burning.
    public readonly Reimagined.InfernoBlast Inferno = new();

    /// The difficulty of Remastered and Reimagined (it can be changed in the game menu).
    public DifficultyLevel Difficulty = DifficultyLevel.Normal;

    /// The part of a point of damage not yet dealt to each actor, when a difficulty scales damage
    /// down: 0.4 of a point three times in a row makes one point.
    public double[] DamageCarry = new double[Actors.ACTOR_SLOTS];
    public Witch Witch = new();
    public Ui Ui = new();
    public EncounterState Encounter = new();
    public MapData Map = new();
    public ushort[] SavePadding = new ushort[3];
    public Rng Rng = new();
    public Extent[] Extents = (Extent[])Tables.InitialExtents.Clone();
    public Menu[] Menus = Tables.InitialMenus.Select(menu => menu.Copy()).ToArray();
    public ObjectLists Objects = new();

    /// The palette of the play field at full light; the fades start from it.
    public ushort[] PageColors = (ushort[])ObjectTables.InitialPageColors.Clone();
    public Actors Actors = new();
    public World World = new() { Region = STARTING_REGION };
    public Hero Hero = new();

    /// The actor r names.
    public Shape Actor(ActorRef r) => Actors.Actor(r);

    /// The missile r names.
    public Missile Missile(MissileRef r) => Actors.Missile(r);
}
