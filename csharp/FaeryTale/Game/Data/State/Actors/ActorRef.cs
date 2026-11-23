namespace FaeryTale.Game.Data;

/// Names one actor of Actors.List by its number, so code can hold on to "that actor" while it
/// calls others that change the list.
public readonly record struct ActorRef(int Slot);
