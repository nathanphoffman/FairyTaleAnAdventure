namespace FaeryTale.Game.Output;

/// One thing the rules asked an output to do, written as text such as PrintNumber(5, 2) so tests
/// can compare whole sessions with plain strings.
public readonly record struct Call(string Name, string Arguments)
{
    public override string ToString() => $"{Name}({Arguments})";
}
