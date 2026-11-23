namespace FaeryTale.Game;

public readonly record struct Command(CommandKind Kind, int First, int Second);
