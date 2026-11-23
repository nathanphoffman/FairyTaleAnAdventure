namespace FaeryTale.Game.Data;

/// One animation frame of an actor (struct state): which figure image to draw, which weapon image
/// goes with it, and where the weapon sits relative to the actor.
public sealed class StateEntry
{
    public sbyte Figure;
    public sbyte WpnNo;
    public sbyte WpnX;
    public sbyte WpnY;
}
