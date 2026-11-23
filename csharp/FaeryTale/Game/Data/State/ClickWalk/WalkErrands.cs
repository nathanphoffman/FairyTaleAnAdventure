namespace FaeryTale.Game.Data;

/// What the hero does on arriving: talk to a person, take an item, or look around.
public sealed class WalkErrands
{
    /// A click on a person: the actor to walk up to before the choices of the conversation appear.
    public int? PersonToTalkTo;

    /// A click on something to pick up: the actor to walk to.
    public int? ItemToTake;

    /// The item to take now that the walk ended near it (set when the walk ends, used on the next step).
    public int? ArrivedToTake;

    /// A shift click: look around when the walk is over (ArrivedToLook is that moment).
    public bool LookWhenThere;
    public bool ArrivedToLook;
    public (int X, int Y) LookSpot;
}
