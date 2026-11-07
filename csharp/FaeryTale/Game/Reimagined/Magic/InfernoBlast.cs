namespace FaeryTale.Game.Reimagined;

/// What the inferno skull has in the air: one fireball flying, then the flames it leaves where it
/// burst. They are drawn like the missiles of the game (a Missile holds a spot and a picture), but
/// are kept apart because the game has only six missile places and the saved game has no room for more.
public sealed class InfernoBlast
{
    /// The fireball flying, or null when none is.
    public Data.Missile? Fireball;

    /// The flames burning where the fireball burst; each one's TimeOfFlight counts its age.
    public readonly List<Data.Missile> Flames = [];

    /// Whether anything of the skull is still in the air or burning.
    public bool Active => Fireball is not null || Flames.Count > 0;
}
